//! hit zones domain.

use macroquad::prelude::Rect;

/// Expand an interaction target without changing its drawn footprint.
pub fn touch_target(rect: Rect) -> Rect {
    let width = rect.w.max(44.0);
    let height = rect.h.max(44.0);
    Rect::new(
        rect.x - (width - rect.w) * 0.5,
        rect.y - (height - rect.h) * 0.5,
        width,
        height,
    )
}

/// Expand only the vertical band for controls in a tight horizontal row.
pub fn touch_target_vertical(rect: Rect) -> Rect {
    let height = rect.h.max(44.0);
    Rect::new(rect.x, rect.y - (height - rect.h) * 0.5, rect.w, height)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAction {
    Previous,
    Next,
}

pub mod assign;
pub mod camera;
pub mod log;
pub mod menu;
pub mod toolbar;
pub mod top_bar;

pub use assign::*;
pub use camera::*;
pub use log::*;
pub use menu::*;
pub use toolbar::*;
pub use top_bar::*;
