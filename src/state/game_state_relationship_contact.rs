use super::*;

pub(crate) fn shared_assignment_pin(first: &Colonist, second: &Colonist) -> bool {
    first
        .assigned_habitat
        .is_some_and(|id| second.assigned_habitat == Some(id))
        || first
            .assigned_workplace
            .is_some_and(|id| second.assigned_workplace == Some(id))
}

pub(crate) fn adjacent_positions(first: Position, second: Position) -> bool {
    (first.x - second.x).abs() + (first.y - second.y).abs() <= 1
}

pub(crate) fn average_relationship_between(first: &Colonist, second: &Colonist) -> i32 {
    let first_value = first.relationships.get(&second.id).copied().unwrap_or(0);
    let second_value = second.relationships.get(&first.id).copied().unwrap_or(0);

    if first_value == 0 {
        second_value
    } else if second_value == 0 {
        first_value
    } else {
        (first_value + second_value) / 2
    }
}

pub(crate) fn shared_social_location(first: &Colonist, second: &Colonist) -> bool {
    match (&first.activity_location, &second.activity_location) {
        (
            ActivityLocation::Building {
                building_id: first_id,
                ..
            },
            ActivityLocation::Building {
                building_id: second_id,
                ..
            },
        ) => first_id == second_id,
        (ActivityLocation::Ground(first_pos), ActivityLocation::Ground(second_pos)) => {
            first_pos == second_pos
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "game_state_relationship_contact/tests.rs"]
mod tests;
