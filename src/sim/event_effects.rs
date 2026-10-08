//! Applying an event's effects to the world and the colony.

use super::chronicle_text::line;
use super::research::reveal_any;
use super::{creatures, environment, Sim};
use crate::colony::chronicle::Category;
use crate::data::events::{EventEffect, Severity};
use crate::data::{game_data, NodeKind};
use crate::world::{Calendar, CropStage, ResourceNode};

fn spawn_wreckage(sim: &mut Sim, nodes: u32) {
    for _ in 0..nodes {
        for _ in 0..30 {
            let tile = crate::world::Tile::new(
                sim.rng.range_i32(2, sim.world.map.width - 2),
                sim.rng.range_i32(2, sim.world.map.height - 2),
            );
            let free =
                sim.world.map.is_walkable(tile) && sim.world.map.structure_on(tile).is_none();
            if free {
                let id = sim.world.allocate_node_id();
                let amount = sim.rng.range_f32(40.0, 90.0);
                sim.world.add_node(ResourceNode {
                    id,
                    kind: NodeKind::Wreckage,
                    tile,
                    amount,
                    max_amount: amount,
                    find: None,
                });
                break;
            }
        }
    }
}

fn illness(sim: &mut Sim, count: u32) {
    let ticks = Calendar::ticks_per_day();
    let mut candidates: Vec<u32> = sim
        .present()
        .filter(|a| !a.health.ill)
        .map(|a| a.id)
        .collect();
    for _ in 0..count {
        if candidates.is_empty() {
            break;
        }
        let index = sim.rng.below(candidates.len());
        let id = candidates.swap_remove(index);
        if let Some(agent) = sim.agent_mut(id) {
            agent.health.ill = true;
            agent.mind.add("sick", None, ticks);
        }
    }
}

fn blight(sim: &mut Sim, fraction: f32) {
    for structure in sim.world.structures.iter_mut() {
        let indoor = structure.def().farm.as_ref().is_some_and(|f| f.indoor);
        if let Some(crop) = structure.crop.as_mut() {
            if !indoor && crop.stage != CropStage::Fallow && sim.rng.chance(fraction) {
                crop.stage = CropStage::Fallow;
                crop.growth = 0.0;
            }
        }
    }
}

fn arrivals(sim: &mut Sim, count: u32) {
    let max = game_data().balance.colony.max_population;
    let ticks = Calendar::ticks_per_day();
    for _ in 0..count {
        if sim.population() >= max {
            break;
        }
        let tile = sim.arrival_tile();
        let id = sim.spawn_survivor(tile);
        sim.colony.stats.arrivals += 1;
        let name = sim
            .agent(id)
            .map(|a| a.given_name.clone())
            .unwrap_or_default();
        if let Some(agent) = sim.agent_mut(id) {
            agent.mind.add("recruited", None, ticks);
        }
        let text = line("arrival", sim.calendar.tick, &[("name", &name)]);
        sim.record(Category::Life, 1, text, vec![id]);
    }
}

fn apply(sim: &mut Sim, effect: &EventEffect) {
    match effect {
        EventEffect::Weather { weather, days } => environment::set_weather(sim, *weather, *days),
        EventEffect::Creatures { kind, count } => creatures::spawn(sim, *kind, *count),
        EventEffect::Resource { resource, amount } => sim.colony.stock.add(*resource, *amount),
        EventEffect::Arrivals { count } => arrivals(sim, *count),
        EventEffect::Illness { count } => illness(sim, *count),
        EventEffect::Blight { fraction } => blight(sim, *fraction),
        EventEffect::Wreckage { nodes } => spawn_wreckage(sim, *nodes),
        EventEffect::Insight { branch, amount } => {
            sim.colony.research.add_insight(*branch, *amount)
        }
        EventEffect::RevealTech => {
            if let Some(tech) = reveal_any(&mut sim.colony, &mut sim.rng, false) {
                let name = game_data()
                    .tech(&tech)
                    .map(|t| t.name.clone())
                    .unwrap_or(tech);
                let text = line("tech_revealed", sim.calendar.tick, &[("tech", &name)]);
                sim.record(Category::Discovery, 1, text, Vec::new());
            }
        }
        EventEffect::Damage { fraction } => {
            for structure in sim.world.structures.iter_mut().filter(|s| s.is_built()) {
                if sim.rng.chance(*fraction * 3.0) {
                    structure.condition = (structure.condition - 35.0).max(5.0);
                }
            }
        }
        EventEffect::Mood { amount, days } => {
            let ticks = Calendar::ticks_for_days(*days);
            for agent in sim.agents.iter_mut().filter(|a| a.is_present()) {
                agent.mind.add_custom("colony_event", *amount, ticks);
            }
        }
        EventEffect::Expedition { site } => {
            if !sim.colony.expeditions.revealed_sites.contains(site) {
                sim.colony.expeditions.revealed_sites.push(site.clone());
            }
        }
    }
}

/// Fire an event by id: apply its effects and record it in the chronicle.
pub fn fire_event(sim: &mut Sim, id: &str) {
    let Some(event) = game_data().event(id) else {
        eprintln!("unknown event '{id}'");
        return;
    };
    for effect in &event.effects {
        apply(sim, effect);
    }
    let day = sim.calendar.day();
    sim.colony.events_fired.push(event.id.clone());
    if event.cooldown_days > 0 {
        sim.colony
            .event_cooldowns
            .insert(event.id.clone(), day + event.cooldown_days);
    }
    let (category, importance) = match event.severity {
        Severity::Calm => (Category::Story, 1),
        Severity::Notable => (Category::Story, 2),
        Severity::Danger => (Category::Danger, 2),
    };
    let text = format!("{} — {}", event.title, event.text);
    sim.record(category, importance, text, Vec::new());
}
