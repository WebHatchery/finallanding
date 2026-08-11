use super::*;
use crate::data::colonist::Trait;

fn builder(id: u32, name: &str, position: Position) -> Colonist {
    Colonist::new(
        id,
        name.to_string(),
        position,
        Trait::HardWorker,
        JobPreference::Builder,
    )
}

#[test]
fn test_batch_home_pin_respects_visible_page_and_capacity() {
    let mut colonists = (0..5)
        .map(|id| builder(id, &format!("Colonist {}", id), Position::new(id as i32, 0)))
        .collect::<Vec<_>>();
    colonists[0].assigned_habitat = Some(7);

    let assigned = apply_batch_home_pin(&mut colonists, 0, 7, &[0, 1, 2, 3], 2);

    assert_eq!(assigned, vec!["Colonist 1".to_string()]);
    assert_eq!(colonists[1].assigned_habitat, Some(7));
    assert_eq!(colonists[2].assigned_habitat, None);
}

#[test]
fn test_batch_work_pin_only_copies_to_compatible_visible_roles() {
    let mut colonists = vec![
        builder(0, "Alice", Position::new(0, 0)),
        builder(1, "Bob", Position::new(1, 0)),
        Colonist::new(
            2,
            "Diana".to_string(),
            Position::new(2, 0),
            Trait::Gourmet,
            JobPreference::Cook,
        ),
    ];
    colonists[0].assigned_workplace = Some(9);
    colonists[1].state = ColonistState::Working;
    colonists[1].activity_location = ActivityLocation::Building {
        building_id: 3,
        building_type: BuildingType::Workshop,
    };

    let assigned = apply_batch_work_pin(&mut colonists, 0, 9, BuildingType::Workshop, &[0, 1, 2]);

    assert_eq!(assigned, vec!["Bob".to_string()]);
    assert_eq!(colonists[1].assigned_workplace, Some(9));
    assert_eq!(colonists[1].state, ColonistState::Idle);
    assert_eq!(colonists[1].activity_location, ActivityLocation::None);
    assert_eq!(colonists[2].assigned_workplace, None);
}

#[test]
fn test_batch_assignment_log_names_all_colony_scope() {
    let (_title, detail) = batch_assignment_log(
        "Batch work pins",
        "Alice",
        "W",
        9,
        BatchAssignmentScope::All,
        vec!["Bob".to_string(), "Charlie".to_string()],
    );

    assert!(detail.contains("all compatible survivors"));
    assert!(detail.contains("Bob, Charlie"));
}
