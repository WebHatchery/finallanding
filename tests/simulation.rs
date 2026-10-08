//! Whole-colony simulation: determinism, persistence, commands, expeditions
//! and a scripted colony surviving its first weeks.

use finallanding::autoplay;
use finallanding::colony::RunSetup;
use finallanding::sim::commands::{apply, Command, CommandError};
use finallanding::sim::Sim;
use finallanding::world::{PlacementIssue, Tile};

fn colony(seed: u64) -> Sim {
    Sim::new(RunSetup {
        colony_name: "Test".into(),
        site: "frost_shelf".into(),
        difficulty: "standard".into(),
        seed,
    })
}

fn fingerprint(sim: &Sim) -> String {
    serde_json::to_string(sim).expect("simulation serializes")
}

#[test]
fn the_same_seed_plays_out_identically() {
    let mut first = colony(21);
    let mut second = colony(21);
    first.run_days(2);
    second.run_days(2);
    assert_eq!(fingerprint(&first), fingerprint(&second));
}

#[test]
fn a_saved_colony_restores_exactly() {
    let mut sim = colony(22);
    sim.run_days(1);
    let saved = fingerprint(&sim);
    let restored: Sim = serde_json::from_str(&saved).expect("save loads");
    assert_eq!(fingerprint(&restored), saved);
}

#[test]
fn blueprints_respect_the_map_and_refund_when_cancelled() {
    let mut sim = colony(23);
    let rejected = apply(
        &mut sim,
        Command::PlaceBlueprint {
            building: "survival_tent".into(),
            origin: Tile::new(0, 0),
        },
    );
    assert_eq!(
        rejected,
        Err(CommandError::Placement(PlacementIssue::Unexplored))
    );
    let landing = sim.world.landing_tile;
    let locked = apply(
        &mut sim,
        Command::PlaceBlueprint {
            building: "habitat_dome".into(),
            origin: landing,
        },
    );
    assert_eq!(locked, Err(CommandError::Locked));
    let origin = autoplay::find_site(&sim, "survival_tent").expect("room near the hull");
    apply(
        &mut sim,
        Command::PlaceBlueprint {
            building: "survival_tent".into(),
            origin,
        },
    )
    .expect("placed");
    let id = sim.world.structures.last().expect("blueprint exists").id;
    sim.world
        .structure_mut(id)
        .expect("blueprint")
        .delivered
        .set(finallanding::data::Resource::Fibre, 5.0);
    let before = sim.colony.stock.get(finallanding::data::Resource::Fibre);
    apply(&mut sim, Command::CancelBlueprint(id)).expect("cancelled");
    assert_eq!(
        sim.colony.stock.get(finallanding::data::Resource::Fibre),
        before + 5.0
    );
}

#[test]
fn volunteers_go_on_expeditions_and_come_home() {
    let mut sim = colony(24);
    let def = finallanding::data::game_data()
        .building("expedition_gate")
        .expect("gate");
    let origin = autoplay::find_site(&sim, "expedition_gate").expect("room for a gate");
    sim.world.add_structure(def, origin, 1, true);
    for agent in sim.agents.iter_mut() {
        agent.personality.bravery = 1.0;
        agent.personality.curiosity = 1.0;
    }
    apply(&mut sim, Command::OpenExpedition("fibre_valley".into())).expect("call opened");
    sim.run_days(5);
    assert!(
        sim.colony.expeditions.completed >= 1,
        "a party went out and returned"
    );
    assert!(!sim.colony.expeditions.reports.is_empty());
}

#[test]
fn a_scripted_colony_survives_and_grows_through_its_first_weeks() {
    let mut sim = colony(25);
    for _ in 0..(20 * 6) {
        autoplay::plan(&mut sim);
        sim.run_ticks(120);
    }
    assert!(sim.population() >= 8, "nobody starved or froze");
    assert!(
        sim.world.built().count() >= 8,
        "survivors built what was planned"
    );
    assert!(sim.colony.tree.researched_count() >= 3, "research advanced");
    assert!(
        sim.colony.chronicle.reports.len() >= 19,
        "the colony AI wrote daily reports"
    );
}
