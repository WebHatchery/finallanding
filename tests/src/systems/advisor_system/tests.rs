use super::*;

fn add_colonist(state: &mut GameState, id: u32) {
    state.colonists.push(Colonist::new(
        id,
        format!("Colonist {}", id),
        Position::new(id as i32, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    ));
}

fn place(state: &mut GameState, building_type: BuildingType, position: Position) {
    state
        .building_system
        .try_place_building(&mut state.grid, building_type, position);
}

#[test]
fn test_advisor_starts_with_shelter_and_core_buildings() {
    let mut state = GameState::new();
    add_colonist(&mut state, 1);
    add_colonist(&mut state, 2);
    add_colonist(&mut state, 3);

    let plan = AdvisorSystem::plan(&state);

    assert!(plan
        .lines
        .iter()
        .any(|line| line.title == "Shelter every survivor"));
    assert!(plan
        .lines
        .iter()
        .any(|line| line.title == "Open a meal point"));
    assert!(plan
        .lines
        .iter()
        .any(|line| line.title == "Mark a survey route"));
}

#[test]
fn test_advisor_prompts_missions_when_core_is_built() {
    let mut state = GameState::new();
    add_colonist(&mut state, 1);
    add_colonist(&mut state, 2);
    place(&mut state, BuildingType::Habitat, Position::new(0, 0));
    place(&mut state, BuildingType::MessHall, Position::new(3, 0));
    place(&mut state, BuildingType::Workshop, Position::new(7, 0));
    place(&mut state, BuildingType::Storage, Position::new(10, 0));
    place(
        &mut state,
        BuildingType::ExplorationGate,
        Position::new(13, 0),
    );
    state.priority.active = ColonyPriority::Survey;

    let plan = AdvisorSystem::plan(&state);

    assert!(plan
        .lines
        .iter()
        .any(|line| line.title == "Push toward field tech"));
}

#[test]
fn test_advisor_settles_after_victory_requirements_are_met() {
    let mut state = GameState::new();
    add_colonist(&mut state, 1);
    place(&mut state, BuildingType::Habitat, Position::new(0, 0));
    place(&mut state, BuildingType::MessHall, Position::new(3, 0));
    place(&mut state, BuildingType::Workshop, Position::new(7, 0));
    place(&mut state, BuildingType::Storage, Position::new(10, 0));
    place(
        &mut state,
        BuildingType::ExplorationGate,
        Position::new(13, 0),
    );
    state.resources.supplies = 20;
    state.technology.add_item(MissionItem::MedicinalGel);
    state.technology.add_item(MissionItem::AlienCircuit);
    state.technology.add_item(MissionItem::NutrientPods);

    let plan = AdvisorSystem::plan(&state);

    assert!(plan
        .lines
        .iter()
        .any(|line| line.title == "Hold the landing site"));
}

#[test]
fn test_active_incident_creates_advisor_priority() {
    let mut state = GameState::new();
    state
        .incidents
        .activate(IncidentType::ToolBreakage, state.tick, 60);

    let plan = AdvisorSystem::plan(&state);

    assert_eq!(plan.lines[0].title, "Recover repair stock");
    assert_eq!(plan.lines[0].severity, AdvisorSeverity::Warning);
}
