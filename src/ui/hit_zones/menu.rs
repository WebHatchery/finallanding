//! menu domain.

use macroquad::prelude::Rect;

pub fn menu_start_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 130.0,
        screen_height * 0.5 - 10.0,
        260.0,
        44.0,
    )
}

pub fn menu_continue_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 130.0,
        screen_height * 0.5 + 44.0,
        260.0,
        44.0,
    )
}

pub fn menu_settings_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 130.0,
        screen_height * 0.5 + 98.0,
        126.0,
        44.0,
    )
}

pub fn menu_exit_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 + 4.0,
        screen_height * 0.5 + 98.0,
        126.0,
        44.0,
    )
}

pub fn menu_settings_close_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 + 116.0,
        screen_height * 0.5 - 54.0,
        58.0,
        36.0,
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

pub fn help_panel_rect(screen_width: f32, screen_height: f32) -> Rect {
    let width = (screen_width - 24.0).clamp(320.0, 700.0);
    let height = 330.0_f32.min(screen_height - 24.0);
    Rect::new(
        (screen_width - width) * 0.5,
        (screen_height - height) * 0.5,
        width,
        height,
    )
}

pub fn help_close_rect(screen_width: f32, screen_height: f32) -> Rect {
    let panel = help_panel_rect(screen_width, screen_height);
    Rect::new(
        panel.x + panel.w * 0.5 - 100.0,
        panel.bottom() - 54.0,
        200.0,
        44.0,
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

pub fn result_review_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 90.0,
        screen_height * 0.5 - 8.0,
        180.0,
        38.0,
    )
}

pub fn result_menu_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 90.0,
        screen_height * 0.5 + 104.0,
        180.0,
        38.0,
    )
}

pub fn result_back_rect(screen_width: f32, _screen_height: f32) -> Rect {
    Rect::new(screen_width - 196.0, 18.0, 176.0, 40.0)
}
