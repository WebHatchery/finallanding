use super::*;

#[test]
fn test_workplace_assignment_ignores_wrong_building_type() {
    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Builder,
    );
    colonist.assigned_workplace = Some(20);

    let buildings = vec![(20, BuildingType::MessHall, Position::new(12, 12), (3, 2))];

    assert_eq!(
        specific_target_for_activity(
            &colonist,
            &ActivityType::Work,
            BuildingType::Workshop,
            &buildings,
        ),
        None
    );
}

#[test]
fn test_habitat_assignment_prefers_supportive_roommate() {
    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Builder,
    );
    colonist.relationships.insert(2, -26);
    colonist.relationships.insert(3, 22);

    let buildings = vec![
        (10, BuildingType::Habitat, Position::new(1, 1), (2, 2)),
        (20, BuildingType::Habitat, Position::new(10, 10), (2, 2)),
    ];
    let social_locations = vec![
        (
            2,
            ActivityLocation::Building {
                building_id: 10,
                building_type: BuildingType::Habitat,
            },
        ),
        (
            3,
            ActivityLocation::Building {
                building_id: 20,
                building_type: BuildingType::Habitat,
            },
        ),
    ];
    let mut building_occupancy = HashMap::new();
    let mut pending_logs = Vec::new();

    let target = find_or_assign_habitat(
        &mut colonist,
        &buildings,
        &mut building_occupancy,
        2,
        &social_locations,
        &mut pending_logs,
        420,
    );

    assert_eq!(target, Some(BuildingType::Habitat));
    assert_eq!(colonist.assigned_habitat, Some(20));
}
