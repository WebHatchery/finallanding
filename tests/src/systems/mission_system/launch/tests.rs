use super::super::MissionSystem;

fn add_gate(state: &mut GameState) {
    state.building_system.try_place_building(
        &mut state.grid,
        BuildingType::ExplorationGate,
        Position::new(0, 0),
    );
}

#[test]
fn test_launch_creates_typed_mission_and_starts_cooldown() {
    let mut state = GameState::new();
    add_gate(&mut state);
    state.colonists.push(Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(3, 3),
        Trait::FastWalker,
        JobPreference::Explorer,
    ));

    let mission_id = MissionSystem::launch_mission(&mut state, MissionType::DeepSurvey).unwrap();

    assert_eq!(mission_id, 1);
    assert_eq!(
        state.missions.active_missions[0].mission_type,
        MissionType::DeepSurvey
    );
    assert_eq!(
        state.missions.active_missions[0].remaining_ticks(state.tick),
        MissionType::DeepSurvey.definition().duration_minutes
    );
    assert_eq!(
        state.missions.cooldown_remaining(state.tick),
        MissionType::DeepSurvey.definition().cooldown_minutes
    );
    assert!(state.colonists[0].is_on_mission());
}

#[test]
fn test_hurt_colonist_cannot_launch() {
    let mut state = GameState::new();
    add_gate(&mut state);
    let mut colonist = Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(3, 3),
        Trait::FastWalker,
        JobPreference::Explorer,
    );
    colonist.injured_until_tick = Some(10);
    state.colonists.push(colonist);

    assert_eq!(
        MissionSystem::launch_mission(&mut state, MissionType::PerimeterScan),
        Err(LaunchMissionError::NoAvailableColonist)
    );
}

#[test]
fn test_mission_cooldown_blocks_rapid_relaunch() {
    let mut state = GameState::new();
    add_gate(&mut state);
    state.colonists.push(Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(3, 3),
        Trait::FastWalker,
        JobPreference::Explorer,
    ));
    state.colonists.push(Colonist::new(
        2,
        "Backup".to_string(),
        Position::new(4, 3),
        Trait::HardWorker,
        JobPreference::Builder,
    ));

    MissionSystem::launch_mission(&mut state, MissionType::SupplyRun).unwrap();

    assert_eq!(
        MissionSystem::launch_mission(&mut state, MissionType::PerimeterScan),
        Err(LaunchMissionError::MissionCooldown {
            remaining_ticks: MissionType::SupplyRun.definition().cooldown_minutes
        })
    );
}

#[test]
fn test_drone_survey_reduces_mission_cooldown() {
    let mut state = GameState::new();
    add_gate(&mut state);
    state.technology.add_item(MissionItem::AlienCircuit);
    state.technology.add_item(MissionItem::AlienCircuit);
    state.technology.add_item(MissionItem::StructuralAlloy);
    state.colonists.push(Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(3, 3),
        Trait::FastWalker,
        JobPreference::Explorer,
    ));

    MissionSystem::launch_mission(&mut state, MissionType::PerimeterScan).unwrap();

    assert_eq!(
        state.missions.cooldown_remaining(state.tick),
        MissionType::PerimeterScan.definition().cooldown_minutes - 10
    );
}
