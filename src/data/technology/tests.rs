use super::*;

#[test]
fn test_items_unlock_technology() {
    let mut technology = TechnologyState::default();

    let unlocked = technology.add_item(MissionItem::MedicinalGel);

    assert_eq!(unlocked, vec![TechId::FieldMedicine]);
    assert!(technology.has(TechId::FieldMedicine));
}

#[test]
fn test_combined_requirements_unlock_storage_lattice() {
    let mut technology = TechnologyState::default();
    technology.add_item(MissionItem::StructuralAlloy);

    let unlocked = technology.add_item(MissionItem::AlienCircuit);

    assert!(unlocked.contains(&TechId::StorageLattice));
    assert!(technology.has(TechId::StorageLattice));
}

#[test]
fn test_advanced_technology_requires_prerequisite_and_extra_items() {
    let mut technology = TechnologyState::default();
    technology.add_item(MissionItem::MedicinalGel);
    assert!(technology.has(TechId::FieldMedicine));
    assert!(!technology.has(TechId::TriageProtocols));

    let unlocked = technology.add_item(MissionItem::AlienCircuit);
    assert!(!unlocked.contains(&TechId::TriageProtocols));

    let unlocked = technology.add_item(MissionItem::MedicinalGel);
    assert!(unlocked.contains(&TechId::TriageProtocols));
    assert_eq!(technology.injury_duration_ticks(), 40);
}

#[test]
fn test_research_target_and_progress_text_show_available_tree_work() {
    let mut technology = TechnologyState::default();

    assert_eq!(
        technology.next_research_target(),
        Some(TechId::FieldMedicine)
    );
    assert_eq!(
        technology.requirement_progress_text(TechId::FieldMedicine),
        "Gel 0/1"
    );

    technology.add_item(MissionItem::MedicinalGel);
    assert_eq!(
        technology.next_research_target(),
        Some(TechId::SurveyScanners)
    );
    assert!(technology
        .visible_research_targets(5)
        .contains(&TechId::TriageProtocols));
}

#[test]
fn test_advanced_effects_stack_into_existing_system_modifiers() {
    let mut technology = TechnologyState::default();
    technology.add_item(MissionItem::AlienCircuit);
    technology.add_item(MissionItem::AlienCircuit);
    technology.add_item(MissionItem::StructuralAlloy);

    assert!(technology.has(TechId::DroneSurvey));
    assert_eq!(technology.mission_danger_reduction(), 18);
    assert_eq!(technology.mission_cooldown_reduction(), 10);

    technology.add_item(MissionItem::StructuralAlloy);
    assert!(technology.has(TechId::HullRetrofits));
    assert_eq!(technology.habitat_capacity_bonus(), 2);
    assert_eq!(technology.storage_capacity_bonus(), 35);
}
