use super::*;
use crate::data::colonist::{ActivityLocation, JobPreference, Trait};
use crate::data::schedule::ActivityType;
use crate::game::colonist_ai::assignment::specific_target_for_activity;

#[test]
fn test_work_target_prefers_supportive_occupied_building() {
    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Builder,
    );
    colonist.relationships.insert(2, -35);
    colonist.relationships.insert(3, 30);

    let buildings = vec![
        (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
        (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
    ];
    let social_locations = vec![
        (
            2,
            ActivityLocation::Building {
                building_id: 10,
                building_type: BuildingType::Workshop,
            },
        ),
        (
            3,
            ActivityLocation::Building {
                building_id: 20,
                building_type: BuildingType::Workshop,
            },
        ),
    ];
    let grid = Grid::default();

    let target = find_building_entrance(
        colonist.position,
        BuildingType::Workshop,
        &buildings,
        None,
        &colonist,
        &social_locations,
        &grid,
    )
    .expect("workshop target should be found");

    assert_eq!(target.building_id, 20);
}

#[test]
fn test_work_target_honors_keep_apart_directive() {
    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Builder,
    );
    colonist.relationships.insert(2, 30);
    colonist.avoided_partner_id = Some(2);

    let buildings = vec![
        (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
        (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
    ];
    let social_locations = vec![(
        2,
        ActivityLocation::Building {
            building_id: 10,
            building_type: BuildingType::Workshop,
        },
    )];
    let grid = Grid::default();

    let target = find_building_entrance(
        colonist.position,
        BuildingType::Workshop,
        &buildings,
        None,
        &colonist,
        &social_locations,
        &grid,
    )
    .expect("workshop target should be found");

    assert_eq!(target.building_id, 20);
}

#[test]
fn test_work_target_honors_manual_workplace_assignment() {
    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Builder,
    );
    colonist.assigned_workplace = Some(20);

    let buildings = vec![
        (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
        (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
    ];
    let grid = Grid::default();
    let target = find_building_entrance(
        colonist.position,
        BuildingType::Workshop,
        &buildings,
        specific_target_for_activity(
            &colonist,
            &ActivityType::Work,
            BuildingType::Workshop,
            &buildings,
        ),
        &colonist,
        &[],
        &grid,
    )
    .expect("assigned workshop target should be found");

    assert_eq!(target.building_id, 20);
}
