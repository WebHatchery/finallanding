//! Social steps: talking, sending messages, handing things over, treating.

use super::execute_work::{practise, work_rate};
use super::{Ctx, Interaction};
use crate::agents::messages::{Message, MessageKind};
use crate::agents::plans::{FailReason, Step, StepOutcome};
use crate::agents::{Activity, Agent};
use crate::data::{game_data, Need, Resource, Skill};
use crate::world::{AgentId, Tile};

const TREAT_TICKS: f32 = 40.0;

/// Where another present survivor is, and whether they can be approached.
fn partner_tile(ctx: &Ctx, id: AgentId) -> Option<(Tile, bool)> {
    ctx.others
        .iter()
        .find(|s| s.id == id && s.present)
        .map(|s| (s.tile, s.is_available_to_talk()))
}

fn approach(agent: &mut Agent, ctx: &mut Ctx, tile: Tile, step_ticks: u32) -> Option<StepOutcome> {
    if tile.steps(agent.tile) <= 2 {
        agent.path.clear();
        return None;
    }
    if step_ticks > 160 {
        return Some(StepOutcome::Failed(FailReason::TimedOut));
    }
    if agent.path.is_empty() || step_ticks.is_multiple_of(8) {
        let destination = ctx.world.map.nearest_walkable(tile, 1).unwrap_or(tile);
        match ctx.world.find_path(agent.tile, destination) {
            Some(path) => agent.path = path,
            None => return Some(StepOutcome::Failed(FailReason::NoPath)),
        }
    }
    agent.activity = Activity::Walking;
    Some(StepOutcome::Running)
}

fn talk(
    agent: &mut Agent,
    ctx: &mut Ctx,
    with: AgentId,
    topic: crate::agents::plans::Topic,
    step_ticks: u32,
) -> StepOutcome {
    let Some((tile, available)) = partner_tile(ctx, with) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    if !available {
        return StepOutcome::Failed(FailReason::Refused);
    }
    if let Some(outcome) = approach(agent, ctx, tile, step_ticks) {
        return outcome;
    }
    agent.inside = None;
    agent.activity = Activity::Talking;
    let social = &game_data().balance.social;
    let started = agent
        .beliefs
        .relation(with)
        .map(|r| r.last_chat_tick)
        .unwrap_or(0);
    let elapsed = ctx.calendar.tick.saturating_sub(started) as u32;
    if started == 0 || elapsed > social.chat_ticks * 4 {
        agent.beliefs.relation_mut(with).last_chat_tick = ctx.calendar.tick;
        return StepOutcome::Running;
    }
    if elapsed < social.chat_ticks {
        return StepOutcome::Running;
    }
    agent.needs.change(Need::Social, social.chat_social_gain);
    agent.needs.change(Need::Recreation, 6.0);
    agent.stats.conversations += 1;
    practise(agent, ctx, Skill::Social);
    ctx.effects.interactions.push(Interaction {
        from: agent.id,
        to: with,
        topic,
    });
    StepOutcome::Done
}

fn give(agent: &mut Agent, ctx: &mut Ctx, to: AgentId, step_ticks: u32) -> StepOutcome {
    let Some((tile, _)) = partner_tile(ctx, to) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    if let Some(outcome) = approach(agent, ctx, tile, step_ticks) {
        return outcome;
    }
    let Some((resource, amount)) = agent.carrying.take() else {
        return StepOutcome::Failed(FailReason::StockEmpty);
    };
    ctx.effects.gifts.push((agent.id, to, resource, amount));
    ctx.effects.messages.push(Message {
        from: agent.id,
        to,
        kind: MessageKind::Delivered,
        tick: ctx.calendar.tick,
    });
    StepOutcome::Done
}

fn treat(agent: &mut Agent, ctx: &mut Ctx, patient: AgentId, step_ticks: u32) -> StepOutcome {
    let Some((tile, _)) = partner_tile(ctx, patient) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let needs_care = ctx.others.iter().any(|s| s.id == patient && s.needs_care);
    if !needs_care {
        return StepOutcome::Done;
    }
    if tile.steps(agent.tile) > 2 {
        return approach(agent, ctx, tile, step_ticks).unwrap_or(StepOutcome::Running);
    }
    agent.activity = Activity::Working(Skill::Medicine);
    practise(agent, ctx, Skill::Medicine);
    let rate = work_rate(agent, ctx, Skill::Medicine, false);
    if (step_ticks as f32) < TREAT_TICKS / rate.max(0.3) {
        return StepOutcome::Running;
    }
    let with_medicine = agent.carried(Resource::Medicine) >= 0.5;
    if with_medicine {
        agent.carrying = None;
    }
    ctx.effects
        .treatments
        .push((agent.id, patient, with_medicine));
    StepOutcome::Done
}

pub fn run(agent: &mut Agent, ctx: &mut Ctx, step: &Step, step_ticks: u32) -> StepOutcome {
    match step {
        Step::Talk { with, topic } => talk(agent, ctx, *with, *topic, step_ticks),
        Step::Send { to, kind } => {
            ctx.effects.messages.push(Message {
                from: agent.id,
                to: *to,
                kind: kind.clone(),
                tick: ctx.calendar.tick,
            });
            StepOutcome::Done
        }
        Step::Give(to) => give(agent, ctx, *to, step_ticks),
        Step::Treat(patient) => treat(agent, ctx, *patient, step_ticks),
        _ => StepOutcome::Failed(FailReason::Interrupted),
    }
}
