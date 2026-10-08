//! The immediate-mode UI context: pointer, blocking regions, collected
//! actions, and the drawing helpers every screen shares.

use super::actions::UiAction;
use super::theme::*;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{
    draw_ui_text, measure_ui_text, truncate_text_to_width, wrap_text, Pointer,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Primary,
    Normal,
    Quiet,
    Danger,
    Selected,
}

pub struct Ui {
    pub pointer: Pointer,
    pub wheel: f32,
    pub time: f32,
    blockers: Vec<Rect>,
    actions: Vec<UiAction>,
}

impl Ui {
    pub fn new(pointer: Pointer, wheel: f32, time: f32) -> Self {
        Self {
            pointer,
            wheel,
            time,
            blockers: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Mark a region as UI so world input underneath is ignored.
    pub fn block(&mut self, rect: Rect) {
        self.blockers.push(rect);
    }

    pub fn is_over_ui(&self, point: Vec2) -> bool {
        self.blockers.iter().any(|r| r.contains(point))
    }

    pub fn act(&mut self, action: UiAction) {
        self.actions.push(action);
    }

    pub fn take_actions(&mut self) -> Vec<UiAction> {
        std::mem::take(&mut self.actions)
    }

    pub fn hovered(&self, rect: Rect) -> bool {
        self.pointer.hovering_over(rect) || self.pointer.pressing(rect)
    }

    /// An invisible tappable region; true on release inside it.
    pub fn hit(&mut self, rect: Rect) -> bool {
        self.block(rect);
        self.pointer.released_on(rect)
    }

    pub fn button(&mut self, rect: Rect, label: &str, tone: Tone, enabled: bool) -> bool {
        self.block(rect);
        let hovered = enabled && self.hovered(rect);
        let pressed = enabled && self.pointer.pressing(rect);
        let (fill, border, text) = button_colors(tone, enabled, hovered);
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border);
        if tone == Tone::Selected {
            draw_rectangle(rect.x, rect.y + rect.h - 3.0, rect.w, 3.0, ACCENT);
        }
        let size = if rect.h >= 44.0 {
            TEXT_BODY
        } else {
            TEXT_SMALL
        };
        let fitted = truncate_text_to_width(label, rect.w - 12.0, size);
        let width = text_width(&fitted, size);
        let offset = if pressed { 1.5 } else { 0.0 };
        draw_ui_text(
            &fitted,
            rect.x + (rect.w - width) * 0.5,
            rect.y + rect.h * 0.5 + size * 0.34 + offset,
            size,
            text,
        );
        enabled && self.pointer.released_on(rect)
    }

    /// A button that pushes an action when tapped.
    pub fn action_button(
        &mut self,
        rect: Rect,
        label: &str,
        tone: Tone,
        enabled: bool,
        action: UiAction,
    ) {
        if self.button(rect, label, tone, enabled) {
            self.act(action);
        }
    }
}

fn button_colors(tone: Tone, enabled: bool, hovered: bool) -> (Color, Color, Color) {
    if !enabled {
        return (Color::new(0.09, 0.1, 0.11, 0.9), LINE, TEXT_FAINT);
    }
    let lift = if hovered { 0.06 } else { 0.0 };
    let shade = |c: Color| Color::new(c.r + lift, c.g + lift, c.b + lift, c.a);
    match tone {
        Tone::Primary => (shade(Color::new(0.55, 0.36, 0.13, 1.0)), ACCENT, TEXT),
        Tone::Normal => (shade(PANEL_RAISED), LINE_STRONG, TEXT),
        Tone::Quiet => (shade(Color::new(0.1, 0.12, 0.14, 0.9)), LINE, TEXT_DIM),
        Tone::Danger => (shade(Color::new(0.45, 0.14, 0.12, 1.0)), BAD, TEXT),
        Tone::Selected => (shade(Color::new(0.2, 0.17, 0.11, 1.0)), ACCENT, TEXT),
    }
}

pub fn text_width(text: &str, size: f32) -> f32 {
    measure_ui_text(text, None, size.round() as u16, 1.0).width
}

/// Draw text with its top-left corner at (x, y).
pub fn label(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_ui_text(text, x, y + size * 0.78, size, color);
}

pub fn label_right(text: &str, right: f32, y: f32, size: f32, color: Color) {
    label(text, right - text_width(text, size), y, size, color);
}

pub fn label_centered(text: &str, center: f32, y: f32, size: f32, color: Color) {
    label(text, center - text_width(text, size) * 0.5, y, size, color);
}

/// One line, shortened with an ellipsis to fit the width.
pub fn label_fit(text: &str, x: f32, y: f32, width: f32, size: f32, color: Color) {
    let fitted = truncate_text_to_width(text, width, size);
    label(&fitted, x, y, size, color);
}

/// Wrapped paragraph; returns the height used.
pub fn paragraph(text: &str, x: f32, y: f32, width: f32, size: f32, color: Color) -> f32 {
    let line_height = size * 1.3;
    let mut cursor = y;
    for line in wrap_text(text, width, size) {
        label(&line, x, cursor, size, color);
        cursor += line_height;
    }
    cursor - y
}

pub fn paragraph_height(text: &str, width: f32, size: f32) -> f32 {
    wrap_text(text, width, size).len() as f32 * size * 1.3
}

pub fn panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE);
}

