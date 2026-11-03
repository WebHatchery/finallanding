//! Each native species drawn as itself: tubers in mounds, fungus caps, shells,
//! crystal ores, glyph stones and the dormant engine, all in the species'
//! colour so a landing's patches read at a glance.

use super::features::hash;
use crate::world::Tile;
use macroquad::prelude::*;
use macroquad_toolkit::colors::{darken, lighten, with_alpha};
use std::f32::consts::TAU;

/// Where and how a species node is drawn.
pub struct Spot {
    pub c: Vec2,
    pub s: f32,
    pub color: Color,
    /// Remaining amount as a fraction of the node's full size.
    pub fullness: f32,
    pub tile: Tile,
    pub time: f32,
}

impl Spot {
    fn h(&self, salt: u32) -> f32 {
        hash(self.tile, salt)
    }

    /// How many repeated parts to draw, from `min` when nearly spent to `max`.
    fn count(&self, min: f32, max: f32) -> i32 {
        (min + (max - min) * self.fullness) as i32
    }

    /// A seeded point within `radius` tiles of the centre.
    fn scatter(&self, salt: u32, radius: f32) -> Vec2 {
        let a = self.h(salt) * TAU;
        let d = self.h(salt + 101).sqrt() * radius * self.s;
        self.c + vec2(a.cos(), a.sin()) * d
    }

    fn shadow(&self, r: f32) {
        draw_ellipse(
            self.c.x + 3.0,
            self.c.y + 4.0,
            r,
            r * 0.75,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.3),
        );
    }
}

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::new(r, g, b, 1.0)
}

fn line(a: Vec2, b: Vec2, width: f32, color: Color) {
    draw_line(a.x, a.y, b.x, b.y, width, color);
}

/// Draw a species by id; false when it has no art of its own.
pub fn draw_species(id: &str, spot: &Spot) -> bool {
    match id {
        "glowfruit" => glowfruit(spot),
        "ember_tuber" => ember_tuber(spot),
        "veilcap" => veilcap(spot),
        "reedgrain" => reedgrain(spot),
        "bitterleaf" => bitterleaf(spot),
        "sweetpod" => sweetpod(spot),
        "shellback" => shellback(spot),
        "silkreed" => silkreed(spot),
        "barkweave" => barkweave(spot),
        "spinegrass" => spinegrass(spot),
        "glassvine" => glassvine(spot),
        "basalt" => basalt(spot),
        "limestone" => limestone(spot),
        "glassrock" => glassrock(spot),
        "sunstone" => sunstone(spot),
        "verdigris" => verdigris(spot),
        "rustiron" => rustiron(spot),
        "cobalt_glass" => cobalt_glass(spot),
        "singing_quartz" => singing_quartz(spot),
        "glyph_stones" => glyph_stones(spot),
        "dormant_engine" => dormant_engine(spot),
        "crystal_archive" => crystal_archive(spot),
        _ => return false,
    }
    true
}

// Foods.

/// A dark shrub hung with pulsing lantern fruit.
fn glowfruit(p: &Spot) {
    let r = p.s * 0.42;
    draw_circle(p.c.x, p.c.y, r, rgb(0.14, 0.26, 0.16));
    draw_circle(
        p.c.x + r * 0.3,
        p.c.y - r * 0.25,
        r * 0.6,
        rgb(0.19, 0.34, 0.2),
    );
    for i in 0..p.count(1.0, 6.0) {
        let at = p.scatter(i as u32 + 3, 0.32);
        let pulse = 0.75 + 0.25 * (p.time * 1.6 + i as f32 * 1.7).sin();
        draw_circle(at.x, at.y, 9.0, with_alpha(p.color, 0.16 * pulse));
        line(
            at - vec2(0.0, 5.0),
            at - vec2(0.0, 2.0),
            1.0,
            rgb(0.3, 0.42, 0.24),
        );
        draw_ellipse(at.x, at.y + 1.0, 3.0, 4.0, 0.0, with_alpha(p.color, pulse));
        draw_circle(at.x - 1.0, at.y, 1.2, lighten(p.color, 0.35));
    }
}

