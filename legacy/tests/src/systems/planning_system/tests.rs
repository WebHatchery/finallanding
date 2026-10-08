use super::*;

#[test]
fn test_building_feedback_includes_planning_facts() {
    let state = GameState::new();

    let feedback =
        PlanningSystem::building_feedback(&state, BuildingType::MessHall, Position::new(1, 1));

    assert!(feedback.can_place());
    assert_eq!(feedback.footprint, (3, 2));
    assert_eq!(feedback.cost, 12);
    assert_eq!(feedback.helps, "Food");
    assert!(feedback.purpose.contains("Meal"));
    assert!(feedback.impact.contains("cooks"));
}

#[test]
fn test_invalid_reason_reports_insufficient_salvage() {
    let mut state = GameState::new();
    state.resources.salvage = 0;

    let feedback = PlanningSystem::building_feedback(
        &state,
        BuildingType::ExplorationGate,
        Position::new(1, 1),
    );

    assert!(!feedback.can_place());
    assert_eq!(
        feedback.invalid_reason.as_deref(),
        Some("Need 14 salvage; 0 available.")
    );
}

#[test]
fn test_invalid_reason_reports_map_and_overlap_blocks() {
    let mut state = GameState::new();

    assert_eq!(
        PlanningSystem::invalid_reason(
            &state,
            BuildingType::MessHall,
            Position::new(state.data.grid.width as i32 - 1, state.data.grid.height as i32 - 1)
        )
        .as_deref(),
        Some("Footprint leaves the map.")
    );

    state.building_system.try_place_building(
        &mut state.data.grid,
        BuildingType::Habitat,
        Position::new(2, 2),
    );

    assert_eq!(
        PlanningSystem::invalid_reason(&state, BuildingType::Storage, Position::new(2, 2))
            .as_deref(),
        Some("Footprint overlaps another building.")
    );

    state.data.grid.set_cell_type(8, 8, CellType::Wall);
    assert_eq!(
        PlanningSystem::invalid_reason(&state, BuildingType::Storage, Position::new(8, 8))
            .as_deref(),
        Some("Footprint overlaps blocked terrain.")
    );
}

#[test]
fn test_placement_log_detail_explains_mechanical_change() {
    let state = GameState::new();
    let feedback =
        PlanningSystem::building_feedback(&state, BuildingType::Storage, Position::new(1, 1));

    let detail = PlanningSystem::placement_log_detail(&feedback, 4, 28);

    assert!(detail.contains("Added Storage support"));
    assert!(detail.contains("Adds supply capacity"));
    assert!(detail.contains("Building #4 cost 6 salvage"));
    assert!(detail.contains("28 remain"));
}
