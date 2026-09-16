//! Legacy regression suites migrated from production modules.
//!
//! Each suite imports the public feature seam it exercises. The source files
//! remain split by domain so the migration preserves the original coverage
//! without bringing test-only module declarations back into `src/`.

pub use finallanding::prelude::*;
pub use finallanding::ui::style;
pub use macroquad::prelude::{vec2, Rect};

mod data_assign_roster {
    pub use finallanding::data::assign_roster::*;
    include!("src/data/assign_roster/tests.rs");
}
mod data_building {
    include!("src/data/building/tests.rs");
}
mod data_colonist {
    include!("src/data/colonist/tests.rs");
}
mod data_grid {
    include!("src/data/grid/tests.rs");
}
mod data_simulation_rng {
    include!("src/data/simulation_rng/tests.rs");
}
mod data_technology {
    include!("src/data/technology/tests.rs");
}

mod game_building_system {
    include!("src/game/building_system/tests.rs");
}
mod game_colonist_ai {
    include!("src/game/colonist_ai/tests.rs");
    pub mod assignment {
        include!("src/game/colonist_ai/assignment/tests.rs");
    }
    pub mod movement {
        include!("src/game/colonist_ai/movement/tests.rs");
    }
    pub mod targeting {
        include!("src/game/colonist_ai/targeting/tests.rs");
    }
}
mod game_colonist_spawner {
    include!("src/game/colonist_spawner/tests.rs");
}

mod state_assignment_batch_rules {
    use finallanding::state::game_state::game_state_assignment_batch_rules::*;
    include!("src/state/game_state_assignment_batch_rules/tests.rs");
}
mod state_assignment_roster {
    use finallanding::state::game_state::game_state_assignment_roster::*;
    include!("src/state/game_state_assignment_roster/tests.rs");
}
mod state_assignment_space_rules {
    use finallanding::state::game_state::game_state_assignment_space_rules::*;
    include!("src/state/game_state_assignment_space_rules/tests.rs");
}
mod state_relationship_contact {
    use finallanding::state::game_state::game_state_relationship_contact::*;
    include!("src/state/game_state_relationship_contact/tests.rs");
}
mod state_setup {
    use finallanding::state::game_state::game_state_setup::*;
    include!("src/state/game_state_setup/tests.rs");
}
mod state_social_archive {
    use finallanding::state::game_state::game_state_social_archive::*;
    include!("src/state/game_state_social_archive/tests.rs");
}
mod state_persistence {
    include!("src/state/persistence/tests.rs");
}

mod systems_advisor {
    include!("src/systems/advisor_system/tests.rs");
}
mod systems_assignment {
    include!("src/systems/assignment_system/tests.rs");
}
mod systems_incident {
    include!("src/systems/incident_system/tests.rs");
}
mod systems_mission {
    pub mod launch {
        pub use finallanding::prelude::*;
        include!("src/systems/mission_system/launch/tests.rs");
    }
    pub mod planning {
        pub use finallanding::prelude::*;
        include!("src/systems/mission_system/planning/tests.rs");
    }
    pub mod resolution {
        pub use finallanding::prelude::*;
        include!("src/systems/mission_system/resolution/tests.rs");
    }
}
mod systems_objective {
    include!("src/systems/objective_system/tests.rs");
}
mod systems_planning {
    include!("src/systems/planning_system/tests.rs");
}
mod systems_playtest {
    include!("src/systems/playtest_system/tests.rs");
}
mod systems_relationship_directive {
    include!("src/systems/relationship_directive_system/tests.rs");
}
mod systems_resource {
    include!("src/systems/resource_system/tests.rs");
}
mod systems_scenario {
    include!("src/systems/scenario_system/tests.rs");
}
mod systems_social {
    include!("src/systems/social_system/tests.rs");
}
mod systems_summary {
    include!("src/systems/summary_system/tests.rs");
}
mod systems_time {
    include!("src/systems/time_system/tests.rs");
}
mod systems_work {
    include!("src/systems/work_system/tests.rs");
}

mod ui_art_portrait {
    pub use finallanding::ui::art::portrait::*;
    pub use finallanding::ui::art::profiles::*;
    include!("src/ui/art/portrait/tests.rs");
}
mod ui_art_profiles {
    pub use finallanding::ui::art::profiles::*;
    include!("src/ui/art/profiles/tests.rs");
}
mod ui_art_sprite {
    pub use finallanding::ui::art::profiles::*;
    pub use finallanding::ui::art::sprite::*;
    include!("src/ui/art/sprite/tests.rs");
}
mod ui_font {
    include!("src/ui/font/tests.rs");
}
mod ui_gameplay_building_visuals {
    pub use finallanding::ui::gameplay::building_visuals::*;
    include!("src/ui/gameplay/building_visuals/tests.rs");
}
mod ui_gameplay_colonist_visuals {
    pub use finallanding::ui::gameplay::colonist_visuals::*;
    include!("src/ui/gameplay/colonist_visuals/tests.rs");
}
mod ui_gameplay_terrain_visuals {
    pub use finallanding::ui::gameplay::terrain_visuals::*;
    include!("src/ui/gameplay/terrain_visuals/tests.rs");
}
mod ui_gameplay_wreckage_visuals {
    pub use finallanding::ui::gameplay::wreckage_visuals::*;
    include!("src/ui/gameplay/wreckage_visuals/tests.rs");
}
mod ui_hit_zones_assign {
    pub use finallanding::ui::hit_zones::assign::*;
    use macroquad::prelude::Rect;
    include!("src/ui/hit_zones/assign/tests.rs");
}
mod ui_hit_zones_log {
    pub use finallanding::ui::hit_zones::log::*;
    use macroquad::prelude::Rect;
    include!("src/ui/hit_zones/log/tests.rs");
}
mod ui_hit_zones_menu {
    pub use finallanding::ui::hit_zones::menu::*;
    include!("src/ui/hit_zones/menu/tests.rs");
}
mod ui_hit_zones_toolbar {
    pub use finallanding::ui::hit_zones::toolbar::*;
    use macroquad::prelude::Rect;
    include!("src/ui/hit_zones/toolbar/tests.rs");
}
mod ui_hit_zones_top_bar {
    pub use finallanding::ui::hit_zones::top_bar::*;
    use macroquad::prelude::Rect;
    include!("src/ui/hit_zones/top_bar/tests.rs");
}
mod ui_isometric {
    pub use finallanding::ui::isometric::*;
    use macroquad::prelude::Rect;
    include!("src/ui/isometric/tests.rs");
}
mod ui_right_rail {
    pub use finallanding::ui::right_rail::*;
    include!("src/ui/right_rail/tests.rs");
}
mod ui_toolbar_assign_model {
    use finallanding::ui::toolbar_panel::assign_model::*;
    include!("src/ui/toolbar_panel/assign_model/tests.rs");
}
mod ui_toolbar_log_model {
    pub use finallanding::ui::toolbar_panel::log_model::*;
    include!("src/ui/toolbar_panel/log_model/tests.rs");
}
mod ui_tooltip {
    pub use finallanding::ui::tooltip::*;
    use macroquad::prelude::{vec2, Rect};
    include!("src/ui/tooltip/tests.rs");
}
