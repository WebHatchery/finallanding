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

#[test]
fn test_top_bar_help_action_has_a_distinct_visible_target() {
    let layout = finallanding::ui::Layout::responsive(1280.0, 720.0);
    let rect = top_bar_action_rect(&layout, TopBarAction::Help);
    let (x, y) = center(rect);

    assert_eq!(top_bar_action_at(&layout, x, y), Some(TopBarAction::Help));
    assert!(!rect.overlaps(&top_bar_action_rect(&layout, TopBarAction::Menu)));
}
