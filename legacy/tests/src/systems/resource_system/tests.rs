use super::*;

#[test]
fn test_daily_supply_need_matches_colonists() {
    let mut state = GameState::new();
    state.colonists.push(Colonist::new(
        1,
        "Test".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    ));

    assert_eq!(ResourceSystem::daily_supply_need(&state), 1);
}

#[test]
fn test_prepared_meals_reduce_daily_need() {
    let mut state = GameState::new();
    state.resources.prepared_meals = 1;
    state.colonists.push(Colonist::new(
        1,
        "Test".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    ));

    assert_eq!(ResourceSystem::daily_supply_need(&state), 0);
}

#[test]
fn test_hydroponic_technology_reduces_daily_need() {
    let mut state = GameState::new();
    state
        .technology
        .add_item(finallanding::data::mission::MissionItem::NutrientPods);
    state.colonists.push(Colonist::new(
        1,
        "Test".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    ));

    assert_eq!(ResourceSystem::daily_supply_need(&state), 0);
}

#[test]
fn test_nutrient_culture_reduces_daily_need_further() {
    let mut state = GameState::new();
    state.technology.add_item(MissionItem::NutrientPods);
    state.technology.add_item(MissionItem::NutrientPods);
    state.technology.add_item(MissionItem::MedicinalGel);
    for id in 0..4 {
        state.colonists.push(Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));
    }

    assert_eq!(state.technology.daily_supply_reduction(), 2);
    assert_eq!(ResourceSystem::daily_supply_need(&state), 10);
}

#[test]
fn test_ration_shortage_reduces_mood() {
    let mut state = GameState::new();
    state.resources.supplies = 0;
    state.colonists.push(Colonist::new(
        1,
        "Test".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    ));

    ResourceSystem::handle_new_day(&mut state);

    assert!(state.colonists[0].mood < 50.0);
    assert_eq!(state.resources.condition, ColonyCondition::Critical);
}

#[test]
fn test_missing_habitat_capacity_strains_condition() {
    let mut state = GameState::new();
    state.resources.supplies = 20;
    for id in 0..6 {
        state.colonists.push(Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));
    }

    ResourceSystem::update_condition(&mut state);

    assert_eq!(ResourceSystem::habitat_capacity(&state), 0);
    assert_eq!(state.resources.condition, ColonyCondition::Critical);
}
