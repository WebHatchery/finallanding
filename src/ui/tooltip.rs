use crate::ui::style;
use macroquad::prelude::*;
use macroquad_toolkit::input::mouse_position_vec2;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const PAD_X: f32 = 10.0;
const PAD_Y: f32 = 8.0;
const GAP_FROM_CURSOR: Vec2 = Vec2::new(14.0, 16.0);

pub fn draw_tooltip_near_mouse(bounds: Rect, title: &str, body: &str) {
    let mouse = mouse_position_vec2();
    draw_tooltip_at(mouse + GAP_FROM_CURSOR, bounds, title, body);
}

pub fn draw_tooltip_at(anchor: Vec2, bounds: Rect, title: &str, body: &str) {
    let title_text = style::truncate_text(title, 28);
    let body_text = style::truncate_text(body, 48);
    let title_width = measure_ui_text(&title_text, None, style::SMALL_SIZE as u16, 1.0).width;
    let body_width = measure_ui_text(&body_text, None, style::TINY_SIZE as u16, 1.0).width;
    let width = title_width.max(body_width).clamp(130.0, 300.0) + PAD_X * 2.0;
    let height = 44.0;
    let rect = tooltip_rect(anchor, bounds, width, height);

    style::draw_deep_panel(rect);
    draw_rectangle(rect.x, rect.y, 3.0, rect.h, style::HEADING_BLUE);
    draw_ui_text(
        &title_text,
        rect.x + PAD_X,
        rect.y + PAD_Y + 11.0,
        style::SMALL_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        &body_text,
        rect.x + PAD_X,
        rect.y + PAD_Y + 28.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );
}

pub fn tooltip_rect(anchor: Vec2, bounds: Rect, width: f32, height: f32) -> Rect {
    let max_x = bounds.x + bounds.w - width;
    let max_y = bounds.y + bounds.h - height;
    Rect::new(
        anchor.x.clamp(bounds.x, max_x.max(bounds.x)),
        anchor.y.clamp(bounds.y, max_y.max(bounds.y)),
        width,
        height,
    )
}

#[cfg(test)]
mod tests;
