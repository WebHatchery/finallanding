use super::super::MissionSystem;

#[test]
fn test_priority_adjusts_mission_danger() {
    let mut state = GameState::new();
    state.priority.active = ColonyPriority::Recovery;
    assert_eq!(
        MissionPlanning::mission_danger_percent(&state, MissionType::PerimeterScan),
        12
    );

    state.priority.active = ColonyPriority::Survey;
    assert_eq!(
        MissionPlanning::mission_danger_percent(&state, MissionType::PerimeterScan),
        27
    );
    assert!(MissionPlanning::mission_danger_percent(&state, MissionType::DeepSurvey) > 27);
}

#[test]
fn test_priority_changes_visible_mission_recommendation() {
    let mut state = GameState::new();
    state.resources.supplies = 1;
    assert_eq!(
        MissionSystem::recommended_mission_type(&state),
        MissionType::SupplyRun
    );
    assert!(
        MissionPlanning::recommendation_reason(&state, MissionType::SupplyRun)
            .contains("safety buffer")
    );

    state.resources.supplies = 30;
    state.priority.active = ColonyPriority::Recovery;
    assert_eq!(
        MissionSystem::recommended_mission_type(&state),
        MissionType::PerimeterScan
    );

    state.priority.active = ColonyPriority::Survey;
    assert_eq!(
        MissionSystem::recommended_mission_type(&state),
        MissionType::DeepSurvey
    );

    let plans = MissionSystem::mission_plans(&state);
    assert!(plans
        .iter()
        .any(|plan| plan.mission_type == MissionType::DeepSurvey && plan.recommended));
}
