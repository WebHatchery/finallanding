//! top bar domain.

use crate::data::game_state::TimeSpeed;
use crate::data::priority::ColonyPriority;
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, HitTarget};

use crate::ui::Layout;

pub const TOP_BAR_BUTTON_Y: f32 = 10.0;
pub const TOP_BAR_BUTTON_H: f32 = 30.0;
pub const SPEED_BUTTON_W: f32 = 50.0;
pub const SPEED_BUTTON_START_X: f32 = 300.0;
pub const PRIORITY_LABEL_X: f32 = 850.0;
pub const PRIORITY_BUTTON_W: f32 = 68.0;
pub const PRIORITY_BUTTON_START_X: f32 = 915.0;
pub const BUTTON_GAP: f32 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopBarAction {
    Undo,
    Cancel,
    Menu,
}

pub fn top_bar_action_rect(layout: &Layout, action: TopBarAction) -> Rect {
    let phone = layout.viewport_width < 760.0;
    let width = if phone { 54.0 } else { 70.0 };
    let y = if phone { 32.0 } else { TOP_BAR_BUTTON_Y };
    let height = if phone { 20.0 } else { TOP_BAR_BUTTON_H };
    let x = match action {
        TopBarAction::Undo => layout.viewport_width - width * 3.0 - BUTTON_GAP * 2.0 - 10.0,
        TopBarAction::Cancel => layout.viewport_width - width * 2.0 - BUTTON_GAP - 10.0,
        TopBarAction::Menu => layout.viewport_width - width - 10.0,
    };
    Rect::new(x.max(8.0), y, width, height)
}

pub fn top_bar_action_at(layout: &Layout, x: f32, y: f32) -> Option<TopBarAction> {
    hit_test(
        [TopBarAction::Undo, TopBarAction::Cancel, TopBarAction::Menu]
            .into_iter()
            .map(|action| HitTarget::new(top_bar_action_rect(layout, action), action)),
        vec2(x, y),
    )
}

pub fn speed_button_rect(index: usize) -> Rect {
    Rect::new(
        SPEED_BUTTON_START_X + index as f32 * (SPEED_BUTTON_W + BUTTON_GAP),
        TOP_BAR_BUTTON_Y,
        SPEED_BUTTON_W,
        TOP_BAR_BUTTON_H,
    )
}

pub fn speed_button_rect_for(layout: &Layout, index: usize) -> Rect {
    if layout.viewport_width < 760.0 {
        Rect::new(8.0 + index as f32 * 39.0, 32.0, 36.0, 20.0)
    } else if layout.viewport_width < 1_100.0 {
        Rect::new(
            230.0 + index as f32 * 47.0,
            TOP_BAR_BUTTON_Y,
            44.0,
            TOP_BAR_BUTTON_H,
        )
    } else {
        speed_button_rect(index)
    }
}

pub fn priority_button_rect(index: usize) -> Rect {
    Rect::new(
        PRIORITY_BUTTON_START_X + index as f32 * (PRIORITY_BUTTON_W + BUTTON_GAP),
        TOP_BAR_BUTTON_Y,
        PRIORITY_BUTTON_W,
        TOP_BAR_BUTTON_H,
    )
}

pub fn priority_button_rect_for(layout: &Layout, index: usize) -> Rect {
    if layout.viewport_width < 760.0 {
        Rect::new(-100.0, -100.0, 1.0, 1.0)
    } else if layout.viewport_width < 1_100.0 {
        Rect::new(
            545.0 + index as f32 * 60.0,
            TOP_BAR_BUTTON_Y,
            55.0,
            TOP_BAR_BUTTON_H,
        )
    } else {
        priority_button_rect(index)
    }
}

pub fn top_bar_speed_at(x: f32, y: f32) -> Option<TimeSpeed> {
    let speeds = [
        TimeSpeed::Paused,
        TimeSpeed::Normal,
        TimeSpeed::Fast,
        TimeSpeed::SuperFast,
    ];
    hit_test(
        speeds
            .into_iter()
            .enumerate()
            .map(|(index, speed)| HitTarget::new(speed_button_rect(index), speed)),
        vec2(x, y),
    )
}

pub fn top_bar_speed_at_for(layout: &Layout, x: f32, y: f32) -> Option<TimeSpeed> {
    let speeds = [
        TimeSpeed::Paused,
        TimeSpeed::Normal,
        TimeSpeed::Fast,
        TimeSpeed::SuperFast,
    ];
    hit_test(
        speeds
            .into_iter()
            .enumerate()
            .map(|(index, speed)| HitTarget::new(speed_button_rect_for(layout, index), speed)),
        vec2(x, y),
    )
}

pub fn top_bar_priority_at(x: f32, y: f32) -> Option<ColonyPriority> {
    hit_test(
        ColonyPriority::all()
            .iter()
            .enumerate()
            .map(|(index, priority)| HitTarget::new(priority_button_rect(index), *priority)),
        vec2(x, y),
    )
}

pub fn top_bar_priority_at_for(layout: &Layout, x: f32, y: f32) -> Option<ColonyPriority> {
    hit_test(
        ColonyPriority::all()
            .iter()
            .enumerate()
            .map(|(index, priority)| {
                HitTarget::new(priority_button_rect_for(layout, index), *priority)
            }),
        vec2(x, y),
    )
}
