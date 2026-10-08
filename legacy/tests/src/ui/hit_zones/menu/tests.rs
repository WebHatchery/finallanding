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

#[test]
fn test_menu_start_and_continue_rows_do_not_overlap() {
    let start = menu_start_rect(720.0, 480.0);
    let continue_button = menu_continue_rect(720.0, 480.0);

    assert!(!start.overlaps(&continue_button));
    assert!(start.contains(vec2(360.0, 230.0)));
    assert!(continue_button.contains(vec2(360.0, 284.0)));
}

#[test]
fn test_result_actions_have_distinct_touch_rows() {
    let review = result_review_rect(720.0, 480.0);
    let restart = restart_button_rect(720.0, 480.0);
    let menu = result_menu_rect(720.0, 480.0);

    assert!(!review.overlaps(&restart));
    assert!(!restart.overlaps(&menu));
}
