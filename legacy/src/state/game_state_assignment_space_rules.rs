//! game state assignment space rules domain.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceAssignmentKind {
    Recovery,
    Work,
}

pub fn space_assignment_kind(
    job_preference: crate::data::colonist::JobPreference,
    building_type: BuildingType,
) -> Option<SpaceAssignmentKind> {
    if building_type == BuildingType::Habitat {
        return Some(SpaceAssignmentKind::Recovery);
    }

    (building_type == job_preference.work_building_type()).then_some(SpaceAssignmentKind::Work)
}
