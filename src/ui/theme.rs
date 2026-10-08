//! Colours, type scale and spacing for the 1920×1080 interface.

use crate::colony::chronicle::Category;
use crate::data::{Branch, Resource};
use macroquad::prelude::Color;

pub const VIRTUAL_WIDTH: f32 = 1920.0;
pub const VIRTUAL_HEIGHT: f32 = 1080.0;
pub const TOP_BAR_HEIGHT: f32 = 56.0;
pub const TOOLBAR_HEIGHT: f32 = 76.0;
pub const INSPECTOR_WIDTH: f32 = 470.0;
pub const PAD: f32 = 16.0;

pub const TEXT_TITLE: f32 = 64.0;
pub const TEXT_HEADING: f32 = 30.0;
pub const TEXT_LARGE: f32 = 24.0;
pub const TEXT_BODY: f32 = 20.0;
pub const TEXT_SMALL: f32 = 17.0;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

pub const BACKGROUND: Color = rgb(10, 12, 14);
pub const PANEL: Color = rgba(18, 22, 26, 0.94);
pub const PANEL_SOLID: Color = rgb(20, 24, 29);
pub const PANEL_RAISED: Color = rgb(30, 36, 42);
pub const PANEL_HOVER: Color = rgb(40, 48, 56);
pub const LINE: Color = rgba(140, 160, 170, 0.22);
pub const LINE_STRONG: Color = rgba(170, 190, 200, 0.45);
pub const TEXT: Color = rgb(222, 226, 222);
pub const TEXT_DIM: Color = rgb(150, 160, 160);
pub const TEXT_FAINT: Color = rgb(105, 115, 118);
pub const ACCENT: Color = rgb(232, 166, 78);
pub const ACCENT_DIM: Color = rgb(150, 104, 48);
pub const CYAN: Color = rgb(96, 196, 210);
pub const GOOD: Color = rgb(116, 196, 120);
pub const WARN: Color = rgb(232, 190, 80);
pub const BAD: Color = rgb(226, 92, 80);
pub const ROMANCE: Color = rgb(236, 120, 170);
pub const SELECT: Color = rgb(255, 214, 120);

pub fn resource_color(resource: Resource) -> Color {
    match resource {
        Resource::Food => rgb(150, 200, 90),
        Resource::Meals => rgb(230, 170, 90),
        Resource::Salvage => rgb(150, 160, 175),
        Resource::Fibre => rgb(120, 170, 110),
        Resource::Stone => rgb(170, 160, 140),
        Resource::Metal => rgb(120, 170, 210),
        Resource::Components => rgb(110, 210, 200),
        Resource::Medicine => rgb(230, 110, 120),
        Resource::Relics => rgb(190, 140, 240),
    }
}

pub fn branch_color(branch: Branch) -> Color {
    match branch {
        Branch::Survival => rgb(220, 150, 80),
        Branch::Agronomy => rgb(130, 200, 100),
        Branch::Industry => rgb(120, 160, 210),
        Branch::Medicine => rgb(230, 110, 120),
        Branch::Society => rgb(230, 190, 100),
        Branch::Xenology => rgb(180, 130, 240),
    }
}

pub fn category_color(category: Category) -> Color {
    match category {
        Category::Story => ACCENT,
        Category::Relationship => ROMANCE,
        Category::Discovery => CYAN,
        Category::Danger => BAD,
        Category::Loss => rgb(170, 140, 160),
        Category::Life => GOOD,
        Category::Colony => rgb(170, 180, 190),
        Category::Mind => rgb(200, 150, 230),
    }
}

/// Green for good, amber for middling, red for bad.
pub fn level_color(value: f32) -> Color {
    if value >= 60.0 {
        GOOD
    } else if value >= 30.0 {
        WARN
    } else {
        BAD
    }
}

pub fn opinion_color(opinion: f32) -> Color {
    if opinion >= 30.0 {
        GOOD
    } else if opinion <= -25.0 {
        BAD
    } else {
        TEXT_DIM
    }
}