/// A soil mound with leafy tops and orange tubers breaking the surface.
fn ember_tuber(p: &Spot) {
    let r = p.s * 0.4;
    p.shadow(r);
    let soil = rgb(0.33, 0.22, 0.14);
    draw_ellipse(p.c.x, p.c.y + 2.0, r, r * 0.7, 0.0, soil);
    draw_ellipse(
        p.c.x - 2.0,
        p.c.y,
        r * 0.75,
        r * 0.5,
        0.0,
        lighten(soil, 0.06),
    );
    for i in 0..p.count(1.0, 5.0) {
        let at = p.scatter(i as u32 + 5, 0.24);
        let tilt = (p.h(i as u32 + 40) - 0.5) * 70.0;
        draw_ellipse(at.x, at.y, 5.5, 3.5, tilt, p.color);
        draw_ellipse(
            at.x - 1.0,
            at.y - 1.0,
            3.0,
            1.5,
            tilt,
            lighten(p.color, 0.2),
        );
    }
    for i in 0..3 {
        let base = p.c + vec2((i as f32 - 1.0) * r * 0.5, -r * 0.2);
        let sway = (p.time * 1.1 + i as f32).sin() * 1.5;
        let leaf = rgb(0.32, 0.55, 0.26);
        draw_triangle(
            base,
            base + vec2(-5.0 + sway, -12.0),
            base + vec2(1.0 + sway, -10.0),
            leaf,
        );
        draw_triangle(
            base,
            base + vec2(5.0 + sway, -11.0),
            base + vec2(-1.0 + sway, -9.0),
            darken(leaf, 0.15),
        );
    }
}

/// A cluster of pale fungus caps on slender stems, each with a veil skirt.
fn veilcap(p: &Spot) {
    let mut caps: Vec<(Vec2, f32)> = (0..p.count(1.0, 5.0))
        .map(|i| (p.scatter(i as u32 + 7, 0.3), 4.0 + p.h(i as u32 + 60) * 5.0))
        .collect();
    caps.sort_by(|a, b| a.0.y.total_cmp(&b.0.y));
    for (at, size) in caps {
        let stem = lighten(p.color, 0.15);
        draw_ellipse(
            at.x + 2.0,
            at.y + size * 1.4,
            size * 0.9,
            size * 0.35,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.25),
        );
        draw_rectangle(at.x - size * 0.18, at.y, size * 0.36, size * 1.4, stem);
        draw_ellipse(
            at.x,
            at.y + size * 0.55,
            size * 0.55,
            size * 0.2,
            0.0,
            with_alpha(stem, 0.7),
        );
        draw_ellipse(at.x, at.y, size, size * 0.62, 0.0, darken(p.color, 0.18));
        draw_ellipse(
            at.x,
            at.y - size * 0.12,
            size * 0.92,
            size * 0.5,
            0.0,
            p.color,
        );
        draw_circle(
            at.x - size * 0.35,
            at.y - size * 0.25,
            size * 0.16,
            lighten(p.color, 0.3),
        );
    }
}

/// Tall reed stalks bowing under heavy grain heads.
fn reedgrain(p: &Spot) {
    for i in 0..p.count(3.0, 9.0) {
        let ox = (p.h(i as u32) - 0.5) * p.s * 0.8;
        let base = p.c + vec2(ox, p.s * 0.35);
        let sway = (p.time * 1.2 + i as f32 * 0.7).sin() * 2.5;
        let tip = base + vec2(sway + 4.0, -p.s * (0.55 + p.h(i as u32 + 9) * 0.25));
        line(base, tip, 1.6, rgb(0.5, 0.58, 0.32));
        let lean = (tip.x - base.x).atan2(base.y - tip.y).to_degrees();
        draw_ellipse(tip.x + 1.5, tip.y + 3.0, 2.2, 6.0, lean + 15.0, p.color);
        draw_ellipse(
            tip.x + 1.0,
            tip.y + 2.0,
            1.0,
            4.0,
            lean + 15.0,
            lighten(p.color, 0.18),
        );
    }
}

