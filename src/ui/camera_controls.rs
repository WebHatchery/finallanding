//! camera controls.

use crate::ui::hit_zones::{camera_control_button_rect, CameraAction};
use crate::ui::style;
use macroquad::prelude::Rect;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_camera_controls(area: Rect, zoom: f32) {
    if area.h < 120.0 {
        return;
    }
    let panel = crate::ui::hit_zones::camera_control_rect(area);
    style::draw_deep_panel(panel);
    for action in [
        CameraAction::ZoomOut,
        CameraAction::Recenter,
        CameraAction::ZoomIn,
    ] {
        let rect = camera_control_button_rect(area, action);
        style::draw_button(rect, false, style::button_hovered(rect));
        let label = match action {
            CameraAction::ZoomOut => "−",
            CameraAction::Recenter => "",
            CameraAction::ZoomIn => "+",
        };
        let label = if matches!(action, CameraAction::Recenter) {
            format!("{:.0}%", zoom * 100.0)
        } else {
            label.to_string()
        };
        draw_ui_text(
            &label,
            rect.x + 12.0,
            rect.y + 22.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
    }
}
