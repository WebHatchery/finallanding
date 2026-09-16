//! menu domain.

use macroquad::prelude::Rect;

pub fn menu_start_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(screen_width * 0.5 - 100.0, screen_height * 0.5, 200.0, 50.0)
}

pub fn restart_button_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 90.0,
        screen_height * 0.5 + 48.0,
        180.0,
        38.0,
    )
}
