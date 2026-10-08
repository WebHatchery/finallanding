//! Campaign progression: objectives, acts, the Divergence vote and endings.

use finallanding::colony::campaign::Outcome;
use finallanding::colony::RunSetup;
use finallanding::data::campaign::ObjectiveKind;
use finallanding::data::{game_data, Resource};
use finallanding::sim::campaign::{evaluate, is_met};
use finallanding::sim::vote::projected_support;
use finallanding::sim::{environment, Sim};
use finallanding::world::Tile;

fn colony(seed: u64) -> Sim {
    Sim::new(RunSetup {
        colony_name: "Test".into(),
        site: "verdant_basin".into(),
        difficulty: "standard".into(),
        seed,
    })
}

fn build_instantly(sim: &mut Sim, building: &str) {
    let def = game_data().building(building).expect("known building");
    let origin = finallanding::autoplay::find_site(sim, building).unwrap_or(Tile::new(2, 2));
    sim.world.add_structure(def, origin, 1, true);
}

#[test]
fn objectives_read_the_colony_state() {
    let mut sim = colony(1);
    let beds = ObjectiveKind::Beds { count: 10 };
    assert!(!is_met(&sim, &beds));
    for _ in 0..3 {
        build_instantly(&mut sim, "survival_tent");
    }
    assert!(is_met(&sim, &beds));
    sim.colony.stock.set(Resource::Food, 500.0);
    assert!(is_met(
        &sim,
        &ObjectiveKind::Stock {
            resource: Resource::Food,
            amount: 80.0
        }
    ));
}

#[test]
fn completing_every_objective_opens_the_next_act() {
    let mut sim = colony(2);
    for _ in 0..3 {
        build_instantly(&mut sim, "survival_tent");
    }
    build_instantly(&mut sim, "mess_hall");
    build_instantly(&mut sim, "solar_array");
    sim.colony.stock.set(Resource::Food, 200.0);
    sim.world
        .map
        .explored
        .iter_mut()
        .for_each(|seen| *seen = true);
    for tech in [
        "foraging_lore",
        "field_survey",
        "prefab_shelters",
        "seed_vault_records",
        "salvage_protocols",
    ] {
        sim.colony.tree.complete(tech, 1);
    }
    sim.calendar.tick = finallanding::world::Calendar::ticks_per_day() / 2;
    environment::hourly(&mut sim);
    evaluate(&mut sim);
    assert_eq!(sim.colony.campaign.act, 2);
}

#[test]
fn the_divergence_vote_divides_the_colony() {
    let sim = colony(3);
    let mut supporters_somewhere = 0;
    let mut opponents_somewhere = 0;
    for ending in &game_data().campaign.endings {
        let support = projected_support(&sim, &ending.id);
        supporters_somewhere += support.iter().filter(|(_, s)| *s > 0.0).count();
        opponents_somewhere += support.iter().filter(|(_, s)| *s < 0.0).count();
    }
    assert!(
        supporters_somewhere > 0 && opponents_somewhere > 0,
        "futures compete for support"
    );
}

#[test]
fn the_colony_fails_when_nobody_is_left() {
    let mut sim = colony(4);
    for agent in sim.agents.iter_mut() {
        agent.life = finallanding::agents::LifeState::Dead {
            day: 1,
            cause: "cause_unknown".into(),
        };
    }
    evaluate(&mut sim);
    assert!(matches!(
        sim.colony.campaign.outcome,
        Some(Outcome::Failure { .. })
    ));
}

#[test]
fn enduring_after_the_capstone_wins_the_chosen_ending() {
    let mut sim = colony(5);
    sim.colony.campaign.act = 5;
    sim.colony.campaign.chosen_ending = Some("beacon".into());
    build_instantly(&mut sim, "beacon_array");
    sim.colony.campaign.capstone_built_day = Some(1);
    sim.calendar.tick = finallanding::world::Calendar::ticks_per_day() * 12;
    evaluate(&mut sim);
    assert!(
        matches!(&sim.colony.campaign.outcome, Some(Outcome::Victory { ending, .. }) if ending == "beacon")
    );
}
