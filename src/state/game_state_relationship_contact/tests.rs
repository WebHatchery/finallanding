use super::*;
use crate::data::colonist::{JobPreference, Trait};

fn test_colonist(id: u32, position: Position) -> Colonist {
    Colonist::new(
        id,
        format!("Colonist {}", id),
        position,
        Trait::HardWorker,
        JobPreference::Builder,
    )
}

#[test]
fn test_average_relationship_uses_bidirectional_values() {
    let mut first = test_colonist(1, Position::new(0, 0));
    let mut second = test_colonist(2, Position::new(1, 0));

    first.relationships.insert(2, 26);
    second.relationships.insert(1, 30);

    assert_eq!(average_relationship_between(&first, &second), 28);
}

#[test]
fn test_shared_social_location_requires_same_building_or_ground_cell() {
    let mut first = test_colonist(1, Position::new(0, 0));
    let mut second = test_colonist(2, Position::new(1, 0));

    first.activity_location = ActivityLocation::Building {
        building_id: 7,
        building_type: BuildingType::Workshop,
    };
    second.activity_location = ActivityLocation::Building {
        building_id: 7,
        building_type: BuildingType::Workshop,
    };

    assert!(shared_social_location(&first, &second));

    second.activity_location = ActivityLocation::Ground(Position::new(2, 2));
    assert!(!shared_social_location(&first, &second));
}

#[test]
fn test_shared_assignment_and_adjacency_drive_social_contact() {
    let mut first = test_colonist(1, Position::new(4, 4));
    let mut second = test_colonist(2, Position::new(5, 4));

    assert!(adjacent_positions(first.position, second.position));
    first.assigned_workplace = Some(9);
    second.assigned_workplace = Some(9);
    assert!(shared_assignment_pin(&first, &second));
    second.assigned_workplace = Some(10);
    assert!(!shared_assignment_pin(&first, &second));
}