/// A low rosette of broad pointed leaves with pale veins.
fn bitterleaf(p: &Spot) {
    let leaves = p.count(3.0, 8.0);
    let r = p.s * (0.25 + p.fullness * 0.18);
    let turn = p.h(2) * TAU;
    for i in 0..leaves {
        let a = turn + i as f32 / leaves as f32 * TAU;
        let dir = vec2(a.cos(), a.sin());
        let side = vec2(-dir.y, dir.x) * r * 0.32;
        let tip = p.c + dir * r;
        let mid = p.c + dir * r * 0.45;
        let shade = if i % 2 == 0 {
            p.color
        } else {
            darken(p.color, 0.14)
        };
        draw_triangle(p.c, mid + side, tip, shade);
        draw_triangle(p.c, mid - side, tip, darken(shade, 0.1));
        line(p.c, tip, 1.0, lighten(p.color, 0.3));
    }
    draw_circle(p.c.x, p.c.y, 3.0, darken(p.color, 0.3));
}

/// A curling vine hung with plump seed pods.
fn sweetpod(p: &Spot) {
    let vine = rgb(0.3, 0.48, 0.26);
    let turns = p.count(2.0, 4.0);
    let mut last = p.c + vec2(-p.s * 0.35, p.s * 0.2);
    for i in 1..=turns * 4 {
        let t = i as f32 / (turns * 4) as f32;
        let next = p.c
            + vec2(
                -p.s * 0.35 + t * p.s * 0.7,
                p.s * 0.2 - t * p.s * 0.3 + (t * 9.0 + p.h(1) * 3.0).sin() * p.s * 0.15,
            );
        line(last, next, 2.0, vine);
        if i % 3 == 0 {
            let sway = (p.time * 1.4 + i as f32).sin() * 6.0;
            draw_ellipse(next.x, next.y + 5.0, 2.8, 6.0, sway, p.color);
            draw_ellipse(
                next.x - 0.8,
                next.y + 3.5,
                1.0,
                3.0,
                sway,
                lighten(p.color, 0.3),
            );
        } else if i % 3 == 1 {
            draw_ellipse(next.x, next.y - 3.0, 3.5, 2.0, 30.0, lighten(vine, 0.1));
        }
        last = next;
    }
}

/// Ridged shells huddled on a damp patch of sand.
fn shellback(p: &Spot) {
    let r = p.s * 0.42;
    draw_ellipse(p.c.x, p.c.y, r, r * 0.7, 0.0, rgb(0.5, 0.47, 0.38));
    for i in 0..p.count(1.0, 5.0) {
        let at = p.scatter(i as u32 + 11, 0.26);
        let size = 5.0 + p.h(i as u32 + 30) * 3.0;
        let tilt = (p.h(i as u32 + 50) - 0.5) * 60.0;
        draw_ellipse(
            at.x + 1.5,
            at.y + 2.0,
            size,
            size * 0.7,
            tilt,
            Color::new(0.0, 0.0, 0.0, 0.25),
        );
        draw_ellipse(at.x, at.y, size, size * 0.72, tilt, darken(p.color, 0.12));
        draw_ellipse(at.x, at.y - 1.0, size * 0.85, size * 0.55, tilt, p.color);
        for ridge in -1..=1 {
            let a = (tilt.to_radians()) + ridge as f32 * 0.5 - std::f32::consts::FRAC_PI_2;
            let end = at + vec2(a.cos(), a.sin()) * size * 0.6;
            line(at + vec2(0.0, size * 0.25), end, 1.0, darken(p.color, 0.3));
        }
    }
}

// Fibres.

/// Fine pale strands that wave and end in fluffy tufts.
fn silkreed(p: &Spot) {
    for i in 0..p.count(4.0, 11.0) {
        let ox = (p.h(i as u32) - 0.5) * p.s * 0.75;
        let base = p.c + vec2(ox, p.s * 0.35);
        let height = p.s * (0.5 + p.h(i as u32 + 9) * 0.3);
        let sway = (p.time * 1.6 + i as f32 * 0.9).sin() * 4.0;
        let mid = base + vec2(sway * 0.4, -height * 0.5);
        let tip = base + vec2(sway, -height);
        let strand = darken(p.color, 0.25);
        line(base, mid, 1.2, strand);
        line(mid, tip, 1.2, strand);
        draw_circle(tip.x, tip.y, 3.2, with_alpha(p.color, 0.55));
        draw_circle(tip.x, tip.y, 1.8, p.color);
    }
}

