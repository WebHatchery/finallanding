//! menu domain.

use macroquad::prelude::Rect;

pub fn menu_start_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(screen_width * 0.5 - 100.0, screen_height * 0.5, 200.0, 50.0)
}

pub fn menu_continue_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(screen_width * 0.5 - 110.0, screen_height * 0.5, 220.0, 44.0)
}

pub fn menu_settings_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 110.0,
        screen_height * 0.5 + 52.0,
        220.0,
        44.0,
    )
}

pub fn menu_exit_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 110.0,
        screen_height * 0.5 + 104.0,
        220.0,
        44.0,
    )
}

pub fn introduction_continue_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 100.0,
        screen_height * 0.5 + 98.0,
        200.0,
        48.0,
    )
}

pub fn restart_button_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 90.0,
        screen_height * 0.5 + 48.0,
        180.0,
        38.0,
    )
}
