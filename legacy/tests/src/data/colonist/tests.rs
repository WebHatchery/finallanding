use super::*;

#[test]
fn test_job_assignment_cycles_through_work_roles() {
    assert_eq!(
        JobPreference::Explorer.next_assignable(),
        JobPreference::Builder
    );
    assert_eq!(
        JobPreference::Builder.next_assignable(),
        JobPreference::Cook
    );
    assert_eq!(JobPreference::Cook.next_assignable(), JobPreference::Hauler);
    assert_eq!(
        JobPreference::Hauler.next_assignable(),
        JobPreference::Explorer
    );
    assert_eq!(
        JobPreference::None.next_assignable(),
        JobPreference::Explorer
    );
}

#[test]
fn test_job_preferences_map_to_work_buildings() {
    assert_eq!(
        JobPreference::Explorer.work_building_type(),
        BuildingType::ExplorationGate
    );
    assert_eq!(
        JobPreference::Builder.work_building_type(),
        BuildingType::Workshop
    );
    assert_eq!(
        JobPreference::Cook.work_building_type(),
        BuildingType::MessHall
    );
    assert_eq!(
        JobPreference::Hauler.work_building_type(),
        BuildingType::Storage
    );
}