/// A squat trunk whose bark peels into woven fibre strips.
fn barkweave(p: &Spot) {
    let r = p.s * (0.22 + p.fullness * 0.12);
    p.shadow(r * 1.1);
    let bark = p.color;
    draw_circle(p.c.x, p.c.y, r, darken(bark, 0.2));
    draw_circle(p.c.x, p.c.y, r * 0.78, lighten(bark, 0.12));
    draw_circle_lines(p.c.x, p.c.y, r * 0.5, 1.0, darken(bark, 0.15));
    draw_circle_lines(p.c.x, p.c.y, r * 0.25, 1.0, darken(bark, 0.15));
    for i in 0..p.count(2.0, 6.0) {
        let a = p.h(i as u32 + 4) * TAU;
        let dir = vec2(a.cos(), a.sin());
        let start = p.c + dir * r * 0.9;
        let curl = vec2(-dir.y, dir.x) * 4.0;
        let end = start + dir * p.s * 0.22 + curl;
        line(start, end, 3.0, darken(bark, 0.05));
        line(start + curl * 0.3, end, 1.0, lighten(bark, 0.25));
    }
}

/// A stiff clump of straight spines with sharp pale tips.
fn spinegrass(p: &Spot) {
    let base = p.c + vec2(0.0, p.s * 0.3);
    draw_ellipse(
        base.x,
        base.y,
        p.s * 0.25,
        p.s * 0.08,
        0.0,
        darken(p.color, 0.45),
    );
    let spines = p.count(5.0, 14.0);
    for i in 0..spines {
        let spread = (i as f32 / (spines - 1).max(1) as f32 - 0.5) * 2.2;
        let a = -std::f32::consts::FRAC_PI_2 + spread * 0.7 + (p.h(i as u32) - 0.5) * 0.2;
        let length = p.s * (0.35 + p.h(i as u32 + 20) * 0.3);
        let tip = base + vec2(a.cos(), a.sin()) * length;
        let foot = base + vec2(spread * 3.0, 0.0);
        line(foot, tip, 2.0, darken(p.color, 0.2));
        line(tip - (tip - foot) * 0.25, tip, 1.2, lighten(p.color, 0.3));
    }
}

/// Translucent vine coils that catch the light.
fn glassvine(p: &Spot) {
    let coils = p.count(1.0, 3.0);
    for coil in 0..coils {
        let centre = p.scatter(coil as u32 + 2, 0.18);
        let radius = p.s * (0.16 + p.h(coil as u32 + 70) * 0.1);
        let mut last = centre + vec2(radius, 0.0);
        for i in 1..=14 {
            let t = i as f32 / 14.0;
            let a = t * TAU * 1.4;
            let next = centre + vec2(a.cos(), a.sin()) * radius * (1.0 - t * 0.6);
            line(last, next, 2.5, with_alpha(p.color, 0.75));
            last = next;
        }
        let glint = 0.5 + 0.5 * (p.time * 2.2 + coil as f32 * 2.0).sin();
        let at = centre + vec2(-radius * 0.5, -radius * 0.6);
        draw_circle(at.x, at.y, 2.0, with_alpha(WHITE, 0.4 + glint * 0.5));
        let leaf = centre + vec2(radius * 0.8, radius * 0.4);
        draw_poly(
            leaf.x,
            leaf.y,
            4,
            3.5,
            45.0,
            with_alpha(lighten(p.color, 0.2), 0.8),
        );
    }
}

// Stone.

/// Hexagonal column tops packed together.
fn basalt(p: &Spot) {
    let r = p.s * (0.12 + p.fullness * 0.04);
    p.shadow(p.s * 0.38);
    let offsets = [
        vec2(0.0, 0.0),
        vec2(1.75, 0.0),
        vec2(-1.75, 0.0),
        vec2(0.88, -1.5),
        vec2(-0.88, -1.5),
        vec2(0.88, 1.5),
        vec2(-0.88, 1.5),
    ];
    let columns = p.count(3.0, 7.0) as usize;
    for (i, offset) in offsets.iter().take(columns).enumerate() {
        let at = p.c + *offset * r;
        let lift = p.h(i as u32 + 5) * 4.0;
        draw_poly(at.x, at.y + 3.0, 6, r, 30.0, darken(p.color, 0.3));
        draw_poly(
            at.x,
            at.y - lift,
            6,
            r * 0.95,
            30.0,
            lighten(p.color, 0.08 + lift * 0.02),
        );
        draw_poly_lines(
            at.x,
            at.y - lift,
            6,
            r * 0.95,
            30.0,
            1.0,
            darken(p.color, 0.25),
        );
    }
}

