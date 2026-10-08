//! The deliberation cycle: when to reconsider, whether to switch goals, and
//! choosing (or replacing) a plan.

use super::Ctx;
use crate::agents::deliberation::{candidates, Candidate};
use crate::agents::goals::Goal;
use crate::agents::intention::{DecisionKind, Intention};
use crate::agents::plan_library::plans_for;
use crate::agents::plans::Plan;
use crate::agents::{Activity, Agent};
use crate::data::{game_data, Resource};
use crate::world::Calendar;

/// Survival goals are only displaced by other survival goals or a large margin.
fn switch_threshold(current: &Goal, best: &Goal) -> f32 {
    let commitment = game_data().balance.agents.commitment_bonus;
    if current.is_survival() && !best.is_survival() {
        1.0 + commitment * 3.0
    } else {
        1.0 + commitment
    }
}

/// Whether a new candidate is worth abandoning the current intention for.
pub fn should_switch(current: &Intention, current_utility: f32, best: &Candidate) -> bool {
    if best.goal.key() == current.goal.key() {
        return false;
    }
    if matches!(best.goal, Goal::Break(_)) {
        return true;
    }
    best.utility > current_utility.max(0.01) * switch_threshold(&current.goal, &best.goal)
}

/// Carried materials are set down when a survivor turns to something else,
/// so a hauler can pick them up later.
fn drop_unneeded_load(agent: &mut Agent, ctx: &mut Ctx, next: &Goal) {
    let keeps_load = matches!(next, Goal::Eat | Goal::Help { .. } | Goal::Cook);
    if let Some((resource, amount)) = agent.carrying {
        if !(keeps_load && resource == Resource::Food) {
            ctx.world.drop_item(agent.tile, resource, amount);
            agent.carrying = None;
        }
    }
}

fn adopt(agent: &mut Agent, ctx: &mut Ctx, candidate: Candidate, interrupted_by: Option<String>) {
    if let Some(by) = interrupted_by {
        agent
            .decisions
            .push(ctx.calendar.tick, DecisionKind::Interrupted { by });
    }
    drop_unneeded_load(agent, ctx, &candidate.goal);
    agent.decisions.push(
        ctx.calendar.tick,
        DecisionKind::Adopted {
            goal: candidate.goal,
            utility: candidate.utility,
            reason: candidate.reason,
        },
    );
    agent.path.clear();
    agent.inside = None;
    agent.intention = Some(Intention::new(
        candidate.goal,
        candidate.utility,
        ctx.calendar.tick,
    ));
}

/// Reconsider goals if due or interrupted.
fn deliberate(agent: &mut Agent, ctx: &mut Ctx, interrupted: bool) {
    let options = candidates(agent, &ctx.view());
    let Some(best) = options
        .iter()
        .max_by(|a, b| a.utility.total_cmp(&b.utility))
    else {
        return;
    };
    let best = Candidate {
        goal: best.goal,
        utility: best.utility,
        reason: best.reason.clone(),
    };
    match &agent.intention {
        None => adopt(agent, ctx, best, None),
        Some(current) => {
            let current_utility = options
                .iter()
                .find(|c| c.goal.key() == current.goal.key())
                .map(|c| c.utility)
                .unwrap_or(0.0);
            if should_switch(current, current_utility, &best) {
                let by = if interrupted {
                    Some(game_data().label("interrupt_percept").to_owned())
                } else {
                    None
                };
                adopt(agent, ctx, best, by);
            } else if let Some(intention) = agent.intention.as_mut() {
                intention.utility = current_utility;
            }
        }
    }
}

/// Choose the best applicable plan not yet tried; give up if none remain.
fn ensure_plan(agent: &mut Agent, ctx: &mut Ctx) {
    let Some(intention) = &agent.intention else {
        return;
    };
    if intention.plan.is_some() {
        return;
    }
    let goal = intention.goal;
    let tried = intention.tried.clone();
    let max_attempts = game_data().balance.agents.max_plan_attempts as usize;
    let option = if tried.len() >= max_attempts {
        None
    } else {
        plans_for(&goal, agent, &ctx.view())
            .into_iter()
            .find(|plan| !tried.iter().any(|t| t == plan.id))
    };
    match option {
        Some(option) => {
            agent.decisions.push(
                ctx.calendar.tick,
                DecisionKind::ChosePlan {
                    plan: option.id.to_owned(),
                },
            );
            if let Some(intention) = agent.intention.as_mut() {
                intention.plan = Some(Plan::new(option.id, option.steps));
            }
        }
        None => give_up(agent, ctx, goal),
    }
}

/// Every plan failed: back off from the goal for a while and feel it.
pub fn give_up(agent: &mut Agent, ctx: &mut Ctx, goal: Goal) {
    let backoff = game_data().balance.agents.goal_backoff_ticks as u64;
    agent
        .decisions
        .push(ctx.calendar.tick, DecisionKind::GaveUp { goal });
    agent.back_off(goal.key(), ctx.calendar.tick + backoff);
    if goal.is_survival() || goal.is_social() {
        agent
            .mind
            .add("frustrated", None, Calendar::ticks_per_day());
    }
    if matches!(goal, Goal::Help { .. }) {
        agent.help_task = None;
    }
    agent.intention = None;
    agent.activity = Activity::Idle;
    agent.next_deliberation = ctx.calendar.tick + 1;
}

pub fn think(agent: &mut Agent, ctx: &mut Ctx, interrupted: bool) {
    let due = ctx.calendar.tick >= agent.next_deliberation;
    if agent.intention.is_none() || due || interrupted {
        deliberate(agent, ctx, interrupted);
        let interval = game_data().balance.agents.deliberation_interval as u64;
        agent.next_deliberation = ctx.calendar.tick + interval + agent.id as u64 % 3;
    }
    ensure_plan(agent, ctx);
}
