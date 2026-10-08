//! overlay domain.

use super::*;
use crate::ui::{result_back_rect, result_menu_rect, result_review_rect};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl GameplayState {
    pub fn draw_scenario_overlay(&self) {
        if !self.data.scenario.is_finished() {
            return;
        }

        if self.result_review_open {
            let banner = Rect::new(18.0, 16.0, screen_width() - 36.0, 44.0);
            style::draw_deep_panel(banner);
            draw_ui_text(
                &format!("{} — outcome review", self.data.scenario.outcome.label()),
                banner.x + 14.0,
                banner.y + 28.0,
                16.0,
                WHITE,
            );
            let back = result_back_rect(screen_width(), screen_height());
            style::draw_button(back, false, style::button_hovered(back));
            draw_ui_text("BACK TO OUTCOME", back.x + 13.0, back.y + 25.0, 12.0, WHITE);
            return;
        }

        let w = 520.0;
        let h = 302.0;
        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.08, 0.1, 0.95));
        draw_rectangle_lines(x, y, w, h, 2.0, WHITE);

        let title = self.data.scenario.outcome.label();
        let title_width = measure_ui_text(title, None, 28, 1.0).width;
        draw_ui_text(title, x + (w - title_width) * 0.5, y + 42.0, 28.0, WHITE);

        let line = ScenarioSystem::objective_line(&self.data);
        let line_width = measure_ui_text(&line, None, 16, 1.0).width;
        draw_ui_text(&line, x + (w - line_width) * 0.5, y + 82.0, 16.0, LIGHTGRAY);

        let prompt = "Review the colony story, restart the plan, or return to the menu.";
        let prompt_width = measure_ui_text(prompt, None, 14, 1.0).width;
        draw_ui_text(prompt, x + (w - prompt_width) * 0.5, y + 116.0, 14.0, GRAY);

        let buttons = [
            (
                result_review_rect(screen_width(), screen_height()),
                "Review Log",
            ),
            (
                restart_button_rect(screen_width(), screen_height()),
                "Restart Run",
            ),
            (
                result_menu_rect(screen_width(), screen_height()),
                "Return to Menu",
            ),
        ];
        for (button, label) in buttons {
            style::draw_button(button, false, style::button_hovered(button));
            let button_width = measure_ui_text(label, None, 16, 1.0).width;
            draw_ui_text(
                label,
                button.x + (button.w - button_width) * 0.5,
                button.y + 25.0,
                16.0,
                WHITE,
            );
        }
        draw_ui_text(
            "R or Enter restarts",
            x + 20.0,
            y + h - 15.0,
            12.0,
            LIGHTGRAY,
        );
    }
}
