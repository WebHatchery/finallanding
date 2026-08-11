use super::*;
use macroquad::prelude::vec2;
use macroquad_toolkit::input::rect_contains_point;

#[test]
fn test_menu_start_rect_contains_button_center() {
    let rect = menu_start_rect(1280.0, 720.0);

    assert!(rect_contains_point(rect, vec2(640.0, 385.0)));
}

#[test]
fn test_restart_rect_contains_button_center() {
    let rect = restart_button_rect(1280.0, 720.0);

    assert!(rect_contains_point(rect, vec2(640.0, 427.0)));
}
