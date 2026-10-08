//! Drawing landscape features and buildings in world space.

use crate::data::NodeKind;
use crate::ui::art::Art;
use crate::ui::camera::{tile_center, tile_size};
use crate::ui::theme::*;
use crate::world::{CropStage, ResourceNode, Structure, Tile, World};
use macroquad::prelude::*;
use macroquad_toolkit::colors::{darken, lighten, with_alpha};

fn hash(tile: Tile, salt: u32) -> f32 {
    let mut h = (tile.x as u32).wrapping_mul(374_761_393)
        ^ (tile.y as u32).wrapping_mul(668_265_263)
        ^ salt.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h & 0xffff) as f32 / 65535.0
}

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::new(r, g, b, 1.0)
}

pub fn draw_node(node: &ResourceNode, time: f32) {
    let c = tile_center(node.tile);
    let s = tile_size();
    let fullness = (node.amount / node.max_amount.max(1.0)).clamp(0.15, 1.0);
    match node.kind {
        NodeKind::Wreckage => {
            let r = s * (0.3 + fullness * 0.25);
            let tilt = hash(node.tile, 1) * std::f32::consts::TAU;
            let p = |a: f32, d: f32| c + vec2((tilt + a).cos(), (tilt + a).sin()) * r * d;
            draw_circle(
                c.x + 3.0,
                c.y + 4.0,
                r * 0.9,
                Color::new(0.0, 0.0, 0.0, 0.3),
            );
            draw_triangle(p(0.0, 1.0), p(2.2, 0.8), p(4.1, 1.1), rgb(0.42, 0.44, 0.47));
            draw_triangle(p(1.0, 0.9), p(3.0, 1.0), p(5.0, 0.6), rgb(0.33, 0.35, 0.38));
            draw_circle(p(0.5, 0.4).x, p(0.5, 0.4).y, r * 0.18, rgb(0.62, 0.36, 0.2));
            draw_line(
                p(0.0, 1.0).x,
                p(0.0, 1.0).y,
                p(3.0, 1.0).x,
                p(3.0, 1.0).y,
                1.5,
                rgb(0.6, 0.62, 0.65),
            );
        }
        NodeKind::FibreGrove => {
            let blades = (4.0 + fullness * 7.0) as i32;
            for i in 0..blades {
                let ox = (hash(node.tile, i as u32) - 0.5) * s * 0.8;
                let sway = (time * 1.3 + i as f32).sin() * 2.0;
                let base = c + vec2(ox, s * 0.35);
                let tip = base
                    + vec2(
                        sway + ox * 0.2,
                        -s * (0.45 + hash(node.tile, i as u32 + 9) * 0.3),
                    );
                draw_line(base.x, base.y, tip.x, tip.y, 2.5, rgb(0.55, 0.7, 0.32));
                draw_circle(tip.x, tip.y, 2.0, rgb(0.8, 0.78, 0.45));
            }
        }
        NodeKind::StoneOutcrop => {
            let r = s * (0.28 + fullness * 0.22);
            draw_circle(c.x + 3.0, c.y + 4.0, r, Color::new(0.0, 0.0, 0.0, 0.3));
            draw_circle(c.x, c.y, r, rgb(0.52, 0.5, 0.46));
            draw_circle(
                c.x - r * 0.45,
                c.y + r * 0.2,
                r * 0.6,
                rgb(0.46, 0.44, 0.41),
            );
            draw_circle(c.x - r * 0.25, c.y - r * 0.3, r * 0.3, rgb(0.62, 0.6, 0.56));
        }
        NodeKind::OreVein => {
            let r = s * 0.42;
            draw_circle(c.x + 3.0, c.y + 4.0, r, Color::new(0.0, 0.0, 0.0, 0.3));
            draw_circle(c.x, c.y, r, rgb(0.24, 0.25, 0.28));
            let glints = (2.0 + fullness * 4.0) as i32;
            for i in 0..glints {
                let a = hash(node.tile, i as u32) * std::f32::consts::TAU;
                let p = c + vec2(a.cos(), a.sin()) * r * 0.55;
                let pulse = 0.6 + 0.4 * (time * 2.0 + i as f32).sin();
                draw_poly(p.x, p.y, 4, 3.5, a.to_degrees(), with_alpha(CYAN, pulse));
            }
        }
        NodeKind::Glowfruit => {
            let r = s * 0.42;
            draw_circle(c.x, c.y, r, rgb(0.17, 0.32, 0.18));
            draw_circle(c.x + r * 0.3, c.y - r * 0.2, r * 0.6, rgb(0.21, 0.4, 0.22));
            let fruit = (fullness * 7.0) as i32;
            for i in 0..fruit {
                let a = hash(node.tile, i as u32 + 3) * std::f32::consts::TAU;
                let p = c + vec2(a.cos(), a.sin()) * r * 0.6;
                draw_circle(p.x, p.y, 3.0, rgb(0.98, 0.86, 0.35));
                draw_circle(p.x, p.y, 6.0, Color::new(1.0, 0.85, 0.35, 0.18));
            }
        }
        NodeKind::Ruin => {
            let glow = 0.25 + 0.15 * (time * 0.8).sin();
            draw_circle(
                c.x,
                c.y,
                s * 0.9,
                Color::new(0.65, 0.45, 0.95, glow * fullness),
            );
            for i in 0..3 {
                let x = c.x - s * 0.35 + i as f32 * s * 0.35;
                let h = s * (0.5 + hash(node.tile, i) * 0.5);
                draw_rectangle(x - 4.0, c.y + s * 0.3 - h, 8.0, h, rgb(0.42, 0.38, 0.5));
                draw_rectangle(x - 4.0, c.y + s * 0.3 - h, 8.0, 3.0, rgb(0.75, 0.6, 0.95));
            }
        }
    }
}

