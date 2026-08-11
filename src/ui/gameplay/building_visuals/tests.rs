use super::*;

#[test]
fn test_building_outline_style_prioritizes_hover_over_assignment() {
    let hovered = building_outline_style(true, Some(style::BAR_GREEN)).unwrap();
    let assigned = building_outline_style(false, Some(style::BAR_GREEN)).unwrap();

    assert_eq!(hovered.1, 3.0);
    assert_eq!(assigned.1, 2.0);
    assert!(hovered.0.r > assigned.0.r);
    assert!(building_outline_style(false, None).is_none());
}

#[test]
fn test_assignment_marker_with_filter_adds_filter_without_replacing_assignment() {
    let filtered = assignment_marker_with_filter(None, true).unwrap();
    assert_eq!(filtered.0, "FILTER");
    assert_eq!(filtered.1.r, style::ACCENT_GOLD.r);
    assert_eq!(filtered.1.g, style::ACCENT_GOLD.g);
    assert_eq!(filtered.1.b, style::ACCENT_GOLD.b);

    let assigned = assignment_marker_with_filter(Some(("HOME", style::BAR_GREEN)), true)
        .expect("assignment marker should remain visible");
    assert_eq!(assigned.0, "HOME");
    assert_eq!(assigned.1.r, style::BAR_GREEN.r);
    assert!(assignment_marker_with_filter(None, false).is_none());
}

#[test]
fn test_assign_filter_outline_uses_gold_room_highlight() {
    let filtered =
        building_outline_style_for_assign_filter(true, Some(style::BAR_GREEN), true).unwrap();
    assert_eq!(filtered.0.r, style::ACCENT_GOLD.r);
    assert_eq!(filtered.0.g, style::ACCENT_GOLD.g);
    assert_eq!(filtered.0.b, style::ACCENT_GOLD.b);
    assert_eq!(filtered.1, 3.0);

    let assigned =
        building_outline_style_for_assign_filter(false, Some(style::BAR_GREEN), false).unwrap();
    assert_eq!(assigned.1, 2.0);
    assert!(building_outline_style_for_assign_filter(false, None, false).is_none());
}
