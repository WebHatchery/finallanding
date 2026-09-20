use super::*;

fn center(rect: Rect) -> (f32, f32) {
    (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
}

#[test]
fn test_toolbar_mode_hit_zones_match_visible_buttons() {
    let toolbar = Rect::new(380.0, 640.0, 520.0, 66.0);
    let (build_x, build_y) = center(toolbar_button_rect(toolbar, 0));
    let (log_x, log_y) = center(toolbar_button_rect(toolbar, 4));

    assert_eq!(
        toolbar_mode_at(toolbar, build_x, build_y),
        Some(ToolbarMode::Build)
    );
    assert_eq!(
        toolbar_mode_at(toolbar, log_x, log_y),
        Some(ToolbarMode::Log)
    );
    assert_eq!(toolbar_mode_at(toolbar, 10.0, 10.0), None);
}

#[test]
fn test_toolbar_context_hits_buildings_and_missions() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (habitat_x, habitat_y) = center(toolbar_context_item_rect(context, 0));
    let (gate_x, gate_y) = center(toolbar_context_item_rect(context, 4));

    assert_eq!(
        toolbar_building_at_for_mode(context, ToolbarMode::Build, habitat_x, habitat_y),
        Some(BuildingType::Habitat)
    );
    assert_eq!(
        toolbar_building_at_for_mode(context, ToolbarMode::Build, gate_x, gate_y),
        Some(BuildingType::ExplorationGate)
    );
    assert_eq!(
        toolbar_mission_at(context, habitat_x, habitat_y),
        Some(MissionType::SupplyRun)
    );
}

#[test]
fn test_toolbar_building_filters_match_modes() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (first_x, first_y) = center(toolbar_context_item_rect(context, 0));
    let (third_x, third_y) = center(toolbar_context_item_rect(context, 2));

    assert_eq!(
        toolbar_building_at_for_mode(context, ToolbarMode::Rooms, first_x, first_y),
        Some(BuildingType::Habitat)
    );
    assert_eq!(
        toolbar_building_at_for_mode(context, ToolbarMode::Rooms, third_x, third_y),
        Some(BuildingType::Storage)
    );
    assert_eq!(
        toolbar_building_at_for_mode(context, ToolbarMode::Objects, third_x, third_y),
        None
    );
}

#[test]
fn test_toolbar_priority_and_colonist_hits() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (priority_x, priority_y) = center(toolbar_context_item_rect(context, 1));
    let (colonist_x, colonist_y) = center(toolbar_list_item_rect(context, 4));

    assert_eq!(
        toolbar_priority_at(context, priority_x, priority_y),
        Some(ColonyPriority::Stockpile)
    );
    assert_eq!(
        toolbar_colonist_index_at(context, 5, colonist_x, colonist_y),
        Some(4)
    );
    assert_eq!(
        toolbar_colonist_index_at(context, 4, colonist_x, colonist_y),
        None
    );
}

#[test]
fn test_research_action_is_below_mission_cards() {
    let context = Rect::new(380.0, 460.0, 520.0, 196.0);
    let (action_x, action_y) = center(research_action_rect(context));
    let (mission_x, mission_y) = center(toolbar_context_item_rect(context, 0));

    assert!(research_action_rect(context).contains(vec2(action_x, action_y)));
    assert_ne!((action_x, action_y), (mission_x, mission_y));
    assert!(action_y > mission_y);
}
