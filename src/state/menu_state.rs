//! menu state domain.

use crate::state::persistence::{has_saved_game, load_game};
use crate::state::{State, StateTransition};
use crate::ui::{menu_continue_rect, menu_exit_rect, menu_settings_rect, menu_start_rect, style};
use macroquad::prelude::*;
use macroquad_toolkit::input::InputState;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, Pointer};

#[derive(Default)]
pub struct MenuState {
    pub settings_open: bool,
    pub status_message: Option<String>,
}

impl MenuState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl State for MenuState {
    fn update(&mut self) -> StateTransition {
        StateTransition::None
    }

    fn draw(&self) {
        self.draw_ui();
    }
}

// Re-implementing correctly to separate logic for the immutable draw trait constraint
impl MenuState {
    pub fn update_with_input(&mut self) -> StateTransition {
        let mut input = InputState::capture();
        let pointer = Pointer::read(|position| position);
        if pointer.released {
            input.mouse_pos = pointer.position;
            input.left_released = true;
        }
        if input.space_pressed || input.enter_pressed {
            return StateTransition::ToGameplay(Box::default());
        }

        let btn_rect = menu_start_rect(screen_width(), screen_height());
        if input.left_released_rect(btn_rect) {
            return StateTransition::ToGameplay(Box::default());
        }

        if input.left_released_rect(menu_continue_rect(screen_width(), screen_height())) {
            return match load_game() {
                Ok(game) => StateTransition::ToGameplay(Box::new(
                    crate::state::game_state::GameplayState::from_saved(game),
                )),
                Err(error) => {
                    self.status_message = Some(format!("Continue unavailable: {}", error));
                    StateTransition::None
                }
            };
        }

        if input.left_released_rect(menu_settings_rect(screen_width(), screen_height())) {
            self.settings_open = !self.settings_open;
            return StateTransition::None;
        }

        if input.left_released_rect(menu_exit_rect(screen_width(), screen_height())) {
            #[cfg(not(target_arch = "wasm32"))]
            std::process::exit(0);
            #[cfg(target_arch = "wasm32")]
            {
                self.status_message = Some("Close this browser tab to exit.".to_string());
            }
        }

        StateTransition::None
    }

    pub fn draw_ui(&self) {
        clear_background(macroquad_toolkit::colors::dark::BACKGROUND);

        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;
        let text = crate::data::config::game_config().text.clone();

        // Title
        let title = text.menu_title.as_str();
        let title_dim = measure_ui_text(title, None, 50, 1.0);
        draw_ui_text(
            title,
            screen_center_x - title_dim.width / 2.0,
            screen_center_y - 170.0,
            50.0,
            WHITE,
        );

        for (index, line) in text.menu_premise.iter().enumerate() {
            let dim = measure_ui_text(line, None, 20, 1.0);
            draw_ui_text(
                line,
                screen_center_x - dim.width / 2.0,
                screen_center_y - 112.0 + index as f32 * 28.0,
                20.0,
                LIGHTGRAY,
            );
        }

        let buttons = [
            (
                menu_start_rect(screen_width(), screen_height()),
                text.start_game.as_str(),
                true,
            ),
            (
                menu_continue_rect(screen_width(), screen_height()),
                text.continue_game.as_str(),
                has_saved_game(),
            ),
            (
                menu_settings_rect(screen_width(), screen_height()),
                text.settings.as_str(),
                true,
            ),
            (
                menu_exit_rect(screen_width(), screen_height()),
                text.exit.as_str(),
                true,
            ),
        ];
        for (rect, label, enabled) in buttons {
            style::draw_button(rect, false, enabled && style::button_hovered(rect));
            let color = if enabled { WHITE } else { style::TEXT_MUTED };
            let dim = measure_ui_text(label, None, 20, 1.0);
            draw_ui_text(
                label,
                rect.x + (rect.w - dim.width) / 2.0,
                rect.y + 29.0,
                20.0,
                color,
            );
        }

        // Instructions
        for (index, line) in text.menu_controls.iter().enumerate() {
            let dim = measure_ui_text(line, None, 16, 1.0);
            draw_ui_text(
                line,
                screen_center_x - dim.width / 2.0,
                screen_center_y + 176.0 + index as f32 * 22.0,
                16.0,
                GRAY,
            );
        }

        if let Some(message) = &self.status_message {
            let dim = measure_ui_text(message, None, 14, 1.0);
            draw_ui_text(
                message,
                screen_center_x - dim.width / 2.0,
                screen_center_y + 244.0,
                14.0,
                style::ALERT_RED,
            );
        }

        if self.settings_open {
            let modal = Rect::new(
                screen_center_x - 190.0,
                screen_center_y - 50.0,
                380.0,
                150.0,
            );
            style::draw_deep_panel(modal);
            draw_ui_text(
                "SETTINGS",
                modal.x + 18.0,
                modal.y + 28.0,
                18.0,
                style::TEXT_PRIMARY,
            );
            draw_ui_text(
                "UI scale follows the viewport automatically.",
                modal.x + 18.0,
                modal.y + 62.0,
                14.0,
                style::TEXT_BODY,
            );
            draw_ui_text(
                "Tap Settings again to close this panel.",
                modal.x + 18.0,
                modal.y + 88.0,
                14.0,
                style::TEXT_BODY,
            );
        }
    }
}
