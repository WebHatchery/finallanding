use super::*;

#[test]
fn test_iso_projection_round_trips_grid_cell() {
    let view = IsoView::for_area(Rect::new(300.0, 66.0, 680.0, 568.0), 20, 20);
    let position = Position::new(6, 9);
    let screen = view.grid_to_screen(position);

    assert_eq!(
        view.screen_to_grid(screen + vec2(0.0, view.tile_h * 0.25)),
        position
    );
}

#[test]
fn test_iso_diamond_points_keep_expected_order() {
    let points = iso_diamond_points(vec2(10.0, 20.0), 40.0, 20.0);

    assert_eq!(points[0], vec2(10.0, 20.0));
    assert_eq!(points[1], vec2(30.0, 30.0));
    assert_eq!(points[2], vec2(10.0, 40.0));
    assert_eq!(points[3], vec2(-10.0, 30.0));
}
