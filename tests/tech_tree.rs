//! The dynamic technology tree: reshaped per seed, forks, discovery and cost.

use finallanding::colony::tech_tree::{TechStatus, TechTree};
use finallanding::data::{game_data, Branch};
use macroquad_toolkit::rng::SeededRng;

fn prereq_signature(tree: &TechTree) -> Vec<(String, Vec<String>)> {
    tree.nodes
        .iter()
        .map(|n| (n.id.clone(), n.prereqs.clone()))
        .collect()
}

#[test]
fn the_tree_is_reproducible_per_seed_and_different_between_seeds() {
    let first = TechTree::generate(&mut SeededRng::new(1));
    let again = TechTree::generate(&mut SeededRng::new(1));
    let other = TechTree::generate(&mut SeededRng::new(2));
    assert_eq!(prereq_signature(&first), prereq_signature(&again));
    assert_ne!(prereq_signature(&first), prereq_signature(&other));
    let hidden = |tree: &TechTree| {
        tree.nodes
            .iter()
            .filter(|n| !n.revealed)
            .map(|n| n.id.clone())
            .collect::<Vec<_>>()
    };
    assert!(
        !hidden(&first).is_empty(),
        "some technologies start undiscovered"
    );
}

#[test]
fn prerequisites_always_come_from_earlier_tiers() {
    let data = game_data();
    for seed in 1..20 {
        let tree = TechTree::generate(&mut SeededRng::new(seed));
        for node in &tree.nodes {
            let tier = data.tech(&node.id).expect("known tech").tier;
            for prereq in &node.prereqs {
                assert!(
                    data.tech(prereq).expect("known prereq").tier < tier,
                    "{} depends on {}",
                    node.id,
                    prereq
                );
            }
        }
    }
}

#[test]
fn researching_a_fork_locks_its_alternative() {
    let mut tree = TechTree::generate(&mut SeededRng::new(5));
    let locked = tree.complete("fungal_symbiosis", 10);
    assert_eq!(locked, vec!["gene_tailored_crops".to_owned()]);
    assert_eq!(tree.status("gene_tailored_crops"), TechStatus::Locked);
    assert_eq!(tree.status("fungal_symbiosis"), TechStatus::Researched);
}

#[test]
fn capstones_are_visible_and_relic_knowledge_must_be_found() {
    let tree = TechTree::generate(&mut SeededRng::new(8));
    for ending in &game_data().campaign.endings {
        assert!(tree.node(&ending.capstone_tech).is_some_and(|n| n.revealed));
    }
    let relic_only = game_data().techs.iter().filter(|t| t.relic_only);
    for tech in relic_only {
        assert!(
            !tree.node(&tech.id).is_some_and(|n| n.revealed),
            "{} starts hidden",
            tech.id
        );
        assert!(
            !tree.discoverable(None, false).contains(&tech.id),
            "eurekas cannot reveal relic knowledge"
        );
    }
}

#[test]
fn practice_in_a_branch_makes_its_research_cheaper() {
    let mut tree = TechTree::generate(&mut SeededRng::new(4));
    let before = tree.cost("hydroponics", 1.0);
    for id in ["seed_vault_records", "native_cultivation", "soil_chemistry"] {
        tree.complete(id, 1);
    }
    assert_eq!(tree.researched_in(Branch::Agronomy), 3);
    assert!(tree.cost("hydroponics", 1.0) < before);
}