fn footprint_rect(structure: &Structure) -> Rect {
    let s = tile_size();
    let fp = structure.footprint;
    Rect::new(
        fp.origin.x as f32 * s,
        fp.origin.y as f32 * s,
        fp.width as f32 * s,
        fp.height as f32 * s,
    )
}

fn draw_blueprint(structure: &Structure, rect: Rect, time: f32) {
    let pulse = 0.5 + 0.2 * (time * 2.5).sin();
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.35, 0.75, 0.85, 0.12),
    );
    let dash = 10.0;
    let mut x = rect.x;
    while x < rect.x + rect.w {
        let end = (x + dash * 0.6).min(rect.x + rect.w);
        draw_line(x, rect.y, end, rect.y, 2.0, with_alpha(CYAN, pulse));
        draw_line(
            x,
            rect.y + rect.h,
            end,
            rect.y + rect.h,
            2.0,
            with_alpha(CYAN, pulse),
        );
        x += dash;
    }
    let mut y = rect.y;
    while y < rect.y + rect.h {
        let end = (y + dash * 0.6).min(rect.y + rect.h);
        draw_line(rect.x, y, rect.x, end, 2.0, with_alpha(CYAN, pulse));
        draw_line(
            rect.x + rect.w,
            y,
            rect.x + rect.w,
            end,
            2.0,
            with_alpha(CYAN, pulse),
        );
        y += dash;
    }
    let fraction = structure.construction_fraction();
    if fraction > 0.0 {
        let built_h = rect.h * fraction;
        draw_rectangle(
            rect.x,
            rect.y + rect.h - built_h,
            rect.w,
            built_h,
            with_alpha(structure_color(structure), 0.55),
        );
    }
}

fn structure_color(structure: &Structure) -> Color {
    let c = structure.def().color;
    Color::from_rgba(c[0], c[1], c[2], 255)
}

fn draw_farm(structure: &Structure, rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, rgb(0.27, 0.2, 0.13));
    let Some(crop) = &structure.crop else {
        return;
    };
    let rows = (rect.h / 10.0) as i32;
    for row in 0..rows {
        let y = rect.y + 5.0 + row as f32 * 10.0;
        draw_line(
            rect.x + 3.0,
            y,
            rect.x + rect.w - 3.0,
            y,
            2.0,
            rgb(0.22, 0.16, 0.1),
        );
        if crop.stage == CropStage::Fallow {
            continue;
        }
        let size = 1.5 + crop.growth.min(1.0) * 3.5;
        let color = if crop.stage == CropStage::Ripe {
            rgb(0.9, 0.78, 0.3)
        } else {
            rgb(0.4, 0.68, 0.3)
        };
        let mut x = rect.x + 6.0;
        while x < rect.x + rect.w - 4.0 {
            draw_circle(x, y - 1.0, size, color);
            x += 9.0;
        }
    }
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, rgb(0.45, 0.36, 0.22));
}

