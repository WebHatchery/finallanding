//! game domain.

pub mod building_system;
pub mod colonist_ai;
pub mod colonist_spawner;

use crate::state::menu_state::MenuState;

use crate::data::building::BuildingType;
use crate::data::types::Position;
use crate::state::game_state::GameplayState;
use crate::state::{State, StateTransition};
use macroquad::prelude::{request_new_screen_size, set_fullscreen};

pub enum GameStateEnum {
    Gameplay(Box<GameplayState>),
    Menu(MenuState),
}

pub struct Game {
    state: GameStateEnum,
}

impl Game {
    pub async fn new() -> Self {
        let state = if should_start_gameplay() {
            GameStateEnum::Gameplay(Box::new(GameplayState::new_for_capture()))
        } else {
            GameStateEnum::Menu(MenuState::new())
        };

        Self { state }
    }

    pub fn update(&mut self) {
        match &mut self.state {
            GameStateEnum::Gameplay(state) => {
                let transition = state.update();
                match transition {
                    StateTransition::ToGameplay(new_state) => {
                        self.state = GameStateEnum::Gameplay(new_state);
                    }
                    StateTransition::ToMenu { status_message } => {
                        self.state = GameStateEnum::Menu(MenuState::with_status(status_message));
                    }
                    StateTransition::None => {}
                }
            }
            GameStateEnum::Menu(state) => {
                let transition = state.update_with_input();
                match transition {
                    StateTransition::ToGameplay(new_state) => {
                        self.state = GameStateEnum::Gameplay(new_state);
                    }
                    StateTransition::ToMenu { .. } | StateTransition::None => {}
                }
            }
        }
    }

    pub fn draw(&self) {
        match &self.state {
            GameStateEnum::Gameplay(state) => state.draw(),
            GameStateEnum::Menu(state) => state.draw_ui(),
        }
    }

