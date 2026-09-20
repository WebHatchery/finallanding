//! camera control hit zones.

use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, HitTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraAction {
    ZoomOut,
    Recenter,
    ZoomIn,
}

pub fn camera_control_rect(area: Rect) -> Rect {
    Rect::new(area.x + area.w - 154.0, area.y + 58.0, 142.0, 32.0)
}

pub fn camera_control_button_rect(area: Rect, action: CameraAction) -> Rect {
    let panel = camera_control_rect(area);
    let (offset, width) = match action {
        CameraAction::ZoomOut => (0.0, 34.0),
        CameraAction::Recenter => (39.0, 64.0),
        CameraAction::ZoomIn => (108.0, 34.0),
    };
    Rect::new(panel.x + offset, panel.y, width, panel.h)
}

pub fn camera_action_at(area: Rect, x: f32, y: f32) -> Option<CameraAction> {
    if area.h < 120.0 {
        return None;
    }
    hit_test(
        [
            CameraAction::ZoomOut,
            CameraAction::Recenter,
            CameraAction::ZoomIn,
        ]
        .into_iter()
        .map(|action| HitTarget::new(camera_control_button_rect(area, action), action)),
        vec2(x, y),
    )
}