/// Flat pale slabs stacked in layers.
fn limestone(p: &Spot) {
    let layers = p.count(1.0, 4.0);
    let w = p.s * 0.75;
    p.shadow(w * 0.55);
    for i in 0..layers {
        let shrink = 1.0 - i as f32 * 0.17;
        let y = p.c.y + p.s * 0.18 - i as f32 * 7.0;
        let x = p.c.x - w * shrink * 0.5 + (p.h(i as u32) - 0.5) * 6.0;
        let slab_w = w * shrink;
        draw_rectangle(x, y, slab_w, 8.0, darken(p.color, 0.2));
        draw_rectangle(x, y, slab_w, 4.0, p.color);
        draw_line(
            x + slab_w * 0.3,
            y + 4.0,
            x + slab_w * 0.45,
            y + 8.0,
            1.0,
            darken(p.color, 0.35),
        );
    }
}

/// Dark obsidian shards with bright cutting edges.
fn glassrock(p: &Spot) {
    p.shadow(p.s * 0.35);
    for i in 0..p.count(2.0, 5.0) {
        let at = p.scatter(i as u32 + 3, 0.18);
        let a = p.h(i as u32 + 12) * TAU;
        let size = p.s * (0.18 + p.h(i as u32 + 33) * 0.12);
        let corner = |turn: f32, d: f32| at + vec2((a + turn).cos(), (a + turn).sin()) * size * d;
        draw_triangle(
            corner(0.0, 1.2),
            corner(2.3, 0.7),
            corner(4.0, 0.8),
            p.color,
        );
        draw_triangle(
            corner(0.0, 1.2),
            corner(4.0, 0.8),
            at,
            lighten(p.color, 0.12),
        );
        line(
            corner(0.0, 1.2),
            corner(2.3, 0.7),
            1.2,
            lighten(p.color, 0.55),
        );
    }
}

/// Smooth warm boulders flecked with glowing grains.
fn sunstone(p: &Spot) {
    let r = p.s * (0.25 + p.fullness * 0.15);
    p.shadow(r);
    draw_ellipse(
        p.c.x - r * 0.4,
        p.c.y + r * 0.25,
        r * 0.6,
        r * 0.45,
        0.0,
        darken(p.color, 0.15),
    );
    draw_ellipse(p.c.x, p.c.y, r, r * 0.8, 0.0, p.color);
    draw_ellipse(
        p.c.x - r * 0.25,
        p.c.y - r * 0.3,
        r * 0.4,
        r * 0.22,
        -20.0,
        lighten(p.color, 0.18),
    );
    for i in 0..5 {
        let at = p.c + vec2((p.h(i) - 0.5) * r * 1.3, (p.h(i + 9) - 0.5) * r);
        let pulse = 0.5 + 0.5 * (p.time * 1.5 + i as f32 * 1.3).sin();
        draw_circle(
            at.x,
            at.y,
            1.5,
            with_alpha(rgb(1.0, 0.92, 0.6), 0.4 + pulse * 0.6),
        );
    }
}

// Ores.

fn ore_rock(p: &Spot, rock: Color) -> f32 {
    let r = p.s * 0.4;
    p.shadow(r);
    draw_circle(p.c.x, p.c.y, r, rock);
    draw_circle(
        p.c.x + r * 0.35,
        p.c.y + r * 0.2,
        r * 0.55,
        darken(rock, 0.12),
    );
    draw_circle(
        p.c.x - r * 0.3,
        p.c.y - r * 0.35,
        r * 0.3,
        lighten(rock, 0.08),
    );
    r
}

/// A grey boulder crusted with green patina.
fn verdigris(p: &Spot) {
    let r = ore_rock(p, rgb(0.36, 0.33, 0.3));
    for i in 0..p.count(2.0, 7.0) {
        let at = p.scatter(i as u32 + 8, 0.26);
        let size = 3.0 + p.h(i as u32 + 18) * r * 0.25;
        draw_ellipse(at.x, at.y, size, size * 0.6, p.h(i as u32) * 180.0, p.color);
        draw_circle(at.x - 1.0, at.y - 1.0, size * 0.35, lighten(p.color, 0.25));
    }
}

