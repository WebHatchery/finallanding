//! A scripted colony AI that plays the campaign through the same commands as
//! the player. Used by the headless campaign runner and tests to verify that
//! a run is completable and paced as intended.

pub mod report;

use crate::data::{game_data, Branch, Resource};
use crate::sim::campaign::is_met;
use crate::sim::commands::{apply, is_unlocked, Command};
use crate::sim::expeditions::available_sites;
use crate::sim::vote::projected_support;
use crate::sim::Sim;
use crate::world::{Footprint, Tile};

const MAX_PENDING_BLUEPRINTS: usize = 3;

/// Find a free origin near the landing site, keeping a one-tile gap so
/// survivors can walk between buildings.
pub fn find_site(sim: &Sim, building: &str) -> Option<Tile> {
    let def = game_data().building(building)?;
    let center = sim.world.landing_tile;
    for radius in 3i32..40 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let origin = center.offset(dx, dy);
                if sim.world.check_placement(def, origin).is_err() {
                    continue;
                }
                let padded = Footprint {
                    origin: origin.offset(-1, -1),
                    width: def.size[0] + 2,
                    height: def.size[1] + 2,
                };
                let clear = padded.tiles().all(|tile| {
                    sim.world.map.structure_on(tile).is_none()
                        && sim.world.map.node_on(tile).is_none()
                });
                if clear {
                    return Some(origin);
                }
            }
        }
    }
    None
}

fn pending(sim: &Sim) -> usize {
    sim.world
        .structures
        .iter()
        .filter(|s| !s.is_built())
        .count()
}

fn count(sim: &Sim, building: &str) -> usize {
    sim.world
        .structures
        .iter()
        .filter(|s| s.kind == building)
        .count()
}

fn place(sim: &mut Sim, building: &str) -> bool {
    if pending(sim) >= MAX_PENDING_BLUEPRINTS || !is_unlocked(sim, building) {
        return false;
    }
    let Some(origin) = find_site(sim, building) else {
        return false;
    };
    apply(
        sim,
        Command::PlaceBlueprint {
            building: building.to_owned(),
            origin,
        },
    )
    .is_ok()
}

fn first_unlocked<'a>(sim: &Sim, options: &[&'a str]) -> Option<&'a str> {
    options.iter().copied().find(|b| is_unlocked(sim, b))
}

fn beds(sim: &Sim) -> u32 {
    sim.world.structures.iter().map(|s| s.def().beds).sum()
}

fn plan_essentials(sim: &mut Sim) {
    let population = sim.population() as u32;
    if beds(sim) < population + 2 {
        if let Some(home) = first_unlocked(sim, &["habitat_dome", "bunkhouse", "survival_tent"]) {
            place(sim, home);
        }
    }
    if count(sim, "mess_hall") == 0 {
        place(sim, "mess_hall");
    }
    if count(sim, "campfire") == 0 {
        place(sim, "campfire");
    }
    if count(sim, "research_bench") < 1 + population as usize / 8 {
        place(sim, "research_bench");
    }
    let power = &sim.colony.power;
    if power.capacity < power.demand + 4.0 || count(sim, "solar_array") == 0 {
        let source = first_unlocked(sim, &["geothermal_tap", "fibre_burner", "solar_array"])
            .unwrap_or("solar_array");
        let source = if source == "fibre_burner" && count(sim, "fibre_burner") >= 2 {
            "solar_array"
        } else {
            source
        };
        place(sim, source);
    }
    let farms =
        count(sim, "farm_plot") + count(sim, "glowfruit_orchard") + count(sim, "hydroponics_bay");
    if farms < (population as usize).div_ceil(3).max(3) {
        let farm = if sim.calendar.season() == crate::data::Season::Autumn {
            first_unlocked(sim, &["hydroponics_bay", "glowfruit_orchard", "farm_plot"])
        } else {
            first_unlocked(sim, &["glowfruit_orchard", "farm_plot"])
        };
        if let Some(farm) = farm {
            place(sim, farm);
        }
    }
    if sim.colony.stock.total() > sim.world.storage_capacity() * 0.75 {
        let store = first_unlocked(sim, &["warehouse", "storage_depot"]).unwrap_or("storage_depot");
        place(sim, store);
    }
}

fn plan_growth(sim: &mut Sim) {
    let population = sim.population();
    for (building, wanted) in [
        ("workshop", 1),
        ("infirmary", 1),
        ("expedition_gate", 1),
        ("smelter", 1),
        ("laboratory", 1 + population / 14),
        ("lounge", 1),
        ("garden", population / 8),
        ("fabricator", 1),
        ("xeno_lab", 1),
        ("watchtower", 2),
        ("family_quarters", population / 8),
        ("hospital", 1),
    ] {
        if count(sim, building) < wanted {
            place(sim, building);
        }
    }
    if sim.colony.stats.deaths > 0 && count(sim, "memorial") == 0 {
        place(sim, "memorial");
    }
    if sim.colony.campaign.act >= 3 {
        place_if_missing(sim, "signal_mast");
    }
    if sim.colony.campaign.act >= 4 {
        place_if_missing(sim, "council_hall");
        if sim.colony.stats.creatures_repelled > 4 && count(sim, "arc_turret") < 2 {
            place(sim, "arc_turret");
        }
    }
}

