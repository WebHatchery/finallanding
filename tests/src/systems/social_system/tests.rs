use super::*;

#[test]
fn test_recovery_priority_strengthens_shared_meals() {
    let mut state = GameState::new();
    state.priority.active = ColonyPriority::Recovery;

    for id in 1..=2 {
        let mut colonist = Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::Gourmet,
            JobPreference::Cook,
        );
        colonist.state = ColonistState::Eating;
        colonist.activity_location = ActivityLocation::Building {
            building_id: 7,
            building_type: BuildingType::MessHall,
        };
        colonist.mood = 60.0;
        state.colonists.push(colonist);
    }

    SocialSystem::check_eating_together(&mut state);

    assert_eq!(state.colonists[0].relationships.get(&2), Some(&2));
    assert_eq!(state.colonists[1].relationships.get(&1), Some(&2));
}

#[test]
fn test_first_work_relationship_change_logs_visible_reason() {
    let mut state = GameState::new();
    state.priority.active = ColonyPriority::Recovery;

    for id in 1..=2 {
        let mut colonist = Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        colonist.state = ColonistState::Working;
        colonist.activity_location = ActivityLocation::Building {
            building_id: 12,
            building_type: BuildingType::Workshop,
        };
        colonist.mood = 60.0;
        state.colonists.push(colonist);
    }

    SocialSystem::check_working_together(&mut state);

    let log = state
        .event_log
        .iter()
        .find(|entry| entry.category == LogCategory::Social)
        .expect("first relationship shift should be logged");
    assert_eq!(log.title, "Colonist 1 and Colonist 2 connected");
    assert!(log.detail.contains("Work:"));
    assert!(log.detail.contains("Neutral -> Neutral"));
    assert!(log.detail.contains("now +1"));
}
