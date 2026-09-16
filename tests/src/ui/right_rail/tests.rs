use super::*;

#[test]
fn test_resource_rows_use_real_gameplay_tracks() {
    let resources = ResourceState {
        supplies: 5,
        prepared_meals: 2,
        exploration_progress: 17,
        workshop_progress: 23,
        hauling_progress: 31,
        ..Default::default()
    };

    let rows = resource_rows(&resources, 40, 4);
    let labels = rows.iter().map(|row| row.label).collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec!["Food", "Salvage", "Meals", "Survey", "Repair", "Hauling"]
    );
    assert_eq!(rows[0].detail, "1.2 days");
    assert!(rows[0].alert);
    assert_eq!(rows[2].detail, "-2 need");
}

#[test]
fn test_strongest_relationship_value_uses_absolute_pressure() {
    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    colonist.relationships.insert(2, 18);
    colonist.relationships.insert(3, -31);

    assert_eq!(strongest_relationship_value(&colonist), Some(-31));
    assert_eq!(relationship_color(-31), style::ALERT_RED);
}

#[test]
fn test_social_footer_surfaces_specific_pair_pressure() {
    let summary = ColonyPressureSummary {
        average_mood: 42.0,
        average_relationship: -4.0,
        close_pairs: 0,
        strained_pairs: 1,
        connected_pairs: vec![],
        tense_pairs: vec![finallanding::systems::summary_system::RelationshipPairSummary {
            first_name: "Alice".to_string(),
            second_name: "Bob".to_string(),
            value: -30,
            label: "Tense",
        }],
        strongest_pair: None,
        weakest_pair: None,
    };

    assert!(social_footer(&summary).contains("Alice / Bob -30"));
}
