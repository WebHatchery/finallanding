//! advisor overlay domain.

use super::Layout;
use crate::data::event_log::LogCategory;
use crate::data::resources::ResourceState;
use crate::state::game_state::ActionFeedback;
use crate::systems::advisor_system::{AdvisorPlan, AdvisorSeverity};
use crate::systems::objective_system::{ObjectiveCard, ObjectiveStatus};
use crate::ui::style;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_advisor_overlay(layout: &Layout, objectives: &[ObjectiveCard], plan: &AdvisorPlan) {
    let rail = layout.left_panel();
    let objective_rect = Rect::new(rail.x, rail.y, rail.w, 172.0);
    style::draw_panel(objective_rect);

    style::draw_section_title(
        crate::data::config::game_config()
            .text
            .label("objectives_heading"),
        objective_rect.x + 18.0,
        objective_rect.y + 31.0,
    );
    for (index, objective) in objectives.iter().take(4).enumerate() {
        draw_objective_card(
            objective_rect.x + 14.0,
            objective_rect.y + 50.0 + index as f32 * 29.0,
            objective_rect.w - 28.0,
            objective,
        );
    }

    let alert_y = objective_rect.y + objective_rect.h + 12.0;
    for (index, line) in plan.lines.iter().take(2).enumerate() {
        let row = Rect::new(rail.x, alert_y + index as f32 * 44.0, rail.w, 38.0);
        draw_rectangle(row.x, row.y, row.w, row.h, alert_bg_color(line.severity));
        draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, style::PANEL_BORDER);
        draw_rectangle(
            row.x + 14.0,
            row.y + 12.0,
            10.0,
            10.0,
            severity_color(line.severity),
        );
        draw_ui_text(
            &style::fit_text(&line.title, row.w - 46.0, style::SMALL_SIZE),
            row.x + 34.0,
            row.y + 16.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_ui_text(
            &style::fit_text(&line.detail, row.w - 46.0, style::TINY_SIZE),
            row.x + 34.0,
            row.y + 31.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }
}

pub fn draw_advisor_banner(
    layout: &Layout,
    plan: &AdvisorPlan,
    resources: &ResourceState,
    colonist_count: usize,
    average_mood: f32,
) {
    let Some(line) = plan.lines.first() else {
        return;
    };
    let rect = advisor_banner_rect(layout);
    style::draw_deep_panel(rect);
    draw_rectangle(rect.x, rect.y, 4.0, rect.h, severity_color(line.severity));
    draw_ui_text(
        &style::fit_text(&line.title, rect.w * 0.42, style::SMALL_SIZE),
        rect.x + 14.0,
        rect.y + 18.0,
        style::SMALL_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        &style::fit_text(&line.detail, rect.w * 0.42, style::TINY_SIZE),
        rect.x + 14.0,
        rect.y + 34.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );

    let status = format!(
        "{} survivors  Mood {:.0}  Food {}  Salvage {}",
        colonist_count, average_mood, resources.supplies, resources.salvage
    );
    draw_ui_text(
        &style::fit_text(&status, rect.w * 0.48, style::TINY_SIZE),
        rect.x + rect.w * 0.51,
        rect.y + 27.0,
        style::TINY_SIZE,
        if resources.supplies < 10 {
            style::ALERT_RED
        } else {
            style::TEXT_MUTED
        },
    );
}

pub fn advisor_banner_rect(layout: &Layout) -> Rect {
    let width = (layout.viewport_width - layout.screen_margin * 2.0).clamp(300.0, 660.0);
    Rect::new(
        layout.screen_margin,
        layout.top_bar_height + 8.0,
        width,
        46.0,
    )
}

pub fn draw_action_feedback(layout: &Layout, feedback: &ActionFeedback) {
    let banner = advisor_banner_rect(layout);
    let width = (banner.w * 0.72).clamp(300.0, 540.0);
    let rect = Rect::new(banner.x, banner.bottom() + 6.0, width, 48.0);
    let accent = match feedback.category {
        LogCategory::System | LogCategory::Mission => style::ACCENT_GOLD,
        LogCategory::Resource | LogCategory::Colony => style::BAR_GREEN,
        LogCategory::Mood | LogCategory::Social => style::HEADING_BLUE,
        LogCategory::Time | LogCategory::Work | LogCategory::Technology => style::TEXT_MUTED,
    };
    style::draw_deep_panel(rect);
    draw_rectangle(rect.x, rect.y, 4.0, rect.h, accent);
    draw_ui_text(
        &style::fit_text(&feedback.title, width - 28.0, style::SMALL_SIZE),
        rect.x + 14.0,
        rect.y + 19.0,
        style::SMALL_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        &style::fit_text(&feedback.detail, width - 28.0, style::TINY_SIZE),
        rect.x + 14.0,
        rect.y + 35.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );
}

pub fn draw_objective_card(x: f32, y: f32, width: f32, objective: &ObjectiveCard) {
    let status_color = status_color(objective.status);
    draw_rectangle(x, y, width, 24.0, Color::new(0.045, 0.06, 0.065, 0.74));
    draw_rectangle(x, y, 3.0, 24.0, status_color);
    draw_rectangle_lines(x, y, width, 24.0, 1.0, style::PANEL_DIVIDER);

    let box_x = x + 10.0;
    let box_y = y + 6.0;
    draw_rectangle_lines(box_x, box_y, 10.0, 10.0, 1.0, style::TEXT_MUTED);
    if objective.status == ObjectiveStatus::Complete {
        draw_line(
            box_x + 2.0,
            box_y + 5.0,
            box_x + 5.0,
            box_y + 9.0,
            2.0,
            style::BAR_GREEN,
        );
        draw_line(
            box_x + 5.0,
            box_y + 9.0,
            box_x + 11.0,
            box_y + 0.0,
            2.0,
            style::BAR_GREEN,
        );
    }

    draw_ui_text(
        &style::fit_text(&objective.title, width - 109.0, style::TINY_SIZE),
        x + 28.0,
        y + 11.0,
        style::TINY_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        &style::fit_text(&objective.detail, width - 40.0, style::TINY_SIZE),
        x + 28.0,
        y + 22.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    style::draw_progress_bar(
        Rect::new(x + width - 73.0, y + 8.0, 58.0, 7.0),
        objective.progress,
        status_color,
    );
}

fn status_color(status: ObjectiveStatus) -> Color {
    match status {
        ObjectiveStatus::Complete => style::BAR_GREEN,
        ObjectiveStatus::Active => style::HEADING_BLUE,
        ObjectiveStatus::AtRisk => style::ALERT_RED,
    }
}

fn alert_bg_color(severity: AdvisorSeverity) -> Color {
    match severity {
        AdvisorSeverity::Stable => Color::new(0.07, 0.12, 0.1, 0.88),
        AdvisorSeverity::Action => Color::new(0.13, 0.11, 0.06, 0.88),
        AdvisorSeverity::Warning => Color::new(0.16, 0.07, 0.055, 0.9),
    }
}

fn severity_color(severity: AdvisorSeverity) -> Color {
    match severity {
        AdvisorSeverity::Stable => style::BAR_GREEN,
        AdvisorSeverity::Action => style::BAR_GOLD,
        AdvisorSeverity::Warning => style::ALERT_RED,
    }
}