pub fn panel_solid(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_SOLID);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE);
}

/// A horizontal meter with a dim track.
pub fn meter(rect: Rect, fraction: f32, color: Color) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(1.0, 1.0, 1.0, 0.07),
    );
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w * fraction.clamp(0.0, 1.0),
        rect.h,
        color,
    );
}

/// A small coloured chip with text.
pub fn chip(x: f32, y: f32, text: &str, color: Color) -> f32 {
    let width = text_width(text, TEXT_SMALL) + 16.0;
    draw_rectangle(
        x,
        y,
        width,
        26.0,
        Color::new(color.r, color.g, color.b, 0.16),
    );
    draw_rectangle_lines(
        x,
        y,
        width,
        26.0,
        1.0,
        Color::new(color.r, color.g, color.b, 0.55),
    );
    label(text, x + 8.0, y + 3.0, TEXT_SMALL, color);
    width
}

pub fn divider(x: f32, y: f32, width: f32) {
    draw_line(x, y, x + width, y, 1.0, LINE);
}

/// A section heading in small caps style.
pub fn heading(text: &str, x: f32, y: f32) {
    label(&text.to_uppercase(), x, y, TEXT_SMALL, ACCENT);
}

/// Small vector icons for glyphs the UI font lacks.
pub fn icon_heart(x: f32, y: f32, size: f32, color: Color) {
    let r = size * 0.28;
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + size - r, y + r, r, color);
    draw_triangle(
        vec2(x, y + r * 1.2),
        vec2(x + size, y + r * 1.2),
        vec2(x + size * 0.5, y + size),
        color,
    );
}

pub fn icon_check(x: f32, y: f32, size: f32, color: Color) {
    draw_line(
        x,
        y + size * 0.55,
        x + size * 0.38,
        y + size * 0.9,
        2.0,
        color,
    );
    draw_line(
        x + size * 0.38,
        y + size * 0.9,
        x + size,
        y + size * 0.15,
        2.0,
        color,
    );
}

pub fn icon_arrow(x: f32, y: f32, size: f32, color: Color) {
    draw_triangle(
        vec2(x, y),
        vec2(x, y + size),
        vec2(x + size * 0.8, y + size * 0.5),
        color,
    );
}

pub fn icon_star(x: f32, y: f32, size: f32, color: Color) {
    let c = vec2(x + size * 0.5, y + size * 0.5);
    draw_poly(c.x, c.y, 4, size * 0.55, 45.0, color);
    draw_poly(c.x, c.y, 4, size * 0.55, 0.0, color);
}
