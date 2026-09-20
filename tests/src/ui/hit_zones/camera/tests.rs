fn center(rect: Rect) -> (f32, f32) {
    (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
}

#[test]
fn test_camera_controls_are_distinct_and_visible_for_world_area() {
    let area = Rect::new(12.0, 78.0, 1256.0, 390.0);
    let zoom_out = camera_control_button_rect(area, CameraAction::ZoomOut);
    let recenter = camera_control_button_rect(area, CameraAction::Recenter);
    let zoom_in = camera_control_button_rect(area, CameraAction::ZoomIn);

    assert!(!zoom_out.overlaps(&recenter));
    assert!(!recenter.overlaps(&zoom_in));
    assert_eq!(
        camera_action_at(area, center(zoom_out).0, center(zoom_out).1),
        Some(CameraAction::ZoomOut)
    );
    assert_eq!(
        camera_action_at(area, center(zoom_in).0, center(zoom_in).1),
        Some(CameraAction::ZoomIn)
    );
}

#[test]
fn test_camera_controls_hide_when_context_leaves_no_observation_space() {
    let area = Rect::new(7.0, 65.0, 706.0, 64.0);
    let button = camera_control_button_rect(area, CameraAction::ZoomIn);

    assert_eq!(
        camera_action_at(area, center(button).0, center(button).1),
        None
    );
}
