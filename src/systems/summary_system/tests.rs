use super::*;
use crate::data::colonist::{Colonist, JobPreference, Trait};
use crate::data::types::Position;

#[test]
fn test_colony_pressure_summary_finds_best_and_worst_relationships() {
    let mut state = GameState::new();
    state.colonists.push(test_colonist(1, "Alice"));
    state.colonists.push(test_colonist(2, "Bob"));
    state.colonists.push(test_colonist(3, "Charlie"));

    state.colonists[0].relationships.insert(2, 24);
    state.colonists[1].relationships.insert(1, 20);
    state.colonists[0].relationships.insert(3, -18);
    state.colonists[2].relationships.insert(1, -14);

    let summary = SummarySystem::colony_pressure_summary(&state);

    assert_eq!(summary.close_pairs, 1);
    assert_eq!(summary.strained_pairs, 1);
    assert_eq!(
        summary.strongest_pair.as_ref().map(|pair| pair.value),
        Some(22)
    );
    assert_eq!(
        summary.weakest_pair.as_ref().map(|pair| pair.value),
        Some(-16)
    );
}

#[test]
fn test_colony_pressure_summary_keeps_neutral_pairs_clear() {
    let mut state = GameState::new();
    state.colonists.push(test_colonist(1, "Alice"));
    state.colonists.push(test_colonist(2, "Bob"));
    state.colonists.push(test_colonist(3, "Charlie"));

    let summary = SummarySystem::colony_pressure_summary(&state);

    assert_eq!(summary.close_pairs, 0);
    assert_eq!(summary.strained_pairs, 0);
    assert_eq!(summary.average_relationship, 0.0);
    assert!(summary.connected_pairs.is_empty());
    assert!(summary.tense_pairs.is_empty());
    assert_eq!(
        summary.strongest_pair.as_ref().map(|pair| pair.label),
        Some("Neutral")
    );
    assert_eq!(
        summary.weakest_pair.as_ref().map(|pair| pair.value),
        Some(0)
    );
}

#[test]
fn test_colony_pressure_summary_counts_multiple_hostile_pairs() {
    let mut state = GameState::new();
    state.colonists.push(test_colonist(1, "Alice"));
    state.colonists.push(test_colonist(2, "Bob"));
    state.colonists.push(test_colonist(3, "Charlie"));

    state.colonists[0].relationships.insert(2, -34);
    state.colonists[1].relationships.insert(1, -30);
    state.colonists[0].relationships.insert(3, -16);
    state.colonists[2].relationships.insert(1, -14);

    let summary = SummarySystem::colony_pressure_summary(&state);

    assert_eq!(summary.close_pairs, 0);
    assert_eq!(summary.strained_pairs, 2);
    assert_eq!(summary.tense_pairs.len(), 2);
    assert_eq!(summary.tense_pairs[0].value, -32);
    assert_eq!(summary.tense_pairs[1].value, -15);
    assert_eq!(
        summary.weakest_pair.as_ref().map(|pair| pair.value),
        Some(-32)
    );
    assert_eq!(
        summary.weakest_pair.as_ref().map(|pair| pair.label),
        Some("Hostile")
    );
}

#[test]
fn test_colony_pressure_summary_finds_one_strong_positive_pair() {
    let mut state = GameState::new();
    state.colonists.push(test_colonist(1, "Alice"));
    state.colonists.push(test_colonist(2, "Bob"));
    state.colonists.push(test_colonist(3, "Charlie"));

    state.colonists[0].relationships.insert(2, 42);
    state.colonists[1].relationships.insert(1, 38);

    let summary = SummarySystem::colony_pressure_summary(&state);

    assert_eq!(summary.close_pairs, 1);
    assert_eq!(summary.strained_pairs, 0);
    assert_eq!(summary.connected_pairs.len(), 1);
    assert_eq!(summary.connected_pairs[0].value, 40);
    assert_eq!(
        summary.strongest_pair.as_ref().map(|pair| pair.value),
        Some(40)
    );
    assert_eq!(
        summary.strongest_pair.as_ref().map(|pair| pair.label),
        Some("Close")
    );
}

#[test]
fn test_daily_story_report_names_best_worst_and_pressure() {
    let mut state = GameState::new();
    state.resources.supplies = 1;
    state.colonists.push(test_colonist(1, "Alice"));
    state.colonists.push(test_colonist(2, "Bob"));
    state.colonists.push(test_colonist(3, "Charlie"));
    state.colonists[0].relationships.insert(2, 24);
    state.colonists[1].relationships.insert(1, 20);
    state.colonists[0].relationships.insert(3, -18);
    state.colonists[2].relationships.insert(1, -14);

    let report = SummarySystem::previous_day_report(&state, 3);

    assert_eq!(report.title, "Day 2 summary");
    assert!(report
        .detail
        .contains("Alice and Bob were the strongest bond"));
    assert!(report
        .detail
        .contains("Alice and Charlie carried the sharpest tension"));
    assert!(report.detail.contains("Next pressure: raise supplies"));
    assert!(report.recommendation.contains("raise supplies"));
}

#[test]
fn test_summarize_previous_day_preserves_social_history() {
    let mut state = GameState::new();
    state.resources.supplies = 1;
    state.colonists.push(test_colonist(1, "Alice"));
    state.colonists.push(test_colonist(2, "Bob"));
    state.colonists[0].relationships.insert(2, -20);
    state.colonists[1].relationships.insert(1, -24);

    SummarySystem::summarize_previous_day(&mut state, 2);

    let entry = state
        .social_history
        .last()
        .expect("daily summary should be preserved as history");
    assert_eq!(entry.day, 1);
    assert_eq!(entry.title, "Day 1 summary");
    assert_eq!(entry.strained_pairs, 1);
    assert!(entry.detail.contains("sharpest tension"));
    assert!(state
        .event_log
        .iter()
        .any(|entry| entry.title == "Day 1 summary"));
}

fn test_colonist(id: u32, name: &str) -> Colonist {
    Colonist::new(
        id,
        name.to_string(),
        Position::new(id as i32, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    )
}
