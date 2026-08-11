use super::*;
use crate::data::colonist::{Colonist, ColonistState, JobPreference, Trait};
use crate::data::event_log::LogCategory;

#[test]
fn test_strained_proximity_logs_visible_mood_drop() {
    let mut state = GameState::new();
    state.tick = 420;

    let mut alice = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Explorer,
    );
    alice.state = ColonistState::Moving {
        target: Position::new(1, 0),
    };
    alice.relationships.insert(2, -30);
    alice.mood = 50.0;

    let mut bob = Colonist::new(
        2,
        "Bob".to_string(),
        Position::new(1, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    bob.relationships.insert(1, -30);

    state.colonists.push(alice);
    state.colonists.push(bob);

    update_colonists(&mut state, 1);

    let alice = &state.colonists[0];
    assert_eq!(alice.position, Position::new(0, 0));
    assert!(alice.mood <= 45.1);

    let log = state
        .event_log
        .iter()
        .find(|entry| entry.category == LogCategory::Social)
        .expect("strained proximity should create a social log");
    assert_eq!(log.title, "Alice avoided Bob");
    assert!(log.detail.contains("strained relationship"));
    assert!(log.detail.contains("Mood dropped"));
}
