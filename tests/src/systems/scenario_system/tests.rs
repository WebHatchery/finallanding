use super::*;

#[test]
fn test_victory_requires_day_7_resources_and_tech() {
    let mut state = GameState::new();
    state.tick = TimeSystem::TICKS_PER_DAY * 6;
    state.resources.condition = ColonyCondition::Stable;
    state.resources.supplies = 20;
    state.technology.add_item(MissionItem::MedicinalGel);
    state.technology.add_item(MissionItem::AlienCircuit);
    state.technology.add_item(MissionItem::NutrientPods);

    ScenarioSystem::evaluate(&mut state);

    assert_eq!(state.scenario.outcome, ScenarioOutcome::Victory);
    assert_eq!(state.time.speed, TimeSpeed::Paused);
}

#[test]
fn test_critical_colony_fails_on_target_day() {
    let mut state = GameState::new();
    state.tick = TimeSystem::TICKS_PER_DAY * 6;
    state.resources.condition = ColonyCondition::Critical;
    state.resources.supplies = 20;
    state.technology.add_item(MissionItem::MedicinalGel);
    state.technology.add_item(MissionItem::AlienCircuit);
    state.technology.add_item(MissionItem::NutrientPods);

    ScenarioSystem::evaluate(&mut state);

    assert_eq!(state.scenario.outcome, ScenarioOutcome::Failure);
}

#[test]
fn test_collapsed_colony_fails() {
    let mut state = GameState::new();
    state.resources.condition = ColonyCondition::Collapsed;

    ScenarioSystem::evaluate(&mut state);

    assert_eq!(state.scenario.outcome, ScenarioOutcome::Failure);
    assert_eq!(state.time.speed, TimeSpeed::Paused);
}

#[test]
fn test_day_7_target_sits_in_30_to_40_minute_run_window() {
    let mut state = GameState::new();
    state.tick = 420;

    let minutes = ScenarioSystem::estimated_real_minutes_to_target(&state, 0.25);

    assert!((30.0..=40.0).contains(&minutes));
}
