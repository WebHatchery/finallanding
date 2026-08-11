use super::*;

pub(crate) fn apply_batch_home_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    habitat_id: u32,
    visible_indices: &[usize],
    capacity: u32,
) -> Vec<String> {
    let mut assigned_count = colonists
        .iter()
        .filter(|colonist| colonist.assigned_habitat == Some(habitat_id))
        .count() as u32;
    let mut assigned = Vec::new();

    for index in visible_indices {
        if assigned_count >= capacity {
            break;
        }

        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id || colonist.assigned_habitat == Some(habitat_id) {
            continue;
        }

        colonist.assigned_habitat = Some(habitat_id);
        assigned_count += 1;
        assigned.push(colonist.name.clone());
    }

    assigned
}

pub(crate) fn apply_batch_work_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    workplace_id: u32,
    building_type: BuildingType,
    target_indices: &[usize],
) -> Vec<String> {
    let mut assigned = Vec::new();

    for index in target_indices {
        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id
            || colonist.assigned_workplace == Some(workplace_id)
            || colonist.job_preference.work_building_type() != building_type
        {
            continue;
        }

        colonist.assigned_workplace = Some(workplace_id);
        if matches!(
            colonist.state,
            ColonistState::Working | ColonistState::Moving { .. }
        ) {
            colonist.state = ColonistState::Idle;
            colonist.activity_location = ActivityLocation::None;
        }
        assigned.push(colonist.name.clone());
    }

    assigned
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BatchAssignmentScope {
    Page,
    All,
}

impl BatchAssignmentScope {
    fn label(self) -> &'static str {
        match self {
            BatchAssignmentScope::Page => "visible roster",
            BatchAssignmentScope::All => "all compatible survivors",
        }
    }
}

pub(crate) fn batch_assignment_log(
    title: &'static str,
    source_name: &str,
    pin_prefix: &str,
    building_id: u32,
    scope: BatchAssignmentScope,
    assigned: Vec<String>,
) -> (String, String) {
    let detail = if assigned.is_empty() {
        format!(
            "{} had no compatible survivors in {} to copy {}#{} to.",
            source_name,
            scope.label(),
            pin_prefix,
            building_id
        )
    } else {
        format!(
            "Copied {}#{} from {} to {} in {}.",
            pin_prefix,
            building_id,
            source_name,
            truncate_text(&assigned.join(", "), 45),
            scope.label()
        )
    };

    (title.to_string(), detail)
}

#[cfg(test)]
#[path = "game_state_assignment_batch_rules/tests.rs"]
mod tests;