fn draw_built(structure: &Structure, rect: Rect, art: &Art, night: f32) {
    let def = structure.def();
    if def.farm.is_some() && !def.indoor {
        draw_farm(structure, rect);
        return;
    }
    let base = structure_color(structure);
    let worn = 1.0 - structure.condition / 100.0;
    if structure.kind == "meridian_hull" {
        draw_hull(rect);
    } else if structure.kind == "survival_tent" {
        draw_tent(rect, base);
    } else if let Some(sprite) = def.sprite {
        art.draw_building(
            sprite,
            rect,
            Color::new(1.0 - worn * 0.3, 1.0 - worn * 0.35, 1.0 - worn * 0.35, 1.0),
        );
    } else {
        draw_rectangle(
            rect.x + 6.0,
            rect.y + 8.0,
            rect.w,
            rect.h,
            Color::new(0.0, 0.0, 0.0, 0.35),
        );
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, darken(base, 0.25));
        let inset = (rect.w.min(rect.h) * 0.14).max(4.0);
        draw_rectangle(
            rect.x + inset,
            rect.y + inset,
            rect.w - inset * 2.0,
            rect.h - inset * 2.0,
            base,
        );
        draw_rectangle(
            rect.x + inset,
            rect.y + inset,
            rect.w - inset * 2.0,
            4.0,
            lighten(base, 0.15),
        );
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, darken(base, 0.45));
        if !super::glyphs::draw_building_glyph(&structure.kind, rect, base) {
            draw_detail(structure, rect, base);
        }
    }
    if night > 0.2 && structure.is_operational() && (def.indoor || def.heat > 0.0) {
        let c = rect.center();
        let glow = Color::new(1.0, 0.72, 0.35, 0.12 * night);
        draw_circle(c.x, c.y, rect.w.max(rect.h) * 0.9, glow);
        draw_circle(c.x, c.y, rect.w.max(rect.h) * 0.5, glow);
    }
    if worn > 0.45 {
        draw_line(
            rect.x + rect.w * 0.2,
            rect.y + rect.h * 0.3,
            rect.x + rect.w * 0.5,
            rect.y + rect.h * 0.6,
            2.0,
            Color::new(0.05, 0.05, 0.05, 0.8),
        );
        draw_line(
            rect.x + rect.w * 0.5,
            rect.y + rect.h * 0.6,
            rect.x + rect.w * 0.45,
            rect.y + rect.h * 0.85,
            2.0,
            Color::new(0.05, 0.05, 0.05, 0.8),
        );
    }
    if def.needs_power() && !structure.powered {
        let p = vec2(rect.x + rect.w - 12.0, rect.y + 12.0);
        draw_circle(p.x, p.y, 9.0, Color::new(0.1, 0.05, 0.05, 0.9));
        draw_triangle(
            p + vec2(2.0, -7.0),
            p + vec2(-4.0, 1.0),
            p + vec2(1.0, 1.0),
            BAD,
        );
        draw_triangle(
            p + vec2(-1.0, -1.0),
            p + vec2(4.0, -1.0),
            p + vec2(-2.0, 7.0),
            BAD,
        );
    }
}

