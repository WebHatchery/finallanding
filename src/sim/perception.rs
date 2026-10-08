//! Perception: what a survivor can sense from where they stand.
//!
//! Survivors only learn about nodes, fauna and each other inside their sight
//! radius, which shrinks at night and in storms. Seeing a dangerous creature
//! interrupts deliberation and triggers a warning to everyone in earshot.

use super::Ctx;
use crate::agents::beliefs::{NodeBelief, ThreatBelief};
use crate::agents::messages::{Message, MessageKind};
use crate::agents::Agent;
use crate::data::{game_data, Need};
use crate::world::Calendar;

const EARSHOT: f32 = 14.0;

pub fn sight_radius(ctx: &Ctx) -> f32 {
    let agents = &game_data().balance.agents;
    let mut radius = agents.sight_radius + ctx.colony.modifiers.sight;
    if !ctx.calendar.is_daylight() {
        radius *= agents.night_sight_factor;
    }
    if ctx.world.weather.is_storm() {
        radius *= agents.storm_sight_factor;
    }
    radius
}

fn perceive_nodes(agent: &mut Agent, ctx: &Ctx, radius: f32) {
    let now = ctx.calendar.tick;
    for node in &ctx.world.nodes {
        if node.tile.distance(agent.tile) <= radius {
            agent.beliefs.note_node(
                node.id,
                NodeBelief {
                    kind: node.kind,
                    tile: node.tile,
                    amount: node.amount,
                    seen_tick: now,
                },
            );
        }
    }
    // Remembered nodes that should be visible but are gone were used up.
    let visible_ids: Vec<u32> = agent
        .beliefs
        .nodes
        .iter()
        .filter(|(_, b)| b.tile.distance(agent.tile) <= radius)
        .map(|(id, _)| *id)
        .collect();
    for id in visible_ids {
        if ctx.world.node(id).is_none() {
            agent.beliefs.forget_node(id);
        }
    }
}

/// Returns true when a new dangerous creature was spotted.
fn perceive_creatures(agent: &mut Agent, ctx: &mut Ctx, radius: f32) -> bool {
    let now = ctx.calendar.tick;
    let mut alarmed = false;
    let balance = &game_data().balance;
    let sightings: Vec<ThreatBelief> = ctx
        .world
        .creatures
        .iter()
        .filter(|c| c.tile.distance(agent.tile) <= radius)
        .map(|c| ThreatBelief {
            creature: c.id,
            tile: c.tile,
            dangerous: balance.creatures.get(&c.kind).is_some_and(|b| b.aggressive),
            seen_tick: now,
        })
        .collect();
    for sighting in sightings {
        let known = agent
            .beliefs
            .threats
            .iter()
            .any(|t| t.creature == sighting.creature);
        if !known {
            alarmed = true;
            agent
                .needs
                .change(Need::Safety, -balance.needs.safety_loss_on_threat);
            warn_others(agent, ctx, &sighting);
        }
        agent.beliefs.note_threat(sighting);
    }
    agent.beliefs.threats.retain(|threat| {
        threat.tile.distance(agent.tile) > radius
            || ctx.world.creatures.iter().any(|c| c.id == threat.creature)
    });
    alarmed
}

fn warn_others(agent: &Agent, ctx: &mut Ctx, sighting: &ThreatBelief) {
    let now = ctx.calendar.tick;
    for other in ctx.others.iter().filter(|o| o.id != agent.id && o.present) {
        if other.tile.distance(agent.tile) <= EARSHOT {
            ctx.effects.messages.push(Message {
                from: agent.id,
                to: other.id,
                kind: MessageKind::Warn(sighting.clone()),
                tick: now,
            });
        }
    }
}

fn perceive_people(agent: &mut Agent, ctx: &Ctx, radius: f32) {
    let social = &game_data().balance.social;
    let interval = game_data().balance.agents.perception_interval as f32;
    let familiarity = Calendar::per_tick(social.proximity_familiarity_per_day) * interval;
    let shared = Calendar::per_tick(social.shared_work_opinion_per_day) * interval;
    let my_goal = agent
        .current_goal()
        .filter(|g| g.work_type().is_some())
        .map(|g| g.key());
    agent.beliefs.nearby.clear();
    for other in ctx.others.iter().filter(|o| o.id != agent.id && o.present) {
        if other.tile.distance(agent.tile) > radius {
            continue;
        }
        agent.beliefs.nearby.push((other.id, other.tile));
        let relation = agent.beliefs.relation_mut(other.id);
        relation.familiarity = (relation.familiarity + familiarity).min(100.0);
        if my_goal.is_some() && other.goal.map(|g| g.key()) == my_goal {
            relation.change_opinion(shared);
        }
    }
}

/// Sense the surroundings. Returns true when something demands an immediate
/// rethink (a new threat).
pub fn perceive(agent: &mut Agent, ctx: &mut Ctx) -> bool {
    let interval = game_data().balance.agents.perception_interval as u64;
    if !(ctx.calendar.tick + agent.id as u64).is_multiple_of(interval) {
        return false;
    }
    let radius = sight_radius(ctx);
    ctx.world.map.reveal_circle(agent.tile, radius);
    perceive_nodes(agent, ctx, radius);
    perceive_people(agent, ctx, radius);
    if is_at_storage(agent, ctx) {
        agent.beliefs.stock = ctx.colony.stock;
    }
    perceive_creatures(agent, ctx, radius)
}

/// Standing at a store lets a survivor see what is actually in it.
fn is_at_storage(agent: &Agent, ctx: &Ctx) -> bool {
    ctx.world
        .built()
        .filter(|s| s.def().storage > 0.0 || s.def().kitchen)
        .any(|s| s.footprint.distance_to(agent.tile) <= 1.5)
}
