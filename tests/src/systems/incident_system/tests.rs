use super::*;

#[test]
fn test_ration_spoilage_uses_existing_supplies_and_advisor_state() {
    let mut state = GameState::new();
    state.tick = TimeSystem::ticks_per_day() + TimeSystem::ticks_per_hour() * 12;
    state.resources.supplies = 10;
    state.colonists.push(Colonist::new(
        1,
        "Ari".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    ));

    IncidentSystem::process_hourly_incidents(&mut state);

    assert!(state.resources.supplies < 10);
    assert!(state.incidents.has_triggered(IncidentType::RationSpoilage));
    assert_eq!(
        state.incidents.active_incident(state.tick),
        Some(IncidentType::RationSpoilage)
    );
    assert!(state
        .event_log
        .iter()
        .any(|entry| entry.title == IncidentType::RationSpoilage.title()));
}
