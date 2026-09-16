//! Public game library shared by the native launcher and integration tests.
//!
//! The binary is intentionally a thin event-loop shell. Keeping the game
//! domains here makes the simulation testable through deliberate public seams
//! and prevents test-only compilation from changing the production module
//! graph.

pub mod data;
pub mod game;
pub mod state;
pub mod systems;
pub mod ui;

/// Deliberate public seams used by integration tests and small tooling.
///
/// The game remains organized by domain; this module only gathers the stable
/// value types and pure feature helpers that callers need to exercise rules.
pub mod prelude {
    pub use crate::data::assign_roster::{AssignRosterFilter, AssignRosterSort};
    pub use crate::data::building::*;
    pub use crate::data::colonist::*;
    pub use crate::data::event_log::*;
    pub use crate::data::game_state::{ColonyData, TimeSpeed, TimeState};
    pub use crate::data::grid::*;
    pub use crate::data::incident::*;
    pub use crate::data::mission::*;
    pub use crate::data::priority::*;
    pub use crate::data::resources::*;
    pub use crate::data::scenario::*;
    pub use crate::data::schedule::*;
    pub use crate::data::simulation_rng::*;
    pub use crate::data::technology::*;
    pub use crate::data::types::*;
    pub use crate::game::building_system::*;
    pub use crate::game::colonist_ai::assignment::*;
    pub use crate::game::colonist_ai::movement::*;
    pub use crate::game::colonist_ai::targeting::*;
    pub use crate::game::colonist_ai::*;
    pub use crate::game::colonist_spawner::*;
    pub use crate::state::game_state::*;
    pub use crate::state::runtime_state::GameState;
    pub use crate::systems::advisor_system::*;
    pub use crate::systems::assignment_system::*;
    pub use crate::systems::incident_system::*;
    pub use crate::systems::mission_system::launch::*;
    pub use crate::systems::mission_system::planning::*;
    pub use crate::systems::mission_system::resolution::*;
    pub use crate::systems::mission_system::*;
    pub use crate::systems::objective_system::*;
    pub use crate::systems::planning_system::*;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::systems::playtest_report::*;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::systems::playtest_strategy::*;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::systems::playtest_system::*;
    pub use crate::systems::relationship_directive_system::*;
    pub use crate::systems::resource_system::*;
    pub use crate::systems::scenario_system::*;
    pub use crate::systems::social_system::*;
    pub use crate::systems::summary_system::*;
    pub use crate::systems::time_system::*;
    pub use crate::systems::work_system::*;
    pub use crate::ui::art::SpritePose;
    pub use crate::ui::hit_zones::{PageAction, *};
    pub use std::collections::HashMap;
}
