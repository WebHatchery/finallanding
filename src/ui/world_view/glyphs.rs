//! Top-down glyphs for buildings without painted sprites, so each reads at a
//! glance on a busy late-game map.

use crate::ui::theme::{CYAN, SELECT};
use macroquad::prelude::*;
use macroquad_toolkit::colors::{darken, lighten};

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::new(r, g, b, 1.0)
}

fn flame(c: Vec2, size: f32) {
    draw_triangle(
        c + vec2(-size * 0.5, size * 0.4),
        c + vec2(size * 0.5, size * 0.4),
        c + vec2(0.0, -size * 0.8),
        rgb(0.95, 0.5, 0.15),
    );
    draw_triangle(
        c + vec2(-size * 0.25, size * 0.4),
        c + vec2(size * 0.25, size * 0.4),
        c + vec2(0.0, -size * 0.3),
        rgb(1.0, 0.85, 0.4),
    );
}

fn gear(c: Vec2, radius: f32, color: Color) {
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        let tooth = c + vec2(a.cos(), a.sin()) * radius;
        draw_circle(tooth.x, tooth.y, radius * 0.22, color);
    }
    draw_circle(c.x, c.y, radius * 0.8, color);
    draw_circle(c.x, c.y, radius * 0.32, darken(color, 0.5));
}

fn crates(rect: Rect, base: Color) {
    let size = (rect.w.min(rect.h) - 16.0) / 3.0;
    for row in 0..3 {
        for column in 0..3 {
            let x = rect.x + 8.0 + column as f32 * size;
            let y = rect.y + 8.0 + row as f32 * size;
            draw_rectangle(
                x + 1.0,
                y + 1.0,
                size - 3.0,
                size - 3.0,
                lighten(base, 0.08 * ((row + column) % 2) as f32),
            );
            draw_rectangle_lines(
                x + 1.0,
                y + 1.0,
                size - 3.0,
                size - 3.0,
                1.0,
                darken(base, 0.4),
            );
        }
    }
}

fn garden(rect: Rect) {
    draw_rectangle(
        rect.x + 4.0,
        rect.y + 4.0,
        rect.w - 8.0,
        rect.h - 8.0,
        rgb(0.25, 0.45, 0.26),
    );
    draw_line(
        rect.x + 4.0,
        rect.center().y,
        rect.x + rect.w - 4.0,
        rect.center().y,
        4.0,
        rgb(0.6, 0.55, 0.42),
    );
    draw_line(
        rect.center().x,
        rect.y + 4.0,
        rect.center().x,
        rect.y + rect.h - 4.0,
        4.0,
        rgb(0.6, 0.55, 0.42),
    );
    for (fx, fy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
        let p = vec2(rect.x + rect.w * fx, rect.y + rect.h * fy);
        draw_circle(p.x, p.y, rect.w * 0.13, rgb(0.2, 0.55, 0.3));
        draw_circle(p.x + 3.0, p.y - 2.0, rect.w * 0.05, rgb(0.95, 0.6, 0.75));
    }
}

fn columns(rect: Rect, base: Color) {
    let count = 5;
    for i in 0..count {
        let x = rect.x + 10.0 + i as f32 * (rect.w - 20.0) / (count - 1) as f32;
        draw_circle(x, rect.y + rect.h * 0.3, 5.0, lighten(base, 0.3));
        draw_circle(x, rect.y + rect.h * 0.7, 5.0, lighten(base, 0.3));
    }
    draw_rectangle(
        rect.x + rect.w * 0.3,
        rect.y + rect.h * 0.42,
        rect.w * 0.4,
        rect.h * 0.16,
        darken(base, 0.3),
    );
}

fn tower(c: Vec2, size: f32, base: Color) {
    draw_rectangle(
        c.x - size * 0.35,
        c.y - size * 0.35,
        size * 0.7,
        size * 0.7,
        darken(base, 0.2),
    );
    draw_rectangle_lines(
        c.x - size * 0.45,
        c.y - size * 0.45,
        size * 0.9,
        size * 0.9,
        2.0,
        lighten(base, 0.2),
    );
    draw_line(
        c.x - size * 0.45,
        c.y - size * 0.45,
        c.x + size * 0.45,
        c.y + size * 0.45,
        1.5,
        lighten(base, 0.1),
    );
    draw_line(
        c.x + size * 0.45,
        c.y - size * 0.45,
        c.x - size * 0.45,
        c.y + size * 0.45,
        1.5,
        lighten(base, 0.1),
    );
    draw_circle(c.x, c.y, size * 0.12, rgb(1.0, 0.75, 0.3));
}