fn draw_hull(rect: Rect) {
    let hull = rgb(0.6, 0.62, 0.64);
    draw_rectangle(
        rect.x + 6.0,
        rect.y + 10.0,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    draw_circle(
        rect.x + rect.h * 0.5,
        rect.y + rect.h * 0.5,
        rect.h * 0.5,
        darken(hull, 0.2),
    );
    draw_rectangle(
        rect.x + rect.h * 0.5,
        rect.y,
        rect.w - rect.h * 0.5,
        rect.h,
        hull,
    );
    draw_triangle(
        vec2(rect.x + rect.w, rect.y),
        vec2(rect.x + rect.w + rect.h * 0.4, rect.y + rect.h * 0.5),
        vec2(rect.x + rect.w, rect.y + rect.h),
        darken(hull, 0.1),
    );
    for i in 1..6 {
        let x = rect.x + rect.h * 0.5 + i as f32 * (rect.w - rect.h * 0.5) / 6.0;
        draw_line(
            x,
            rect.y + 3.0,
            x,
            rect.y + rect.h - 3.0,
            1.0,
            darken(hull, 0.3),
        );
    }
    draw_rectangle(
        rect.x + rect.h * 0.6,
        rect.y + rect.h * 0.4,
        rect.w * 0.6,
        rect.h * 0.18,
        rgb(0.3, 0.36, 0.42),
    );
    for i in 0..6 {
        let x = rect.x + rect.h * 0.7 + i as f32 * rect.w * 0.1;
        draw_rectangle(
            x,
            rect.y + rect.h * 0.43,
            rect.w * 0.05,
            rect.h * 0.12,
            rgb(1.0, 0.8, 0.45),
        );
    }
    draw_line(
        rect.x + rect.w * 0.35,
        rect.y,
        rect.x + rect.w * 0.45,
        rect.y + rect.h,
        2.0,
        rgb(0.25, 0.2, 0.18),
    );
    draw_rectangle(
        rect.x + rect.w * 0.82,
        rect.y + rect.h * 0.15,
        rect.w * 0.08,
        rect.h * 0.2,
        rgb(0.62, 0.36, 0.2),
    );
}

fn draw_tent(rect: Rect, base: Color) {
    let top = vec2(rect.x + rect.w * 0.5, rect.y + 2.0);
    draw_triangle(
        vec2(rect.x + 6.0, rect.y + rect.h + 4.0),
        vec2(rect.x + rect.w + 8.0, rect.y + rect.h + 4.0),
        top + vec2(10.0, 6.0),
        Color::new(0.0, 0.0, 0.0, 0.3),
    );
    draw_triangle(
        vec2(rect.x, rect.y + rect.h),
        vec2(rect.x + rect.w, rect.y + rect.h),
        top,
        base,
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.5, rect.y + rect.h),
        vec2(rect.x + rect.w, rect.y + rect.h),
        top,
        darken(base, 0.18),
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.4, rect.y + rect.h),
        vec2(rect.x + rect.w * 0.6, rect.y + rect.h),
        vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.55),
        rgb(0.12, 0.1, 0.08),
    );
}

/// Category glyphs so buildings without painted sprites read at a glance.
fn draw_detail(structure: &Structure, rect: Rect, base: Color) {
    let def = structure.def();
    let c = rect.center();
    let light = lighten(base, 0.3);
    if def.heat > 0.0 && !def.indoor && def.size[0] == 1 {
        draw_circle(c.x, c.y, rect.w * 0.3, rgb(0.95, 0.55, 0.15));
        draw_circle(c.x, c.y - 2.0, rect.w * 0.17, rgb(1.0, 0.85, 0.4));
    } else if def.solar {
        for i in 0..3 {
            let y = rect.y + rect.h * (0.25 + i as f32 * 0.22);
            draw_line(
                rect.x + 6.0,
                y,
                rect.x + rect.w - 6.0,
                y,
                1.5,
                rgb(0.55, 0.7, 0.95),
            );
        }
    } else if def.research > 0.0 {
        draw_circle_lines(c.x, c.y, rect.w.min(rect.h) * 0.22, 2.0, light);
        draw_circle(c.x, c.y, 3.0, CYAN);
    } else if def.care_beds > 0 {
        draw_rectangle(c.x - 3.0, c.y - 10.0, 6.0, 20.0, rgb(0.85, 0.25, 0.25));
        draw_rectangle(c.x - 10.0, c.y - 3.0, 20.0, 6.0, rgb(0.85, 0.25, 0.25));
    } else if def.defence > 0.0 {
        draw_circle(c.x, c.y, rect.w * 0.22, darken(base, 0.4));
        draw_circle_lines(c.x, c.y, rect.w * 0.22, 2.0, light);
    } else if def.beds > 0 {
        let beds = def.beds.min(6);
        for i in 0..beds {
            let x = rect.x + 8.0 + (i % 3) as f32 * (rect.w - 16.0) / 3.0;
            let y = rect.y + 8.0 + (i / 3) as f32 * 14.0;
            draw_rectangle(x, y, (rect.w - 16.0) / 3.0 - 4.0, 9.0, light);
        }
    } else if def.capstone.is_some() {
        draw_poly(c.x, c.y, 6, rect.w * 0.3, 0.0, light);
        draw_poly_lines(c.x, c.y, 6, rect.w * 0.36, 0.0, 2.0, SELECT);
    }
}

pub fn draw_structure(structure: &Structure, art: &Art, time: f32, night: f32) {
    let rect = footprint_rect(structure);
    if structure.is_built() {
        draw_built(structure, rect, art, night);
    } else {
        draw_blueprint(structure, rect, time);
    }
}

pub fn structure_rect(world: &World, id: u32) -> Option<Rect> {
    world.structure(id).map(footprint_rect)
}
