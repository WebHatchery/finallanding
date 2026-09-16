#[test]
fn test_mission_item_unlocks_technology() {
    let mut state = GameState::new();
    let mission = ActiveMission {
        id: 1,
        colonist_id: 1,
        mission_type: MissionType::PerimeterScan,
        started_tick: 0,
        completes_at_tick: 1,
        danger_percent: 0,
        priority: ColonyPriority::Survey,
    };
    state.colonists.push(Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(3, 3),
        Trait::FastWalker,
        JobPreference::Explorer,
    ));
    state.missions.active_missions.push(mission);
    state.tick = 1;

    MissionResolution::process_completed_missions(&mut state);

    assert!(state.technology.has(TechId::FieldMedicine));
}

#[test]
fn test_mission_completion_log_names_mission_rewards_and_priority() {
    let mut state = GameState::new();
    let mission = ActiveMission {
        id: 1,
        colonist_id: 1,
        mission_type: MissionType::SupplyRun,
        started_tick: 0,
        completes_at_tick: 1,
        danger_percent: 0,
        priority: ColonyPriority::Stockpile,
    };
    state.colonists.push(Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(3, 3),
        Trait::FastWalker,
        JobPreference::Explorer,
    ));
    state.missions.active_missions.push(mission);
    state.tick = 1;

    MissionResolution::process_completed_missions(&mut state);

    let log = state
        .event_log
        .iter()
        .find(|entry| entry.title == "Scout returned from Supply Run")
        .expect("mission completion should log the mission name");
    assert!(log.detail.contains("Stockpile priority"));
    assert!(log.detail.contains("Found Salvage Cache"));
    assert!(log.detail.contains("Stored"));
    assert!(log.detail.contains("Added"));
    assert!(log.detail.contains("resource-focused return"));
}

#[test]
fn test_dangerous_mission_can_hurt_colonist() {
    let mut state = GameState::new();
    let mission = ActiveMission {
        id: 1,
        colonist_id: 1,
        mission_type: MissionType::PerimeterScan,
        started_tick: 0,
        completes_at_tick: 1,
        danger_percent: 100,
        priority: ColonyPriority::Stockpile,
    };
    state.colonists.push(Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(3, 3),
        Trait::FastWalker,
        JobPreference::Explorer,
    ));
    state.missions.active_missions.push(mission);
    state.tick = 1;

    MissionResolution::process_completed_missions(&mut state);

    assert!(state.colonists[0].is_hurt(state.tick));
    assert!(!state.colonists[0].can_start_mission(state.tick));
}

#[test]
fn test_survey_priority_favors_research_items() {
    let mission = ActiveMission {
        id: 4,
        colonist_id: 1,
        mission_type: MissionType::PerimeterScan,
        started_tick: 0,
        completes_at_tick: 1,
        danger_percent: 0,
        priority: ColonyPriority::Survey,
    };

    assert_ne!(
        MissionResolution::item_for_mission(&mission),
        MissionItem::SalvageCache
    );
}