fn house(rect: Rect, base: Color) {
    let mid = rect.y + rect.h * 0.5;
    draw_triangle(
        vec2(rect.x + 4.0, mid),
        vec2(rect.x + rect.w - 4.0, mid),
        vec2(rect.center().x, rect.y + 4.0),
        lighten(base, 0.12),
    );
    draw_triangle(
        vec2(rect.x + 4.0, mid),
        vec2(rect.x + rect.w - 4.0, mid),
        vec2(rect.center().x, rect.y + rect.h - 4.0),
        darken(base, 0.12),
    );
    draw_line(
        rect.x + 4.0,
        mid,
        rect.x + rect.w - 4.0,
        mid,
        2.0,
        darken(base, 0.4),
    );
    draw_rectangle(
        rect.center().x - 5.0,
        mid + 6.0,
        10.0,
        8.0,
        rgb(1.0, 0.8, 0.45),
    );
}

/// Draw a glyph for a specific building; returns false if it has none.
pub fn draw_building_glyph(kind: &str, rect: Rect, base: Color) -> bool {
    let c = rect.center();
    let small = rect.w.min(rect.h);
    match kind {
        "smelter" | "fibre_burner" => {
            draw_circle(c.x, c.y, small * 0.3, rgb(0.2, 0.12, 0.1));
            flame(c, small * 0.35);
        }
        "fabricator" => gear(c, small * 0.26, lighten(base, 0.25)),
        "warehouse" => crates(rect, base),
        "garden" => garden(rect),
        "memorial" => {
            draw_rectangle(
                c.x - 4.0,
                c.y - small * 0.4,
                8.0,
                small * 0.8,
                rgb(0.82, 0.82, 0.86),
            );
            draw_triangle(
                vec2(c.x - 4.0, c.y - small * 0.4),
                vec2(c.x + 4.0, c.y - small * 0.4),
                vec2(c.x, c.y - small * 0.52),
                rgb(0.9, 0.9, 0.95),
            );
        }
        "council_hall" => columns(rect, base),
        "xeno_lab" | "signal_mast" => {
            draw_circle_lines(c.x, c.y, small * 0.32, 3.0, rgb(0.75, 0.55, 1.0));
            draw_circle_lines(c.x, c.y, small * 0.18, 2.0, rgb(0.85, 0.7, 1.0));
            draw_circle(c.x, c.y, small * 0.07, rgb(1.0, 0.95, 1.0));
        }
        "laboratory" => {
            draw_triangle(
                vec2(c.x - small * 0.22, c.y + small * 0.25),
                vec2(c.x + small * 0.22, c.y + small * 0.25),
                vec2(c.x, c.y - small * 0.1),
                CYAN,
            );
            draw_rectangle(
                c.x - small * 0.06,
                c.y - small * 0.3,
                small * 0.12,
                small * 0.22,
                CYAN,
            );
        }
        "watchtower" => tower(c, small, base),
        "arc_turret" => {
            draw_circle(c.x, c.y, small * 0.32, darken(base, 0.4));
            draw_circle_lines(c.x, c.y, small * 0.32, 2.0, CYAN);
            draw_line(
                c.x,
                c.y,
                c.x + small * 0.4,
                c.y - small * 0.2,
                3.0,
                lighten(base, 0.3),
            );
        }
        "geothermal_tap" => {
            draw_circle(c.x, c.y, small * 0.3, rgb(0.3, 0.15, 0.08));
            draw_circle_lines(c.x, c.y, small * 0.22, 3.0, rgb(1.0, 0.55, 0.2));
            draw_circle(c.x, c.y, small * 0.1, rgb(1.0, 0.8, 0.4));
        }
        "family_quarters" | "bunkhouse" => house(rect, base),
        "lounge" => {
            draw_rectangle(
                c.x - small * 0.3,
                c.y - small * 0.05,
                small * 0.6,
                small * 0.25,
                lighten(base, 0.2),
            );
            draw_rectangle(
                c.x - small * 0.3,
                c.y - small * 0.18,
                small * 0.6,
                small * 0.12,
                darken(base, 0.2),
            );
            draw_circle(c.x, c.y - small * 0.32, small * 0.06, SELECT);
        }
        "hydroponics_bay" => {
            for i in 0..4 {
                let y = rect.y + 10.0 + i as f32 * (rect.h - 20.0) / 3.0;
                draw_line(
                    rect.x + 8.0,
                    y,
                    rect.x + rect.w - 8.0,
                    y,
                    5.0,
                    rgb(0.35, 0.8, 0.45),
                );
            }
        }
        _ => return false,
    }
    true
}