    pub fn begin_capture_scene(&mut self, scene: &str) {
        const KEYS: &[&str] = &[
            "TFL_START_TOOLBAR_MODE",
            "TFL_START_SELECTED_COLONIST",
            "TFL_START_SOCIAL_HISTORY_DAY",
            "TFL_START_SELECTED_BUILDING",
            "TFL_PREVIEW_GRID_X",
            "TFL_PREVIEW_GRID_Y",
            "TFL_SEED_SOCIAL_HISTORY",
            "TFL_SEED_ACTIVITY_POSES",
            "TFL_SEED_ASSIGN_SPACES",
        ];
        for key in KEYS {
            std::env::remove_var(key);
        }

        if let Some((width, height)) = capture_menu_size(scene) {
            set_fullscreen(false);
            request_new_screen_size(width as f32, height as f32);
            self.state = GameStateEnum::Menu(MenuState::new());
            return;
        }

        if let Some((width, height, reviewed)) = capture_result_size(scene) {
            set_fullscreen(false);
            request_new_screen_size(width as f32, height as f32);
            let mut gameplay = GameplayState::new_for_capture();
            gameplay.data.scenario.outcome = crate::data::scenario::ScenarioOutcome::Victory;
            gameplay.data.scenario.outcome_tick = Some(gameplay.data.tick);
            gameplay.result_review_open = reviewed;
            if reviewed {
                gameplay.toolbar_mode = crate::ui::ToolbarMode::Log;
            }
            self.state = GameStateEnum::Gameplay(Box::new(gameplay));
            return;
        }

        let (width, height, fullscreen, values) = match scene {
            "smoke_1920x1080" => (1920, 1080, true, vec![("TFL_START_TOOLBAR_MODE", "build")]),
            "smoke_closed_1280x720" => {
                (1280, 720, false, vec![("TFL_START_TOOLBAR_MODE", "build")])
            }
            "smoke_assign_1280x720" => (
                1280,
                720,
                false,
                vec![
                    ("TFL_START_TOOLBAR_MODE", "assign"),
                    ("TFL_START_SELECTED_COLONIST", "5"),
                    ("TFL_SEED_ASSIGN_SPACES", "1"),
                ],
            ),
            "smoke_log_1280x720" => (
                1280,
                720,
                false,
                vec![
                    ("TFL_START_TOOLBAR_MODE", "log"),
                    ("TFL_SEED_SOCIAL_HISTORY", "1"),
                    ("TFL_START_SOCIAL_HISTORY_DAY", "4"),
                ],
            ),
            "smoke_log_timeline_1280x720" => (
                1280,
                720,
                false,
                vec![
                    ("TFL_START_TOOLBAR_MODE", "log"),
                    ("TFL_SEED_SOCIAL_HISTORY", "1"),
                ],
            ),
            "smoke_research_1280x720" => (
                1280,
                720,
                false,
                vec![("TFL_START_TOOLBAR_MODE", "research")],
            ),
            "smoke_research_ready_1280x720" => (
                1280,
                720,
                false,
                vec![("TFL_START_TOOLBAR_MODE", "research")],
            ),
            "smoke_research_touch_720x480" => (
                720,
                480,
                false,
                vec![("TFL_START_TOOLBAR_MODE", "research")],
            ),
            "smoke_colony_1280x720" => {
                (1280, 720, false, vec![("TFL_START_TOOLBAR_MODE", "colony")])
            }
            "smoke_placement_1280x720" => (
                1280,
                720,
                false,
                vec![
                    ("TFL_START_TOOLBAR_MODE", "build"),
                    ("TFL_START_SELECTED_BUILDING", "habitat"),
                    ("TFL_PREVIEW_GRID_X", "5"),
                    ("TFL_PREVIEW_GRID_Y", "9"),
                ],
            ),
            "smoke_poses_1280x720" => (
                1280,
                720,
                false,
                vec![
                    ("TFL_START_TOOLBAR_MODE", "build"),
                    ("TFL_SEED_ACTIVITY_POSES", "1"),
                ],
            ),
            "smoke_touch_720x480" => (
                720,
                480,
                false,
                vec![
                    ("TFL_START_TOOLBAR_MODE", "assign"),
                    ("TFL_START_SELECTED_COLONIST", "0"),
                    ("TFL_SEED_ASSIGN_SPACES", "1"),
                ],
            ),
            _ => (1280, 720, false, vec![("TFL_START_TOOLBAR_MODE", "build")]),
        };
        for (key, value) in values {
            std::env::set_var(key, value);
        }
        set_fullscreen(fullscreen);
        request_new_screen_size(width as f32, height as f32);
        let mut gameplay = GameplayState::new_for_capture();
        if scene == "smoke_closed_1280x720" {
            gameplay.context_panel_open = false;
        }
        if scene == "smoke_research_ready_1280x720" {
            let data = &mut gameplay.data;
            let crate::state::runtime_state::GameState {
                data: colony_data,
                building_system,
                ..
            } = data;
            let crate::data::game_state::ColonyData { grid, .. } = colony_data;
            let _ = building_system.try_place_building(
                grid,
                BuildingType::ExplorationGate,
                Position::new(1, 1),
            );
        }
        self.state = GameStateEnum::Gameplay(Box::new(gameplay));
    }
}

fn should_start_gameplay() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("TFL_START_GAMEPLAY").is_ok_and(|value| value != "0")
    }

    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

fn capture_menu_size(scene: &str) -> Option<(i32, i32)> {
    match scene {
        "menu_1280x720" => Some((1280, 720)),
        "menu_720x480" => Some((720, 480)),
        _ => None,
    }
}

fn capture_result_size(scene: &str) -> Option<(i32, i32, bool)> {
    match scene {
        "result_1280x720" => Some((1280, 720, false)),
        "result_review_1280x720" => Some((1280, 720, true)),
        "result_720x480" => Some((720, 480, false)),
        _ => None,
    }
}
