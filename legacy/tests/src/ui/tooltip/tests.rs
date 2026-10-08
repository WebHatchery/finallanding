#[test]
fn oversized_tooltip_stays_inside_narrow_offset_bounds() {
    let bounds = Rect::new(25.0, 40.0, 120.0, 32.0);
    let rect = tooltip_rect(vec2(500.0, 800.0), bounds, 320.0, 44.0);
    assert_eq!(rect, bounds);
}

#[test]
fn test_tooltip_rect_clamps_inside_bounds() {
    let bounds = Rect::new(300.0, 66.0, 680.0, 568.0);
    let rect = tooltip_rect(vec2(970.0, 630.0), bounds, 160.0, 44.0);

    assert_eq!(rect.x, 820.0);
    assert_eq!(rect.y, 590.0);
    assert_eq!(rect.w, 160.0);
    assert_eq!(rect.h, 44.0);
}
