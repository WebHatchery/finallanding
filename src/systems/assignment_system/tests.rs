use super::*;
use crate::data::colonist::{Colonist, Trait};
use crate::data::types::Position;

fn test_colonist(id: u32, name: &str, role: JobPreference) -> Colonist {
    Colonist::new(
        id,
        name.to_string(),
        Position::new(id as i32, 0),
        Trait::HardWorker,
        role,
    )
}

#[test]
fn test_forecast_flags_tense_same_role_assignment() {
    let mut alice = test_colonist(1, "Alice", JobPreference::Cook);
    let mut bob = test_colonist(2, "Bob", JobPreference::Builder);
    alice.relationships.insert(2, -24);
    bob.relationships.insert(1, -20);

    let forecast = AssignmentSystem::forecast_role_change(&[alice, bob], 1, JobPreference::Builder);

    assert_eq!(forecast.pressure, AssignmentPressure::Tense);
    assert!(forecast.detail.contains("Bob"));
    assert!(forecast.detail.contains("Tense"));
}

#[test]
fn test_forecast_identifies_supportive_same_role_assignment() {
    let mut alice = test_colonist(1, "Alice", JobPreference::Cook);
    let mut bob = test_colonist(2, "Bob", JobPreference::Hauler);
    alice.relationships.insert(2, 30);
    bob.relationships.insert(1, 26);

    let forecast = AssignmentSystem::forecast_role_change(&[alice, bob], 1, JobPreference::Hauler);

    assert_eq!(forecast.pressure, AssignmentPressure::Supported);
    assert!(forecast.detail.contains("Bob"));
    assert!(forecast.detail.contains("Friendly"));
}
