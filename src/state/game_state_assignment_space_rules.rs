use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpaceAssignmentKind {
    Recovery,
    Work,
}

pub(crate) fn space_assignment_kind(
    job_preference: crate::data::colonist::JobPreference,
    building_type: BuildingType,
) -> Option<SpaceAssignmentKind> {
    if building_type == BuildingType::Habitat {
        return Some(SpaceAssignmentKind::Recovery);
    }

    (building_type == job_preference.work_building_type()).then_some(SpaceAssignmentKind::Work)
}

#[cfg(test)]
#[path = "game_state_assignment_space_rules/tests.rs"]
mod tests;
