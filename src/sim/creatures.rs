//! Native fauna: simple reactive agents that raid the colony and flee from
//! defenders.

use super::chronicle_text::line;
use super::Sim;
use crate::colony::chronicle::Category;
use crate::data::{game_data, CreatureKind, Need, Resource};
use crate::world::{Calendar, Creature, CreatureMood, Point, Tile};
use std::collections::VecDeque;

fn edge_tile(sim: &mut Sim) -> Tile {
    let map = &sim.world.map;
    let (width, height) = (map.width, map.height);
    for _ in 0..40 {
        let tile = match sim.rng.below(4) {
            0 => Tile::new(sim.rng.range_i32(0, width), 0),
            1 => Tile::new(sim.rng.range_i32(0, width), height - 1),
            2 => Tile::new(0, sim.rng.range_i32(0, height)),
            _ => Tile::new(width - 1, sim.rng.range_i32(0, height)),
        };
        if sim.world.map.is_walkable(tile) {
            return tile;
        }
    }
    sim.world
        .map
        .nearest_walkable(Tile::new(0, 0), 20)
        .unwrap_or_default()
}

pub fn spawn(sim: &mut Sim, kind: CreatureKind, count: u32) {
    let act = sim.colony.campaign.act as f32;
    let scale = sim.colony.difficulty_value(|d| d.threat_scale) * (1.0 + (act - 1.0) * 0.25);
    let count = ((count as f32) * scale).round().max(1.0) as u32;
    let Some(balance) = game_data().balance.creatures.get(&kind) else {
        return;
    };
    let origin = edge_tile(sim);
    for _ in 0..count {
        let tile = sim
            .world
            .map
            .nearest_walkable(
                origin.offset(sim.rng.range_i32(-2, 3), sim.rng.range_i32(-2, 3)),
                3,
            )
            .unwrap_or(origin);
        let id = sim.world.next_creature_id;
        sim.world.next_creature_id += 1;
        sim.world.creatures.push(Creature {
            id,
            kind,
            tile,
            position: Point::of_tile(tile),
            path: VecDeque::new(),
            move_budget: 0.0,
            health: balance.health,
            mood: CreatureMood::Approaching,
            target: None,
            ticks_alive: 0,
            stolen: 0.0,
        });
    }
}

/// What a creature wants: crops and stores, or people if it is a hunter.
fn choose_target(sim: &Sim, creature: &Creature, aggressive: bool) -> Option<Tile> {
    if aggressive {
        let prey = sim.present().filter(|a| a.inside.is_none()).min_by(|a, b| {
            a.tile
                .distance(creature.tile)
                .total_cmp(&b.tile.distance(creature.tile))
        });
        if let Some(prey) = prey.filter(|p| p.tile.distance(creature.tile) < 18.0) {
            return Some(prey.tile);
        }
    }
    sim.world
        .built()
        .filter(|s| s.crop.is_some() || s.def().storage > 0.0)
        .min_by(|a, b| {
            a.footprint
                .distance_to(creature.tile)
                .total_cmp(&b.footprint.distance_to(creature.tile))
        })
        .and_then(|s| sim.world.access_tile(s.id, creature.tile))
}

fn defence_damage(sim: &Sim, tile: Tile) -> f32 {
    let base = game_data().balance.colony.defence_damage_per_tick * sim.colony.modifiers.defence;
    sim.world
        .built()
        .filter(|s| s.def().defence > 0.0 && s.is_operational())
        .filter(|s| s.footprint.distance_to(tile) <= s.def().defence_range)
        .map(|s| s.def().defence * base)
        .sum()
}

/// Steal from stores or crops next to the creature.
fn raid(sim: &mut Sim, index: usize, amount: f32) {
    let tile = sim.world.creatures[index].tile;
    let near_crop = sim
        .world
        .structures
        .iter_mut()
        .filter(|s| s.footprint.distance_to(tile) <= 1.5)
        .find_map(|s| s.crop.as_mut());
    if let Some(crop) = near_crop {
        crop.growth = (crop.growth - 0.3).max(0.0);
    }
    let stolen = sim.colony.stock.take(Resource::Food, amount);
    sim.world.creatures[index].stolen += stolen;
}

