//! colonist ai domain.

pub mod assignment;
mod behavior;
pub mod movement;
mod social;
pub mod targeting;
mod types;

use crate::data::types::Position;
use crate::game::colonist_ai::types::{BuildingSnapshot, PendingLog, SocialLocation};
use crate::state::runtime_state::GameState;
use crate::systems::mood_system::update_mood;
use crate::systems::time_system::TimeSystem;
use std::collections::HashMap;

/// Movement speed for visual interpolation (pixels per frame)
const VISUAL_MOVE_SPEED: f32 = 2.0;
pub const REFUSAL_LOG_COOLDOWN_TICKS: u64 = 60;
pub const SOCIAL_STRAIN_LOG_COOLDOWN_TICKS: u64 = 120;

pub fn update_colonists(state: &mut GameState, elapsed_ticks: u64) {
    if elapsed_ticks == 0 {
        for colonist in &mut state.data.colonists {
            colonist.update_visual_position(VISUAL_MOVE_SPEED);
        }
        return;
    }

    let tick = state.data.tick;
    let (_, hour, _) = TimeSystem::get_time_of_day(tick);

    let occupied: HashMap<Position, u32> = state
        .data
        .colonists
        .iter()
        .filter(|c| !c.is_on_mission())
        .map(|c| (c.position, c.id))
        .collect();
    let colonist_names: HashMap<u32, String> = state
        .data
        .colonists
        .iter()
        .map(|colonist| (colonist.id, colonist.name.clone()))
        .collect();
    let social_locations: Vec<SocialLocation> = state
        .data
        .colonists
        .iter()
        .map(|colonist| (colonist.id, colonist.activity_location.clone()))
        .collect();

    let mut building_occupancy: HashMap<u32, u32> = HashMap::new();
    for c in &state.data.colonists {
        if c.is_on_mission() {
            continue;
        }

        if let Some(bid) = c.assigned_habitat {
            *building_occupancy.entry(bid).or_default() += 1;
        }
    }

    let buildings: Vec<BuildingSnapshot> = state
        .building_system
        .buildings()
        .iter()
        .map(|b| (b.id, b.building_type, b.position, b.size()))
        .collect();
    let habitat_capacity = 2 + state.data.technology.habitat_capacity_bonus();
    let priority = state.data.priority.active;

    let mut pending_logs: Vec<PendingLog> = Vec::new();

    for i in 0..state.data.colonists.len() {
        let scheduled_activity = state.data.colonists[i].schedule.get_activity_for_hour(hour);

        let mut context = behavior::ColonistAiContext {
            scheduled_activity: &scheduled_activity,
            occupied: &occupied,
            colonist_names: &colonist_names,
            social_locations: &social_locations,
            grid: &state.data.grid,
            rng: &mut state.rng,
            buildings: &buildings,
            building_occupancy: &mut building_occupancy,
            habitat_capacity,
            current_tick: tick,
            pending_logs: &mut pending_logs,
        };
        behavior::update_colonist_ai(&mut state.data.colonists[i], &mut context);

        state.data.colonists[i].update_visual_position(VISUAL_MOVE_SPEED);
        update_mood(&mut state.data.colonists[i], elapsed_ticks, priority);
    }

    for (category, title, detail) in pending_logs {
        state.push_log(category, title, detail);
    }
}
