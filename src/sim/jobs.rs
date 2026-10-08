//! Rebuilding the job board from the colony's current state.

use super::Sim;
use crate::colony::jobs::{BuildJob, JobBoard};
use crate::data::{game_data, Branch, NodeKind, Resource, ResourceBag};
use crate::world::CropStage;

/// Pick each station's recipe: researched, affordable and below its target.
fn assign_recipes(sim: &mut Sim) -> Vec<u32> {
    let data = game_data();
    let mut stations = Vec::new();
    let stock = sim.colony.stock;
    for structure in sim.world.structures.iter_mut().filter(|s| s.is_built()) {
        let Some(station) = structure.def().station.clone() else {
            continue;
        };
        if structure.craft_progress > 0.0 && structure.craft_recipe.is_some() {
            stations.push(structure.id);
            continue;
        }
        let recipe = data
            .recipes
            .iter()
            .filter(|r| r.station == station)
            .filter(|r| {
                r.tech
                    .as_ref()
                    .is_none_or(|t| sim.colony.tree.is_researched(t))
            })
            .filter(|r| stock.covers(&r.input_bag()))
            .filter(|r| {
                r.main_output()
                    .is_some_and(|out| stock.get(out) < r.target_stock)
            })
            .min_by(|a, b| {
                let fill = |r: &crate::data::buildings::RecipeDef| {
                    r.main_output()
                        .map(|o| stock.get(o) / r.target_stock)
                        .unwrap_or(1.0)
                };
                fill(a).total_cmp(&fill(b))
            });
        structure.craft_recipe = recipe.map(|r| r.id.clone());
        if structure.craft_recipe.is_some() {
            stations.push(structure.id);
        }
    }
    stations
}

/// How much the colony wants more of each resource, 0 (plenty) to 1.5.
fn resource_wants(sim: &Sim, builds: &ResourceBag) -> ResourceBag {
    let population = sim.population().max(1) as f32;
    let act = sim.colony.campaign.act as f32;
    let mut target = ResourceBag::default();
    target.set(Resource::Food, 18.0 * population);
    target.set(Resource::Salvage, 80.0 + act * 20.0);
    target.set(Resource::Fibre, 50.0 + act * 25.0);
    target.set(Resource::Stone, 30.0 + act * 40.0);
    target.set(Resource::Metal, 30.0 + act * 30.0);
    target.set(
        Resource::Relics,
        if act >= 2.0 { 6.0 + act * 3.0 } else { 0.0 },
    );
    target.add_bag(builds);
    let mut want = ResourceBag::default();
    for resource in crate::data::Resource::ALL {
        let goal = target.get(resource);
        if goal > 0.0 {
            let have = sim.colony.stock.get(resource);
            want.set(resource, ((goal - have) / goal).clamp(0.0, 1.5));
        }
    }
    want
}

fn node_resource(kind: NodeKind) -> Resource {
    crate::agents::goals::gathered_resource(kind)
}

pub fn refresh(sim: &mut Sim) {
    let crafts = assign_recipes(sim);
    let mut board = JobBoard::default();
    let mut build_needs = ResourceBag::default();
    for structure in sim.world.structures.iter().filter(|s| !s.is_built()) {
        let missing = structure.materials_missing();
        build_needs.add_bag(&missing);
        board.builds.push(BuildJob {
            site: structure.id,
            missing,
            ready_to_build: missing.is_empty(),
        });
    }
    let season = sim.calendar.season();
    let growth = game_data().balance.farming.season_growth[season.index()];
    for structure in sim.world.built() {
        if structure.needs_repair() {
            board.repairs.push(structure.id);
        }
        if let Some(crop) = &structure.crop {
            let indoor = structure.def().farm.as_ref().is_some_and(|f| f.indoor);
            let plantable = crop.stage == CropStage::Fallow && (indoor || growth > 0.0);
            if plantable || crop.stage == CropStage::Ripe {
                board.farms.push(structure.id);
            }
        }
        if structure.def().kitchen && structure.is_operational() {
            board.kitchens.push(structure.id);
        }
        if structure.def().research > 0.0 {
            board.labs.push(structure.id);
        }
    }
    board.crafts = crafts;
    board.items = sim.world.items.iter().map(|i| i.tile).collect();
    let population = sim.population().max(1) as f32;
    let meal_target = game_data().balance.work.meal_stock_per_person * population;
    let meals = sim.colony.stock.get(Resource::Meals);
    board.cook_demand = ((meal_target - meals) / meal_target.max(1.0)).clamp(0.0, 1.5);
    board.research_open = !sim.colony.tree.available().is_empty();
    // Relics a blueprint is waiting for are not spent on study.
    let reserved = build_needs.get(Resource::Relics);
    board.relic_study = sim.colony.stock.get(Resource::Relics) >= reserved + 1.0
        && sim
            .world
            .built()
            .any(|s| s.def().research_branch == Some(Branch::Xenology) && s.def().indoor);
    board.patients = sim
        .present()
        .filter(|a| a.health.needs_care() && !a.health.treated)
        .map(|a| a.id)
        .collect();
    let aggressive = |kind| {
        game_data()
            .balance
            .creatures
            .get(&kind)
            .is_some_and(|c| c.aggressive)
    };
    board.threats = sim
        .world
        .creatures
        .iter()
        .filter(|c| sim.present().any(|a| a.tile.distance(c.tile) < 16.0))
        .map(|c| (c.id, c.tile, aggressive(c.kind)))
        .collect();
    let full = sim.colony.stock.total() >= sim.world.storage_capacity();
    board.storage_full = full;
    let want = resource_wants(sim, &build_needs);
    board.gather_demand = NodeKind::ALL
        .into_iter()
        .map(|kind| {
            let demand = want.get(node_resource(kind)) * if full { 0.2 } else { 1.0 };
            (kind, demand)
        })
        .collect();
    sim.colony.jobs = board;
}
