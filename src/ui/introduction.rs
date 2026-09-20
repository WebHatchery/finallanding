//! First-run arrival briefing shown before the survivor roster becomes active.

use crate::ui::hit_zones::introduction_continue_rect;
use crate::ui::hit_zones::{help_close_rect, help_panel_rect};
use crate::ui::style;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub const ARRIVAL_STAGE_COUNT: usize = 4;

pub fn draw_arrival_overlay(stage: usize) {
    let width = screen_width();
    let height = screen_height();
    let panel_width = (width - 24.0).clamp(300.0, 620.0);
    let panel_height = 270.0;
    let panel = Rect::new(
        (width - panel_width) * 0.5,
        (height - panel_height) * 0.5 - 24.0,
        panel_width,
        panel_height,
    );

    draw_rectangle(0.0, 0.0, width, height, Color::new(0.01, 0.02, 0.025, 0.68));
    style::draw_deep_panel(panel);
    draw_ui_text(
        "ARRIVAL BRIEFING",
        panel.x + 18.0,
        panel.y + 31.0,
        style::TITLE_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        &format!(
            "STEP {} OF {}",
            stage.min(ARRIVAL_STAGE_COUNT - 1) + 1,
            ARRIVAL_STAGE_COUNT
        ),
        panel.x + panel.w - 96.0,
        panel.y + 29.0,
        style::TINY_SIZE,
        style::HEADING_BLUE,
    );

    let (heading, body) = briefing_copy(stage);
    draw_ui_text(
        heading,
        panel.x + 18.0,
        panel.y + 70.0,
        style::SECTION_SIZE,
        style::ACCENT_GOLD,
    );
    draw_wrapped_text(
        &body,
        panel.x + 18.0,
        panel.y + 99.0,
        panel.w - 36.0,
        style::BODY_SIZE,
        style::TEXT_BODY,
    );

    let button = introduction_continue_rect(width, height);
    style::draw_button(button, false, style::button_hovered(button));
    let label = if stage + 1 >= ARRIVAL_STAGE_COUNT {
        "ENTER COLONY"
    } else {
        "CONTINUE"
    };
    let label_width = measure_ui_text(label, None, style::BODY_SIZE as u16, 1.0).width;
    draw_ui_text(
        label,
        button.x + (button.w - label_width) * 0.5,
        button.y + 30.0,
        style::BODY_SIZE,
        style::TEXT_PRIMARY,
    );
}

pub fn draw_help_overlay() {
    let panel = help_panel_rect(screen_width(), screen_height());
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.01, 0.02, 0.025, 0.7),
    );
    style::draw_deep_panel(panel);
    draw_ui_text(
        "COLONY HELP",
        panel.x + 18.0,
        panel.y + 31.0,
        style::TITLE_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        "Reopen this guide any time from HELP.",
        panel.x + 18.0,
        panel.y + 55.0,
        style::SMALL_SIZE,
        style::TEXT_MUTED,
    );

    let text = &crate::data::config::game_config().text;
    let sections = [
        ("BUILD", text.intro_building.as_str()),
        (
            "ASSIGN",
            "Tap a survivor to inspect. Use NEXT ROLE, PAIR / APART, or FILTER ROOM before changing the colony.",
        ),
        ("RESEARCH", text.intro_missions.as_str()),
        (
            "RECOVER",
            "Use UNDO or CANCEL beside an active plan. MENU saves before leaving; outcomes offer Review, Restart, and Return to Menu.",
        ),
    ];
    for (index, (heading, body)) in sections.into_iter().enumerate() {
        let y = panel.y + 88.0 + index as f32 * 48.0;
        draw_ui_text(
            heading,
            panel.x + 18.0,
            y,
            style::SMALL_SIZE,
            style::ACCENT_GOLD,
        );
        draw_wrapped_text(
            body,
            panel.x + 92.0,
            y,
            panel.w - 110.0,
            style::SMALL_SIZE,
            style::TEXT_BODY,
        );
    }

    let close = help_close_rect(screen_width(), screen_height());
    style::draw_button(close, false, style::button_hovered(close));
    draw_ui_text(
        "CLOSE HELP",
        close.x + 49.0,
        close.y + 29.0,
        style::BODY_SIZE,
        style::TEXT_PRIMARY,
    );
}

fn briefing_copy(stage: usize) -> (&'static str, String) {
    let text = &crate::data::config::game_config().text;
    match stage {
        0 => ("THE WRECK IS STILL WARM", text.intro_building.clone()),
        1 => ("SIX LIVES, ONE CAMP", text.intro_assignments.clone()),
        2 => ("THE PERIMETER IS UNKNOWN", text.intro_missions.clone()),
        _ => ("YOUR RUN BELONGS TO YOU", text.intro_restart.clone()),
    }
}

fn draw_wrapped_text(text: &str, x: f32, start_y: f32, max_width: f32, size: f32, color: Color) {
    let mut line = String::new();
    let mut y = start_y;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && measure_ui_text(&candidate, None, size as u16, 1.0).width > max_width
        {
            draw_ui_text(&line, x, y, size, color);
            y += size + 6.0;
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_ui_text(&line, x, y, size, color);
    }
}
