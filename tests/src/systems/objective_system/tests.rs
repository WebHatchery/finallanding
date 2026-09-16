use super::*;

fn add_colonists(state: &mut GameState, count: u32) {
    for id in 0..count {
        state.colonists.push(Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));
    }
}

fn place(state: &mut GameState, building_type: BuildingType, x: i32) {
    state
        .building_system
        .try_place_building(&mut state.grid, building_type, Position::new(x, 0));
}

#[test]
fn test_objectives_flag_missing_shelter_as_risk() {
    let mut state = GameState::new();
    add_colonists(&mut state, 4);

    let cards = ObjectiveSystem::active_cards(&state);
    let shelter = cards
        .iter()
        .find(|card| card.title == "Shelter every survivor")
        .expect("shelter card should be visible when at risk");

    assert_eq!(shelter.status, ObjectiveStatus::AtRisk);
    assert!(shelter.detail.contains("0 beds"));
}

#[test]
fn test_objectives_track_core_rooms_and_technology() {
    let mut state = GameState::new();
    add_colonists(&mut state, 2);
    place(&mut state, BuildingType::Habitat, 0);
    place(&mut state, BuildingType::MessHall, 3);
    place(&mut state, BuildingType::Workshop, 7);
    place(&mut state, BuildingType::Storage, 10);
    place(&mut state, BuildingType::ExplorationGate, 13);
    state.technology.add_item(MissionItem::MedicinalGel);
    state.technology.add_item(MissionItem::AlienCircuit);
    state.technology.add_item(MissionItem::NutrientPods);

    let core = ObjectiveSystem::core_rooms_card(&state);
    let tech = ObjectiveSystem::technology_card(&state);

    assert_eq!(core.status, ObjectiveStatus::Complete);
    assert_eq!(tech.status, ObjectiveStatus::Complete);
}
