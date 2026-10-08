//! Discovery: each landing draws its own native species, gathering them
//! inspires technologies, absent species never shape the tree, content is
//! always reachable, and orchards grow the food the colony knows best.

use finallanding::autoplay;
use finallanding::colony::tech_tree::TechStatus;
use finallanding::colony::RunSetup;
use finallanding::data::{game_data, NodeKind};
use finallanding::sim::research::inspire;
use finallanding::sim::Sim;
use finallanding::world::Calendar;
use std::collections::HashSet;

fn landing(site: &str, seed: u64) -> Sim {
    Sim::new(RunSetup {
        colony_name: "Test".into(),
        site: site.into(),
        difficulty: "standard".into(),
        seed,
    })
}

#[test]
fn each_landing_draws_and_places_its_own_species() {
    let mut draws = HashSet::new();
    for seed in 1..13 {
        let sim = landing("verdant_basin", seed);
        let species = &sim.world.species;
        draws.insert(species.clone());
        for id in species {
            assert!(
                sim.world
                    .nodes
                    .iter()
                    .any(|n| n.find.as_deref() == Some(id)),
                "seed {seed}: {id} was drawn but grows nowhere"
            );
        }
        for node in sim
            .world
            .nodes
            .iter()
            .filter(|n| n.kind != NodeKind::Wreckage)
        {
            let find = node
                .find
                .as_deref()
                .expect("every natural node has a species");
            assert!(
                species.iter().any(|s| s == find),
                "{find} is not native here"
            );
            assert_eq!(game_data().find(find).map(|f| f.node), Some(node.kind));
        }
    }
    assert!(
        draws.len() >= 10,
        "only {} distinct species draws in 12 landings",
        draws.len()
    );
}

#[test]
fn gathering_a_species_inspires_its_technologies() {
    let mut sim = landing("verdant_basin", 7);
    let food = sim
        .world
        .species
        .iter()
        .find(|id| game_data().find(id).is_some_and(|f| f.crop.is_some()))
        .cloned()
        .expect("every landing has a plantable food");
    let needed = game_data()
        .tech("native_cultivation")
        .expect("pool tech")
        .inspiration;
    assert_eq!(
        sim.colony.tree.status("native_cultivation"),
        TechStatus::Hidden
    );
    sim.colony.finds.record(&food, needed - 1.0, 1, "Ada");
    assert!(
        inspire(&mut sim.colony, &food).is_empty(),
        "not enough gathered yet"
    );
    assert_eq!(
        sim.colony.tree.status("native_cultivation"),
        TechStatus::Hidden
    );
    sim.colony.finds.record(&food, 1.0, 2, "Ada");
    let inspired = inspire(&mut sim.colony, &food);
    assert!(inspired.contains(&"native_cultivation".to_owned()));
    assert_eq!(
        sim.colony.tree.status("native_cultivation"),
        TechStatus::Available
    );
}

#[test]
fn absent_species_never_shape_the_tree_until_a_sample_arrives() {
    let sim = landing("ashen_steppe", 19);
    let tree = &sim.colony.tree;
    let mut absent = Vec::new();
    for def in game_data().techs.iter().filter(|t| t.is_inspired()) {
        let native = def
            .inspired_by
            .iter()
            .any(|f| sim.world.species.contains(f));
        assert_eq!(
            tree.status(&def.id) == TechStatus::Absent,
            !native,
            "{}",
            def.id
        );
        if !native {
            absent.push(def.id.clone());
        }
        let dependents = tree.nodes.iter().filter(|n| n.prereqs.contains(&def.id));
        assert_eq!(dependents.count(), 0, "{} is never a prerequisite", def.id);
        assert!(
            !tree.discoverable(None, true).contains(&def.id),
            "eurekas cannot find {}",
            def.id
        );
    }
    assert!(!absent.is_empty(), "a landing lacks some species");
    let foreign = game_data()
        .tech(&absent[0])
        .and_then(|t| t.inspired_by.first().cloned())
        .expect("inspired tech names a species");
    let mut tree = sim.colony.tree.clone();
    tree.introduce(&foreign);
    assert_eq!(
        tree.status(&absent[0]),
        TechStatus::Hidden,
        "a sample makes it findable"
    );
}

#[test]
fn technologies_that_unlock_content_can_always_be_inspired() {
    let data = game_data();
    let content: Vec<&str> = data
        .techs
        .iter()
        .filter(|t| t.is_inspired() && data.tech_unlocks_content(&t.id))
        .map(|t| t.id.as_str())
        .collect();
    assert!(content.contains(&"native_cultivation") && content.contains(&"smelting"));
    for site in &data.campaign.sites {
        for seed in 1..25 {
            let sim = landing(&site.id, seed);
            for id in &content {
                assert_ne!(
                    sim.colony.tree.status(id),
                    TechStatus::Absent,
                    "{} seed {seed}: {id} can never be inspired",
                    site.id
                );
            }
        }
    }
}

#[test]
fn colonies_find_species_in_different_orders_and_favour_what_they_gather() {
    let mut orders = HashSet::new();
    for seed in [3, 5, 8] {
        let mut sim = landing("verdant_basin", seed);
        let step = Calendar::ticks_per_day() / 6;
        while sim.calendar.day() < 10 {
            autoplay::plan(&mut sim);
            sim.run_ticks(step);
        }
        let order: Vec<String> = sim
            .colony
            .finds
            .found
            .iter()
            .map(|f| f.id.clone())
            .collect();
        assert!(order.len() >= 3, "seed {seed} found only {order:?}");
        orders.insert(order.into_iter().take(3).collect::<Vec<_>>());
        let favourite = sim.colony.finds.favourite_crop().map(str::to_owned);
        let most = sim
            .colony
            .finds
            .found
            .iter()
            .filter(|f| game_data().find(&f.id).is_some_and(|d| d.crop.is_some()))
            .map(|f| f.gathered)
            .fold(0.0, f32::max);
        if let Some(id) = &favourite {
            assert_eq!(
                sim.colony.finds.gathered(id),
                most,
                "favourite is the most gathered food"
            );
        }
    }
    assert!(
        orders.len() >= 2,
        "three colonies found species in the same order"
    );
}