fn place_if_missing(sim: &mut Sim, building: &str) {
    if count(sim, building) == 0 {
        place(sim, building);
    }
}

/// Research what the campaign needs next, otherwise the cheapest option.
fn choose_research(sim: &mut Sim) {
    let available = sim.colony.tree.available();
    if available.is_empty() {
        return;
    }
    let ending = sim.colony.campaign.chosen_ending.clone();
    let capstone_path: Vec<String> = ending
        .and_then(|e| game_data().ending(&e))
        .map(|e| {
            let mut path = vec![e.capstone_tech.clone()];
            if let Some(node) = sim.colony.tree.node(&e.capstone_tech) {
                path.extend(node.prereqs.iter().cloned());
            }
            path
        })
        .unwrap_or_default();
    let wanted = [
        "signal_analysis",
        "xenoarchaeology",
        "civic_assembly",
        "field_survey",
        "prefab_shelters",
        "field_medicine",
    ];
    let scale = sim.colony.difficulty_value(|d| d.research_cost);
    let score = |id: &str| -> f32 {
        let mut cost = sim.colony.tree.cost(id, scale);
        if capstone_path.iter().any(|p| p == id) {
            cost *= 0.1;
        }
        if wanted.contains(&id) {
            cost *= 0.3;
        }
        if game_data()
            .tech(id)
            .is_some_and(|t| t.branch == Branch::Xenology)
            && sim.colony.campaign.act == 3
        {
            cost *= 0.5;
        }
        cost
    };
    let pick = available
        .iter()
        .min_by(|a, b| score(a).total_cmp(&score(b)))
        .map(|id| id.to_string());
    if sim.colony.research.focus.is_none() {
        let _ = apply(sim, Command::SetResearchFocus(pick));
    }
}

fn manage_expeditions(sim: &mut Sim) {
    let has_gate = sim.world.built().any(|s| s.def().expedition);
    let busy = sim.colony.expeditions.call.is_some() || !sim.colony.expeditions.active.is_empty();
    if !has_gate || busy || sim.present().count() < 6 {
        return;
    }
    let needs_relics = sim.colony.campaign.act >= 2;
    let sites = available_sites(sim);
    let site = sites
        .iter()
        .filter(|s| s.danger <= 0.3 + sim.colony.campaign.act as f32 * 0.12)
        .max_by(|a, b| {
            let value = |s: &&&crate::data::events::ExpeditionSiteDef| {
                let freshness = sim.colony.expeditions.reward_factor(&s.id);
                (s.relics * if needs_relics { 30.0 } else { 0.0 }
                    + if s.recruits { 25.0 } else { 0.0 }
                    + s.rewards.len() as f32)
                    * freshness
            };
            value(a).total_cmp(&value(b))
        })
        .map(|s| s.id.clone());
    if let Some(site) = site {
        let _ = apply(sim, Command::OpenExpedition(site));
    }
}

fn manage_future(sim: &mut Sim) {
    if sim.colony.campaign.act < 4
        || sim.colony.campaign.vote.is_some()
        || sim.world.count_built("council_hall") == 0
    {
        return;
    }
    let best = game_data()
        .campaign
        .endings
        .iter()
        .map(|e| {
            let support: f32 = projected_support(sim, &e.id).iter().map(|(_, s)| s).sum();
            (e.id.clone(), support)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1));
    if let Some((ending, _)) = best {
        let _ = apply(sim, Command::ProposeFuture(ending));
    }
}

fn manage_capstone(sim: &mut Sim) {
    let Some(ending) = sim
        .colony
        .campaign
        .chosen_ending
        .clone()
        .and_then(|e| game_data().ending(&e))
    else {
        return;
    };
    if sim.colony.tree.is_researched(&ending.capstone_tech)
        && count(sim, &ending.capstone_building) == 0
    {
        place(sim, &ending.capstone_building);
    }
}

/// Keep food and comfort policies sensible for the colony's situation.
fn manage_policies(sim: &mut Sim) {
    let population = sim.population().max(1) as f32;
    let food = sim.colony.stock.get(Resource::Food) + sim.colony.stock.get(Resource::Meals);
    let rations = if food < population * 4.0 {
        "strict"
    } else if food > population * 25.0 {
        "generous"
    } else {
        "standard"
    };
    let _ = apply(
        sim,
        Command::SetPolicy {
            policy: "rations".into(),
            option: rations.into(),
        },
    );
}

/// One planning pass; call a few times per in-game day.
pub fn plan(sim: &mut Sim) {
    plan_with(sim, true);
}

/// A planning pass that leaves the Divergence vote to someone else.
pub fn plan_without_vote(sim: &mut Sim) {
    plan_with(sim, false);
}

fn plan_with(sim: &mut Sim, vote: bool) {
    if sim.colony.campaign.is_over() {
        return;
    }
    plan_essentials(sim);
    plan_growth(sim);
    choose_research(sim);
    manage_expeditions(sim);
    if vote {
        manage_future(sim);
    }
    manage_capstone(sim);
    manage_policies(sim);
}

/// Whether every objective of the current act is met (for reports).
pub fn act_progress(sim: &Sim) -> (usize, usize) {
    let Some(act) = game_data().act(sim.colony.campaign.act) else {
        return (0, 0);
    };
    let done = act
        .objectives
        .iter()
        .filter(|o| is_met(sim, &o.kind))
        .count();
    (done, act.objectives.len())
}
