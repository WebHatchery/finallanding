//! Runtime-owned state that composes reusable colony data with orchestration services.

use crate::data::event_log::{ColonyLogEntry, LogCategory, SocialHistoryEntry};
use crate::data::game_state::{ColonyData, TimeSpeed};
use crate::data::simulation_rng::SimulationRng;
use crate::game::building_system::BuildingSystem;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// The mutable simulation state owned by the state layer.
///
/// `ColonyData` stays independent of placement and random-number services;
/// this wrapper is the one deliberate place where runtime orchestration is
/// composed with reusable data. `Deref` preserves the concise `state.tick`
/// access used by systems while keeping ownership explicit.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub data: ColonyData,
    pub building_system: BuildingSystem,
    pub rng: SimulationRng,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            data: ColonyData::new(),
            building_system: BuildingSystem::new(),
            rng: SimulationRng::default(),
        }
    }

    pub fn push_log(
        &mut self,
        category: LogCategory,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.data.push_log(category, title, detail);
    }

    pub fn push_social_history(&mut self, entry: SocialHistoryEntry) {
        self.data.push_social_history(entry);
    }

    pub fn time_speed(&self) -> TimeSpeed {
        self.time.speed
    }

    pub fn event_log(&self) -> &[ColonyLogEntry] {
        &self.event_log
    }
}

impl Deref for GameState {
    type Target = ColonyData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for GameState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
