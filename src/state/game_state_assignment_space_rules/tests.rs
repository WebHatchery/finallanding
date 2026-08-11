use super::*;
use crate::data::colonist::JobPreference;

#[test]
fn test_space_assignment_kind_matches_role_and_room() {
    assert_eq!(
        space_assignment_kind(JobPreference::Builder, BuildingType::Habitat),
        Some(SpaceAssignmentKind::Recovery)
    );
    assert_eq!(
        space_assignment_kind(JobPreference::Builder, BuildingType::Workshop),
        Some(SpaceAssignmentKind::Work)
    );
    assert_eq!(
        space_assignment_kind(JobPreference::Builder, BuildingType::MessHall),
        None
    );
    assert_eq!(
        space_assignment_kind(JobPreference::Cook, BuildingType::MessHall),
        Some(SpaceAssignmentKind::Work)
    );
}
