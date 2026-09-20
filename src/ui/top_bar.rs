//! Top bar UI component - time display and speed controls

use super::Layout;
use crate::data::game_state::TimeSpeed;
use crate::data::resources::ResourceState;
use crate::systems::time_system::TimeSystem;
use crate::ui::hit_zones::{speed_button_rect_for, top_bar_action_rect, TopBarAction};
use crate::ui::style;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, format_clock, measure_ui_text};

/// Draw the top bar with time and speed controls
pub fn draw_top_bar(
    layout: &Layout,
    tick: u64,
    current_speed: TimeSpeed,
    colonist_count: usize,
    average_mood: f32,
    resources: &ResourceState,
) {
    let rect = layout.top_bar();

    style::draw_deep_panel(Rect::new(12.0, 12.0, rect.w.min(840.0), rect.h - 16.0));

    let phone = layout.viewport_width < 760.0;

    // Title
    draw_ui_text(
        if phone {
            "FINAL LANDING"
        } else {
            "THE FINAL LANDING"
        },
        if phone { 12.0 } else { 30.0 },
        if phone { 19.0 } else { 42.0 },
        if phone { 14.0 } else { style::TITLE_SIZE },
        style::TEXT_PRIMARY,
    );

    // Time display
    let (day, hour, minute) = TimeSystem::get_time_of_day(tick);
    let time_str = format!("Day {}, {}", day, format_clock(hour, minute));
    let is_night = TimeSystem::is_night(tick);
    let time_color = if is_night {
        Color::new(0.6, 0.7, 1.0, 1.0)
    } else {
        Color::new(1.0, 0.9, 0.6, 1.0)
    };
    let time_icon = if is_night { "MOON" } else { "SUN" };

    draw_ui_text(
        &format!("{}  {}", time_str, time_icon),
        if phone { 130.0 } else { 565.0 },
        if phone { 19.0 } else { 42.0 },
        if phone { 12.0 } else { 18.0 },
        time_color,
    );

    // Speed controls
    let speeds = [
        (TimeSpeed::Paused, "II", "Pause"),
        (TimeSpeed::Normal, ">", "Normal"),
        (TimeSpeed::Fast, ">>", "Fast"),
        (TimeSpeed::SuperFast, ">>>", "Super"),
    ];

    for (i, (speed, label, _tooltip)) in speeds.iter().enumerate() {
        let button_rect = speed_button_rect_for(layout, i);
        let is_active = current_speed == *speed;

        style::draw_button(button_rect, is_active, style::button_hovered(button_rect));

        let text_w = measure_ui_text(label, None, 16, 1.0).width;
        draw_ui_text(
            label,
            button_rect.x + (button_rect.w - text_w) / 2.0,
            button_rect.y + if phone { 14.0 } else { 20.0 },
            if phone { 11.0 } else { 16.0 },
            if is_active {
                style::HEADING_BLUE
            } else {
                style::TEXT_BODY
            },
        );
    }

    for action in [TopBarAction::Undo, TopBarAction::Cancel, TopBarAction::Menu] {
        let action_rect = top_bar_action_rect(layout, action);
        let hovered = style::button_hovered(action_rect);
        style::draw_button(action_rect, false, hovered);
        let label = match action {
            TopBarAction::Undo => crate::data::config::game_config()
                .text
                .label("toolbar_undo"),
            TopBarAction::Cancel => crate::data::config::game_config()
                .text
                .label("toolbar_cancel"),
            TopBarAction::Menu => "MENU",
        };
        let label_width = measure_ui_text(label, None, 12, 1.0).width;
        draw_ui_text(
            label,
            action_rect.x + (action_rect.w - label_width) * 0.5,
            action_rect.y + if phone { 14.0 } else { 20.0 },
            if phone { 9.0 } else { 12.0 },
            style::TEXT_PRIMARY,
        );
    }

    if phone {
        return;
    }

    let status_label = format!(
        "C:{} Mood:{:.0} Supplies:{} Salvage:{} {}",
        colonist_count,
        average_mood,
        resources.supplies,
        resources.salvage,
        resources.condition.label()
    );
    let actions_start = top_bar_action_rect(layout, TopBarAction::Undo).x;
    let status_x = if layout.viewport_width < 1_100.0 {
        430.0
    } else {
        700.0
    };
    let status_width = measure_ui_text(&status_label, None, 16, 1.0).width;
    if status_x + status_width <= actions_start - 10.0 {
        draw_ui_text(&status_label, status_x, 42.0, 16.0, style::TEXT_BODY);
    } else {
        let compact_status = format!(
            "Mood:{:.0} S:{} {}",
            average_mood,
            resources.supplies,
            resources.condition.label()
        );
        let compact_width = measure_ui_text(&compact_status, None, 14, 1.0).width;
        let compact_x = rect.w - compact_width - 10.0;
        if compact_x > status_x + 10.0 && compact_x + compact_width <= actions_start - 10.0 {
            draw_ui_text(&compact_status, compact_x, 42.0, 14.0, style::TEXT_BODY);
        }
    }
}
