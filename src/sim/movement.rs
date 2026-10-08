//! Walking along planned paths, with smooth positions for rendering.

use super::Ctx;
use crate::agents::Agent;
use crate::data::game_data;
use crate::world::Point;

pub fn advance(agent: &mut Agent, ctx: &mut Ctx) {
    if agent.path.is_empty() {
        agent.move_budget = 0.0;
        settle_position(agent, ctx);
        return;
    }
    let balance = game_data().balance.agents.tiles_per_tick;
    let indoors = agent.inside.is_some();
    let weather = if ctx.world.weather.is_storm() && !indoors {
        game_data()
            .balance
            .work
            .severe_weather_outdoor_factor
            .max(0.6)
    } else {
        1.0
    };
    let child = if agent.is_child { 0.8 } else { 1.0 };
    let terrain = ctx.world.map.move_cost(agent.tile);
    agent.move_budget += balance * weather * child * agent.health.capacity().max(0.5) / terrain;
    while agent.move_budget >= 1.0 {
        let Some(next) = agent.path.front().copied() else {
            break;
        };
        if !ctx.world.map.is_walkable(next) {
            // A new building or fallen rock blocks the way: replan next tick.
            agent.path.clear();
            break;
        }
        agent.path.pop_front();
        agent.tile = next;
        agent.move_budget -= 1.0;
    }
    let target = agent.path.front().copied().map(Point::of_tile);
    let here = Point::of_tile(agent.tile);
    agent.position = match target {
        Some(next) => here.lerp(next, agent.move_budget.clamp(0.0, 1.0)),
        None => here,
    };
}

/// Survivors inside a building are drawn at its centre; others on their tile.
fn settle_position(agent: &mut Agent, ctx: &Ctx) {
    agent.position = match agent.inside.and_then(|id| ctx.world.structure(id)) {
        Some(structure) => {
            let center = structure.footprint.center();
            let jitter = (agent.id % 5) as f32 * 0.25 - 0.5;
            Point {
                x: center.x + jitter,
                y: center.y + jitter * 0.5,
            }
        }
        None => Point::of_tile(agent.tile),
    };
}
