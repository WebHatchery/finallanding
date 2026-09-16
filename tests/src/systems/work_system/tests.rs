use super::*;

#[test]
fn test_workshop_generates_salvage() {
    let mut state = GameState::new();
    let mut colonist = Colonist::new(
        1,
        "Builder".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    colonist.state = ColonistState::Working;
    colonist.activity_location = ActivityLocation::Building {
        building_id: 1,
        building_type: BuildingType::Workshop,
    };
    colonist.mood = 70.0;
    state.colonists.push(colonist);

    let salvage_before = state.resources.salvage;
    WorkSystem::process_hourly_work(&mut state);
    WorkSystem::process_hourly_work(&mut state);

    assert!(state.resources.salvage > salvage_before);
}

#[test]
fn test_cook_prepares_meals() {
    let mut state = GameState::new();
    let mut colonist = Colonist::new(
        1,
        "Cook".to_string(),
        Position::new(0, 0),
        Trait::Gourmet,
        JobPreference::Cook,
    );
    colonist.state = ColonistState::Working;
    colonist.activity_location = ActivityLocation::Building {
        building_id: 1,
        building_type: BuildingType::MessHall,
    };
    state.colonists.push(colonist);

    WorkSystem::process_hourly_work(&mut state);
    WorkSystem::process_hourly_work(&mut state);

    assert!(state.resources.prepared_meals > 0);
}

#[test]
fn test_stockpile_priority_boosts_workshop_salvage() {
    let mut state = GameState::new();
    state.priority.active = ColonyPriority::Stockpile;
    let mut colonist = Colonist::new(
        1,
        "Builder".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    colonist.state = ColonistState::Working;
    colonist.activity_location = ActivityLocation::Building {
        building_id: 1,
        building_type: BuildingType::Workshop,
    };
    colonist.mood = 70.0;
    state.colonists.push(colonist);

    WorkSystem::process_hourly_work(&mut state);

    assert_eq!(state.resources.salvage, ResourceState::default().salvage);
    assert_eq!(state.resources.workshop_progress, 5);

    WorkSystem::process_hourly_work(&mut state);

    assert!(state.resources.salvage > ResourceState::default().salvage);
}

#[test]
fn test_survey_priority_boosts_exploration_progress() {
    let mut state = GameState::new();
    state.priority.active = ColonyPriority::Survey;
    let mut colonist = Colonist::new(
        1,
        "Scout".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Explorer,
    );
    colonist.state = ColonistState::Working;
    colonist.activity_location = ActivityLocation::Building {
        building_id: 1,
        building_type: BuildingType::ExplorationGate,
    };
    colonist.mood = 70.0;
    state.colonists.push(colonist);

    WorkSystem::process_hourly_work(&mut state);

    assert_eq!(state.resources.exploration_progress, 5);
}

#[test]
fn test_fabrication_jigs_improve_salvage_recovery() {
    let mut state = GameState::new();
    for item in [
        MissionItem::StructuralAlloy,
        MissionItem::StructuralAlloy,
        MissionItem::StructuralAlloy,
        MissionItem::AlienCircuit,
        MissionItem::AlienCircuit,
    ] {
        state.technology.add_item(item);
    }
    assert_eq!(state.technology.salvage_recovery_bonus(), 1);

    let mut colonist = Colonist::new(
        1,
        "Builder".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    colonist.state = ColonistState::Working;
    colonist.activity_location = ActivityLocation::Building {
        building_id: 1,
        building_type: BuildingType::Workshop,
    };
    colonist.mood = 70.0;
    state.colonists.push(colonist);

    let salvage_before = state.resources.salvage;
    WorkSystem::process_hourly_work(&mut state);
    WorkSystem::process_hourly_work(&mut state);

    assert_eq!(state.resources.salvage - salvage_before, 2);
}
