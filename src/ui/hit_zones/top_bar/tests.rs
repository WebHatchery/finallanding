use super::*;

fn center(rect: Rect) -> (f32, f32) {
    (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
}

#[test]
fn test_top_bar_speed_hit_zones_match_visible_buttons() {
    let (x, y) = center(speed_button_rect(1));

    assert_eq!(top_bar_speed_at(x, y), Some(TimeSpeed::Normal));
    assert_eq!(top_bar_speed_at(10.0, y), None);
}

#[test]
fn test_top_bar_priority_hit_zones_match_visible_buttons() {
    let (x, y) = center(priority_button_rect(2));

    assert_eq!(top_bar_priority_at(x, y), Some(ColonyPriority::Survey));
}
