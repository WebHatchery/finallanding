//! Per-tick bodily change: need decay, warmth, health and mood.

use super::Ctx;
use crate::agents::{Activity, Agent};
use crate::data::{game_data, Need};
use crate::world::{Calendar, World};

/// Indoor shelter and heat sources reaching where the survivor stands.
fn shelter_and_heat(agent: &Agent, world: &World) -> (bool, f32) {
    if let Some(inside) = agent.inside.and_then(|id| world.structure(id)) {
        let def = inside.def();
        return (def.indoor, if def.heat > 0.0 { 1.0 } else { 0.0 });
    }
    (false, world.heat_at(agent.tile))
}

fn update_warmth(agent: &mut Agent, ctx: &Ctx, decay_scale: f32) {
    let needs = &game_data().balance.needs;
    let (sheltered, heat) = shelter_and_heat(agent, ctx.world);
    let temperature = ctx.world.weather.temperature;
    let mut change = 0.0;
    if heat > 0.0 {
        change += needs.heat_warmth_gain_per_day * heat;
    }
    if sheltered {
        change += needs.shelter_warmth_gain_per_day;
    } else if temperature < needs.comfortable_temperature {
        let cold = (needs.comfortable_temperature - temperature) * needs.cold_loss_per_degree;
        let gear = ctx.colony.modifiers.need_decay[Need::Warmth.index()];
        change -= cold * gear * decay_scale;
    } else {
        change += needs.shelter_warmth_gain_per_day * 0.5;
    }
    agent.needs.change(Need::Warmth, Calendar::per_tick(change));
}

fn decay_needs(agent: &mut Agent, ctx: &Ctx) {
    let data = game_data();
    let decay_scale = ctx.colony.difficulty_value(|d| d.need_decay);
    for need in [Need::Food, Need::Rest, Need::Social, Need::Recreation] {
        if need == Need::Rest && agent.activity == Activity::Sleeping {
            continue;
        }
        let base = data
            .balance
            .needs
            .decay_per_day
            .get(&need)
            .copied()
            .unwrap_or(0.0);
        let rate = base
            * agent.personality.decay(need)
            * ctx.colony.modifiers.need_decay[need.index()]
            * decay_scale;
        agent.needs.change(need, -Calendar::per_tick(rate));
    }
    agent.needs.change(
        Need::Safety,
        Calendar::per_tick(data.balance.needs.safety_recovery_per_day),
    );
    update_warmth(agent, ctx, decay_scale);
}

fn update_health(agent: &mut Agent, ctx: &mut Ctx) {
    let health = &game_data().balance.health;
    let mut change = 0.0;
    if agent.needs.get(Need::Food) <= 0.5 {
        change -= health.starvation_damage_per_day;
        if agent.mind.thoughts.iter().all(|t| t.id != "went_hungry") {
            agent
                .mind
                .add("went_hungry", None, Calendar::ticks_per_day());
        }
    }
    if agent.needs.get(Need::Warmth) <= 0.5 {
        change -= health.hypothermia_damage_per_day;
        if !agent.mind.has("freezing") {
            agent.mind.add("freezing", None, Calendar::ticks_per_day());
        }
    }
    if agent.health.ill {
        change -= health.illness_damage_per_day * if agent.health.treated { 0.4 } else { 1.0 };
        let recovery = health.illness_recovery_chance_per_day
            + if agent.health.treated {
                health.treated_recovery_bonus
            } else {
                0.0
            };
        if ctx.rng.chance(Calendar::per_tick(recovery)) {
            agent.health.ill = false;
            agent.health.treated = false;
            agent.mind.remove("sick");
        }
    }
    if change >= 0.0 && agent.needs.get(Need::Food) > 10.0 {
        let in_ward = agent
            .inside
            .and_then(|id| ctx.world.structure(id))
            .is_some_and(|s| s.def().care_beds > 0);
        let rate = match (agent.activity == Activity::Recovering, in_ward) {
            (true, true) => health.care_regen_per_day,
            (true, false) => health.regen_per_day * 2.0,
            _ => health.regen_per_day,
        };
        change += rate * ctx.colony.modifiers.heal_rate;
    }
    agent.health.hp = (agent.health.hp + Calendar::per_tick(change)).clamp(0.0, 100.0);
}

/// Current mood from baseline, personality, needs, thoughts, technology and policy.
pub fn compute_mood(agent: &Agent, ctx: &Ctx) -> f32 {
    let data = game_data();
    let expectations =
        data.balance.mood.expectation_per_act * (ctx.colony.campaign.act as f32 - 1.0);
    let mut mood = data.balance.mood.baseline + agent.personality.optimism - expectations;
    mood += agent.needs.mood_pressure();
    mood += agent.mind.total();
    mood += ctx.colony.modifiers.mood;
    for effects in ctx.colony.policy_effects() {
        mood += effects.mood;
        for (trait_id, amount) in &effects.trait_mood {
            if agent.has_trait(trait_id) {
                mood += amount;
            }
        }
    }
    if agent.health.hp < 50.0 {
        mood -= (50.0 - agent.health.hp) * 0.2;
    }
    mood.clamp(0.0, 100.0)
}

pub fn update(agent: &mut Agent, ctx: &mut Ctx) {
    decay_needs(agent, ctx);
    update_health(agent, ctx);
    agent.mind.tick(1);
    if (ctx.calendar.tick + agent.id as u64).is_multiple_of(10) {
        agent.mood = compute_mood(agent, ctx);
    }
}
