//! colonist spawner domain.

use crate::data::colonist::Colonist;
use crate::data::game_state::GameState;
use crate::data::types::Position;

pub fn spawn_initial_colonists(state: &mut GameState) {
    for survivor in &crate::data::config::game_config().survivors {
        let trait_data = crate::data::colonist::Trait::from_id(&survivor.trait_data)
            .expect("validated survivor trait must be recognized");
        let job_pref = crate::data::colonist::JobPreference::from_id(&survivor.job)
            .expect("validated survivor job must be recognized");
        let position = Position::new(survivor.position[0], survivor.position[1]);

        let mut colonist = Colonist::new(
            survivor.id,
            survivor.name.clone(),
            position,
            trait_data,
            job_pref,
        );
        colonist.schedule = crate::data::schedule::Schedule::new_randomized(&mut state.rng);

        state.colonists.push(colonist);
    }

    seed_starting_relationships(state);
}

fn seed_starting_relationships(state: &mut GameState) {
    for relationship in &crate::data::config::game_config().starting_relationships {
        set_pair_relationship(
            state,
            relationship.first_id,
            relationship.second_id,
            relationship.value,
        );
    }
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
