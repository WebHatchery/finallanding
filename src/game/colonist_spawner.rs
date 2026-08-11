use crate::data::colonist::{Colonist, JobPreference, Trait};
use crate::data::game_state::GameState;
use crate::data::types::Position;

pub fn spawn_initial_colonists(state: &mut GameState) {
    let colonists_data = [
        ("Alice", Trait::HardWorker, JobPreference::Builder),
        ("Bob", Trait::Lazy, JobPreference::Cook),
        ("Charlie", Trait::FastWalker, JobPreference::Explorer),
        ("Diana", Trait::Gourmet, JobPreference::Hauler),
        ("Evan", Trait::HardWorker, JobPreference::Explorer),
        ("Fiona", Trait::Lazy, JobPreference::Builder),
    ];

    for (i, (name, trait_data, job_pref)) in colonists_data.iter().enumerate() {
        let id = i as u32;
        // Simple spawn layout for now, in a row
        let position = Position::new(5 + i as i32, 5);

        let mut colonist = Colonist::new(id, name.to_string(), position, *trait_data, *job_pref);
        colonist.schedule = crate::data::schedule::Schedule::new_randomized(&mut state.rng);

        state.colonists.push(colonist);
    }

    seed_starting_relationships(state);
}

fn seed_starting_relationships(state: &mut GameState) {
    set_pair_relationship(state, 0, 5, -24);
    set_pair_relationship(state, 2, 4, 28);
    set_pair_relationship(state, 1, 3, 14);
}

fn set_pair_relationship(state: &mut GameState, first_id: u32, second_id: u32, value: i32) {
    for colonist in &mut state.colonists {
        if colonist.id == first_id {
            colonist.relationships.insert(second_id, value);
        } else if colonist.id == second_id {
            colonist.relationships.insert(first_id, value);
        }
    }
}

#[cfg(test)]
mod tests;