fn bite(sim: &mut Sim, tile: Tile, damage: f32) {
    let ticks = Calendar::ticks_per_day();
    let mut victims = Vec::new();
    for agent in sim
        .agents
        .iter_mut()
        .filter(|a| a.is_present() && a.inside.is_none())
    {
        if agent.tile.steps(tile) <= 1 {
            agent.health.damage(damage);
            agent.needs.change(Need::Safety, -10.0);
            if !agent.mind.has("injured") {
                agent.mind.add("injured", None, ticks);
                agent.health.injuries += 1;
                victims.push((agent.id, agent.given_name.clone()));
            }
        } else if agent.tile.distance(tile) < 6.0 && !agent.mind.has("witnessed_attack") {
            agent.mind.add("witnessed_attack", None, ticks);
        }
    }
    for (id, name) in victims {
        let text = line("attacked", sim.calendar.tick, &[("name", &name)]);
        sim.record(Category::Danger, 1, text, vec![id]);
    }
}

fn step_creature(sim: &mut Sim, index: usize) {
    let creature = sim.world.creatures[index].clone();
    let Some(balance) = game_data().balance.creatures.get(&creature.kind) else {
        return;
    };
    let fleeing = creature.mood == CreatureMood::Fleeing;
    if !fleeing
        && (creature.health < balance.health * 0.4 || creature.ticks_alive > balance.linger_ticks)
    {
        sim.world.creatures[index].mood = CreatureMood::Fleeing;
        let exit = edge_tile(sim);
        sim.world.creatures[index].target = Some(exit);
        sim.world.creatures[index].path.clear();
    } else if !fleeing && creature.ticks_alive.is_multiple_of(20) {
        sim.world.creatures[index].target = choose_target(sim, &creature, balance.aggressive);
        sim.world.creatures[index].path.clear();
    }
    let damage = defence_damage(sim, creature.tile);
    let target = sim.world.creatures[index].target;
    if let Some(target) = target {
        if target.steps(creature.tile) <= 1 && !fleeing {
            sim.world.creatures[index].mood = CreatureMood::Raiding;
            if creature.ticks_alive.is_multiple_of(10) {
                raid(sim, index, balance.raid_amount / 6.0);
            }
            if balance.aggressive {
                bite(sim, creature.tile, balance.damage_per_tick);
            }
        } else if sim.world.creatures[index].path.is_empty() {
            if let Some(path) = sim.world.find_path(creature.tile, target) {
                sim.world.creatures[index].path = path;
            }
        }
    }
    let world = &mut sim.world;
    let entry = &mut world.creatures[index];
    entry.health -= damage;
    entry.ticks_alive += 1;
    entry.move_budget += balance.tiles_per_tick * if fleeing { 1.3 } else { 1.0 };
    while entry.move_budget >= 1.0 {
        let Some(next) = entry.path.pop_front() else {
            break;
        };
        entry.tile = next;
        entry.move_budget -= 1.0;
    }
    entry.position = match entry.path.front() {
        Some(next) => {
            Point::of_tile(entry.tile).lerp(Point::of_tile(*next), entry.move_budget.min(1.0))
        }
        None => Point::of_tile(entry.tile),
    };
}

pub fn update(sim: &mut Sim) {
    for index in 0..sim.world.creatures.len() {
        step_creature(sim, index);
    }
    let before = sim.world.creatures.len();
    let map_edge = |tile: Tile, width: i32, height: i32| {
        tile.x <= 0 || tile.y <= 0 || tile.x >= width - 1 || tile.y >= height - 1
    };
    let (width, height) = (sim.world.map.width, sim.world.map.height);
    sim.world.creatures.retain(|c| {
        let left = c.mood == CreatureMood::Fleeing
            && map_edge(c.tile, width, height)
            && c.ticks_alive > 30;
        c.health > 0.0 && !left
    });
    let removed = before - sim.world.creatures.len();
    if removed > 0 {
        sim.colony.stats.creatures_repelled += removed as u32;
    }
}