/// A red-brown boulder streaked with rust runs.
fn rustiron(p: &Spot) {
    let r = ore_rock(p, rgb(0.3, 0.27, 0.27));
    for i in 0..p.count(2.0, 6.0) {
        let x = p.c.x + (p.h(i as u32 + 2) - 0.5) * r * 1.3;
        let top = p.c.y - r * (0.5 - p.h(i as u32 + 7) * 0.3);
        let length = r * (0.4 + p.h(i as u32 + 14) * 0.6);
        draw_line(x, top, x + 1.5, top + length, 3.0, p.color);
        draw_circle(x + 1.5, top + length, 2.0, darken(p.color, 0.15));
    }
    draw_circle(
        p.c.x - r * 0.2,
        p.c.y - r * 0.1,
        r * 0.25,
        with_alpha(p.color, 0.6),
    );
}

/// Dark rock split open by blue crystal prisms.
fn cobalt_glass(p: &Spot) {
    let r = ore_rock(p, rgb(0.2, 0.21, 0.25));
    for i in 0..p.count(2.0, 5.0) {
        let at = p.scatter(i as u32 + 6, 0.2);
        let tilt = (p.h(i as u32 + 3) - 0.5) * 80.0;
        let length = r * (0.35 + p.h(i as u32 + 21) * 0.3);
        let pulse = 0.75 + 0.25 * (p.time * 2.0 + i as f32).sin();
        draw_rectangle_ex(
            at.x,
            at.y,
            5.0,
            length,
            DrawRectangleParams {
                offset: vec2(0.5, 1.0),
                rotation: tilt.to_radians(),
                color: with_alpha(p.color, pulse),
            },
        );
        let tip = at + vec2(tilt.to_radians().sin(), -tilt.to_radians().cos()) * length;
        draw_poly(tip.x, tip.y, 6, 3.0, tilt, lighten(p.color, 0.3));
    }
}

/// Tall pale crystal points that hum in slow rings.
fn singing_quartz(p: &Spot) {
    let r = p.s * 0.36;
    p.shadow(r);
    let ring = (p.time * 0.6).fract();
    draw_circle_lines(
        p.c.x,
        p.c.y,
        r * (0.6 + ring * 0.9),
        1.5,
        with_alpha(p.color, 0.5 * (1.0 - ring)),
    );
    draw_ellipse(
        p.c.x,
        p.c.y + r * 0.3,
        r,
        r * 0.45,
        0.0,
        rgb(0.3, 0.3, 0.34),
    );
    let points = p.count(2.0, 6.0);
    for i in 0..points {
        let spread = (i as f32 / (points - 1).max(1) as f32 - 0.5) * 2.0;
        let base = p.c + vec2(spread * r * 0.6, r * 0.3);
        let height = r * (1.0 + p.h(i as u32 + 5) * 0.7) * (1.0 - spread.abs() * 0.35);
        let tip = base + vec2(spread * 5.0, -height);
        draw_triangle(base - vec2(4.0, 0.0), base + vec2(4.0, 0.0), tip, p.color);
        draw_triangle(base, base + vec2(4.0, 0.0), tip, darken(p.color, 0.15));
        line(base - vec2(2.0, 0.0), tip, 1.0, lighten(p.color, 0.4));
    }
}

// Ruins.

fn ruin_glow(p: &Spot, color: Color) {
    let glow = 0.22 + 0.12 * (p.time * 0.8).sin();
    draw_circle(
        p.c.x,
        p.c.y,
        p.s * 0.9,
        with_alpha(color, glow * p.fullness),
    );
}

