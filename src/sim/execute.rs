//! Executing the current plan step, and recovering when a step fails.

use super::cognition::give_up;
use super::{execute_life, execute_social, execute_work, Ctx};
use crate::agents::beliefs::FailedTarget;
use crate::agents::goals::Goal;
use crate::agents::intention::DecisionKind;
use crate::agents::plans::{FailReason, Step, StepOutcome, Target};
use crate::agents::{Activity, Agent};
use crate::world::Tile;

const MOVE_TIMEOUT_TICKS: u32 = 500;

/// Where a movement target currently is, as a tile to stand on.
pub fn resolve_target(agent: &Agent, ctx: &Ctx, target: Target) -> Option<Tile> {
    match target {
        Target::Tile(tile) => ctx.world.map.nearest_walkable(tile, 2),
        Target::Structure(id) => ctx.world.access_tile(id, agent.tile),
        Target::Node(id) => ctx.world.node_access_tile(id, agent.tile),
        Target::Agent(id) => {
            let other = ctx.others.iter().find(|s| s.id == id && s.present)?;
            match other.inside {
                Some(structure) => ctx.world.access_tile(structure, agent.tile),
                None => Some(other.tile),
            }
        }
    }
}

fn go_to(agent: &mut Agent, ctx: &mut Ctx, target: Target, step_ticks: u32) -> StepOutcome {
    let Some(destination) = resolve_target(agent, ctx, target) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let close_enough = match target {
        Target::Agent(_) => agent.tile.steps(destination) <= 1,
        _ => agent.tile == destination,
    };
    if close_enough {
        agent.path.clear();
        return StepOutcome::Done;
    }
    if step_ticks > MOVE_TIMEOUT_TICKS {
        return StepOutcome::Failed(FailReason::TimedOut);
    }
    let path_end = agent.path.back().copied();
    let needs_path = agent.path.is_empty() || path_end != Some(destination);
    let moving_target = matches!(target, Target::Agent(_)) && step_ticks.is_multiple_of(8);
    if needs_path || moving_target {
        match ctx.world.find_path(agent.tile, destination) {
            Some(path) => agent.path = path,
            None => return StepOutcome::Failed(FailReason::NoPath),
        }
    }
    agent.inside = None;
    if agent.activity != Activity::Fleeing {
        agent.activity = if agent.carrying.is_some() {
            Activity::Hauling
        } else {
            Activity::Walking
        };
    }
    StepOutcome::Running
}

fn dispatch(agent: &mut Agent, ctx: &mut Ctx, step: &Step, step_ticks: u32) -> StepOutcome {
    match step {
        Step::GoTo(target) => go_to(agent, ctx, *target, step_ticks),
        Step::Talk { .. } | Step::Send { .. } | Step::Give(_) | Step::Treat(_) => {
            execute_social::run(agent, ctx, step, step_ticks)
        }
        Step::TakeStock { .. }
        | Step::TakeMeal
        | Step::Deposit
        | Step::Deliver(_)
        | Step::Construct(_)
        | Step::Repair(_)
        | Step::Harvest(_)
        | Step::Forage(_)
        | Step::PickUp(_)
        | Step::Tend(_)
        | Step::Cook(_)
        | Step::Craft(_)
        | Step::Research(_)
        | Step::StudyRelic(_) => execute_work::run(agent, ctx, step, step_ticks),
        _ => execute_life::run(agent, ctx, step, step_ticks),
    }
}

/// Correct beliefs after a failure so the next plan does not repeat it.
fn learn_from_failure(agent: &mut Agent, ctx: &Ctx, step: &Step, reason: FailReason) {
    let now = ctx.calendar.tick;
    match (step, reason) {
        (Step::Harvest(node) | Step::Forage(node), _) | (Step::GoTo(Target::Node(node)), _) => {
            agent.beliefs.forget_node(*node);
            agent.beliefs.note_failure(FailedTarget::Node(*node), now);
        }
        (Step::TakeStock { resource, .. }, FailReason::StockEmpty) => {
            agent.beliefs.stock = ctx.colony.stock;
            agent
                .beliefs
                .stock
                .set(*resource, ctx.colony.stock.get(*resource));
        }
        (Step::TakeMeal, _) => agent.beliefs.stock = ctx.colony.stock,
        (Step::GoTo(Target::Structure(id)), _) | (Step::Sleep(Some(id)), _) => {
            agent
                .beliefs
                .note_failure(FailedTarget::Structure(*id), now);
        }
        (Step::GoTo(Target::Tile(tile)), _) => {
            agent.beliefs.note_failure(FailedTarget::Tile(*tile), now)
        }
        _ => {}
    }
}

fn fail_plan(agent: &mut Agent, ctx: &mut Ctx, step: Step, reason: FailReason) {
    learn_from_failure(agent, ctx, &step, reason);
    agent.path.clear();
    let Some(intention) = agent.intention.as_mut() else {
        return;
    };
    let plan_id = intention.plan.take().map(|p| p.id).unwrap_or_default();
    intention.tried.push(plan_id.clone());
    agent.decisions.push(
        ctx.calendar.tick,
        DecisionKind::PlanFailed {
            plan: plan_id,
            reason,
        },
    );
    agent.activity = Activity::Idle;
}

fn achieve(agent: &mut Agent, ctx: &mut Ctx) {
    let Some(intention) = agent.intention.take() else {
        return;
    };
    if intention.attempts() > 0 {
        agent.stats.plans_recovered += 1;
    }
    if matches!(intention.goal, Goal::Help { .. }) {
        agent.help_task = None;
        agent.stats.helped_others += 1;
    }
    agent.decisions.push(
        ctx.calendar.tick,
        DecisionKind::Achieved {
            goal: intention.goal,
        },
    );
    agent.next_deliberation = ctx.calendar.tick;
}

/// Run one tick of the current step and handle completion or failure.
pub fn run_step(agent: &mut Agent, ctx: &mut Ctx) {
    let Some(intention) = &agent.intention else {
        agent.activity = Activity::Idle;
        return;
    };
    let Some(plan) = &intention.plan else {
        return;
    };
    let Some(step) = plan.current().cloned() else {
        achieve(agent, ctx);
        return;
    };
    let step_ticks = plan.step_ticks;
    match dispatch(agent, ctx, &step, step_ticks) {
        StepOutcome::Running => {
            if let Some(plan) = agent.intention.as_mut().and_then(|i| i.plan.as_mut()) {
                plan.step_ticks += 1;
            }
        }
        StepOutcome::Done => {
            let complete = agent
                .intention
                .as_mut()
                .and_then(|i| i.plan.as_mut())
                .map(|plan| {
                    plan.advance();
                    plan.is_complete()
                })
                .unwrap_or(true);
            if complete {
                achieve(agent, ctx);
            }
        }
        StepOutcome::Failed(reason) => {
            fail_plan(agent, ctx, step, reason);
            let exhausted = agent.intention.as_ref().is_some_and(|i| {
                i.attempts() >= crate::data::game_data().balance.agents.max_plan_attempts as usize
            });
            if exhausted {
                if let Some(goal) = agent.current_goal() {
                    give_up(agent, ctx, goal);
                }
            }
        }
    }
}