/// Leaning standing stones cut with glowing glyphs.
fn glyph_stones(p: &Spot) {
    ruin_glow(p, rgb(0.6, 0.45, 0.9));
    let stone = rgb(0.4, 0.37, 0.46);
    for i in 0..3 {
        let x = p.c.x - p.s * 0.36 + i as f32 * p.s * 0.36;
        let h = p.s * (0.5 + p.h(i) * 0.45);
        let top = p.c.y + p.s * 0.3 - h;
        draw_rectangle(x - 5.0, top + 3.0, 10.0, h - 3.0, stone);
        draw_circle(x, top + 4.0, 5.0, stone);
        draw_rectangle(x - 5.0, top + 3.0, 3.0, h - 3.0, lighten(stone, 0.08));
        let pulse = 0.55 + 0.45 * (p.time * 1.2 + i as f32 * 2.1).sin();
        let carving = with_alpha(p.color, pulse);
        let y = top + h * 0.35;
        match (p.h(i + 20) * 3.0) as i32 {
            0 => draw_circle_lines(x, y, 3.0, 1.2, carving),
            1 => draw_poly_lines(x, y, 3, 3.5, 90.0, 1.2, carving),
            _ => {
                draw_line(x - 3.0, y - 3.0, x + 3.0, y + 3.0, 1.2, carving);
                draw_line(x + 3.0, y - 3.0, x - 3.0, y + 3.0, 1.2, carving);
            }
        }
        draw_line(x - 2.5, y + 7.0, x + 2.5, y + 7.0, 1.2, carving);
    }
}

/// A half-buried alien machine, its turbine still and its core barely awake.
fn dormant_engine(p: &Spot) {
    ruin_glow(p, p.color);
    let r = p.s * 0.45;
    p.shadow(r);
    let metal = darken(p.color, 0.25);
    draw_poly(p.c.x, p.c.y, 8, r, 22.5, metal);
    draw_poly_lines(p.c.x, p.c.y, 8, r, 22.5, 2.0, lighten(metal, 0.2));
    draw_circle(p.c.x, p.c.y, r * 0.68, darken(metal, 0.35));
    let turn = p.time * 0.15 + p.h(1) * TAU;
    for blade in 0..6 {
        let a = turn + blade as f32 / 6.0 * TAU;
        let tip = p.c + vec2(a.cos(), a.sin()) * r * 0.62;
        let side = p.c + vec2((a + 0.45).cos(), (a + 0.45).sin()) * r * 0.4;
        draw_triangle(p.c, tip, side, p.color);
    }
    let pulse = 0.4 + 0.6 * ((p.time * 0.7).sin() * 0.5 + 0.5);
    draw_circle(p.c.x, p.c.y, r * 0.22, darken(metal, 0.2));
    draw_circle(
        p.c.x,
        p.c.y,
        r * 0.13,
        with_alpha(rgb(0.55, 0.85, 1.0), pulse * p.fullness.max(0.3)),
    );
    for bolt in 0..4 {
        let a = bolt as f32 / 4.0 * TAU + 0.4;
        let at = p.c + vec2(a.cos(), a.sin()) * r * 0.84;
        draw_circle(at.x, at.y, 1.8, lighten(metal, 0.35));
    }
}

/// Crystal plinths with shards of stored light floating above them.
fn crystal_archive(p: &Spot) {
    ruin_glow(p, p.color);
    let plinth = rgb(0.32, 0.36, 0.42);
    draw_rectangle(
        p.c.x - p.s * 0.4,
        p.c.y + p.s * 0.12,
        p.s * 0.8,
        p.s * 0.2,
        plinth,
    );
    draw_rectangle(
        p.c.x - p.s * 0.4,
        p.c.y + p.s * 0.12,
        p.s * 0.8,
        3.0,
        lighten(plinth, 0.15),
    );
    for i in 0..3 {
        let x = p.c.x + (i as f32 - 1.0) * p.s * 0.26;
        let h = p.s * (0.3 + p.h(i) * 0.25);
        let base = vec2(x, p.c.y + p.s * 0.12);
        draw_triangle(
            base - vec2(5.0, 0.0),
            base + vec2(5.0, 0.0),
            base - vec2(0.0, h),
            p.color,
        );
        draw_triangle(
            base,
            base + vec2(5.0, 0.0),
            base - vec2(0.0, h),
            darken(p.color, 0.18),
        );
    }
    for i in 0..p.count(1.0, 4.0) {
        let bob = (p.time * 1.3 + i as f32 * 1.6).sin() * 3.0;
        let at = p.c + vec2((p.h(i as u32 + 30) - 0.5) * p.s * 0.6, -p.s * 0.45 + bob);
        draw_poly(
            at.x,
            at.y,
            4,
            3.5,
            45.0 + p.time * 20.0,
            with_alpha(lighten(p.color, 0.3), 0.85),
        );
    }
}
