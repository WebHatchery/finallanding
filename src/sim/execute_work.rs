//! Work steps: stores, construction, gathering, farming, cooking, crafting
//! and research.

use super::chronicle_text::line;
use super::research::{add_insight, contribute};
use super::Ctx;
use crate::agents::deliberation::gather_skill;
use crate::agents::plans::{FailReason, Step, StepOutcome};
use crate::agents::{Activity, Agent};
use crate::colony::chronicle::Category;
use crate::data::{game_data, Branch, Need, Resource, Skill};
use crate::world::{Calendar, CropStage, StructureId};

const NEAR: f32 = 1.6;

fn near_structure(agent: &Agent, ctx: &Ctx, id: StructureId) -> bool {
    ctx.world
        .structure(id)
        .is_some_and(|s| s.footprint.distance_to(agent.tile) <= NEAR)
}

fn near_storage(agent: &Agent, ctx: &Ctx, kitchen: bool) -> bool {
    ctx.world.built().any(|s| {
        let def = s.def();
        let fits = if kitchen {
            def.kitchen
        } else {
            def.storage > 0.0
        };
        fits && s.footprint.distance_to(agent.tile) <= NEAR
    })
}

/// Speed multiplier for work in a skill, including friends working nearby.
pub fn work_rate(agent: &Agent, ctx: &Ctx, skill: Skill, outdoors: bool) -> f32 {
    let mut rate = agent.skills.speed(skill, &agent.personality)
        * ctx.colony.modifiers.work_speed[skill.index()]
        * agent.health.capacity();
    for effects in ctx.colony.policy_effects() {
        rate *= effects.work_speed;
    }
    if outdoors && ctx.world.weather.current.is_severe() {
        rate *= game_data().balance.work.severe_weather_outdoor_factor;
    }
    let friend_near =
        agent.beliefs.nearby.iter().any(|(id, tile)| {
            tile.distance(agent.tile) < 4.0 && agent.beliefs.opinion_of(*id) > 30.0
        });
    if friend_near {
        rate *= 1.1;
    }
    rate
}

/// Gain experience and insight; announce milestones.
pub fn practise(agent: &mut Agent, ctx: &mut Ctx, skill: Skill) {
    if agent.skills.practise(skill, 1.0) {
        agent.mind.add("learned", None, Calendar::ticks_per_day());
        let level = agent.skills.level(skill);
        if level.is_multiple_of(5) {
            let label = game_data().label(skill.key()).to_owned();
            let text = line(
                "skill_milestone",
                ctx.calendar.tick,
                &[
                    ("name", &agent.given_name),
                    ("skill", &label),
                    ("level", &level.to_string()),
                ],
            );
            ctx.log(Category::Life, 0, text, vec![agent.id]);
        }
    }
    let amount = game_data().balance.research.insight_per_tick;
    add_insight(agent, ctx, skill.branch(), amount);
}

fn set_carry(agent: &mut Agent, ctx: &mut Ctx, resource: Resource, amount: f32) {
    if let Some((carried, held)) = agent.carrying {
        if carried != resource {
            ctx.world.drop_item(agent.tile, carried, held);
            agent.carrying = None;
        }
    }
    let current = agent.carried(resource);
    agent.carrying = Some((resource, current + amount));
}

fn take_stock(agent: &mut Agent, ctx: &mut Ctx, resource: Resource, amount: f32) -> StepOutcome {
    if !near_storage(agent, ctx, false) {
        return StepOutcome::Failed(FailReason::TargetGone);
    }
    let capacity = agent.carry_capacity(ctx.colony.modifiers.carry);
    let mut wanted = amount.min(capacity);
    if resource == Resource::Food {
        wanted *= ctx.colony.food_use();
    }
    let taken = ctx.colony.stock.take(resource, wanted);
    agent.beliefs.stock = ctx.colony.stock;
    if taken + 0.001 < wanted.min(0.5) {
        ctx.colony.stock.add(resource, taken);
        return StepOutcome::Failed(FailReason::StockEmpty);
    }
    set_carry(agent, ctx, resource, taken);
    StepOutcome::Done
}

fn take_meal(agent: &mut Agent, ctx: &mut Ctx) -> StepOutcome {
    if !near_storage(agent, ctx, true) {
        return StepOutcome::Failed(FailReason::TargetGone);
    }
    let taken = ctx
        .colony
        .stock
        .take(Resource::Meals, ctx.colony.food_use());
    agent.beliefs.stock = ctx.colony.stock;
    if taken < 0.5 {
        return StepOutcome::Failed(FailReason::StockEmpty);
    }
    set_carry(agent, ctx, Resource::Meals, taken);
    StepOutcome::Done
}

fn deposit(agent: &mut Agent, ctx: &mut Ctx) -> StepOutcome {
    let Some((resource, amount)) = agent.carrying else {
        return StepOutcome::Done;
    };
    if !near_storage(agent, ctx, false) {
        return StepOutcome::Failed(FailReason::TargetGone);
    }
    let room = (ctx.world.storage_capacity() - ctx.colony.stock.total()).max(0.0);
    let stored = amount.min(room);
    ctx.colony.stock.add(resource, stored);
    if resource == Resource::Relics {
        ctx.colony.stats.relics_found += stored;
    }
    if amount - stored > 0.01 {
        ctx.world.drop_item(agent.tile, resource, amount - stored);
    }
    agent.carrying = None;
    agent.beliefs.stock = ctx.colony.stock;
    StepOutcome::Done
}

fn deliver(agent: &mut Agent, ctx: &mut Ctx, site: StructureId) -> StepOutcome {
    let Some((resource, amount)) = agent.carrying else {
        return StepOutcome::Done;
    };
    let Some(structure) = ctx.world.structure_mut(site) else {
        ctx.world.drop_item(agent.tile, resource, amount);
        agent.carrying = None;
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let missing = structure.materials_missing().get(resource);
    let given = amount.min(missing);
    structure.delivered.add(resource, given);
    agent.carrying = None;
    if amount - given > 0.01 {
        ctx.world.drop_item(agent.tile, resource, amount - given);
    }
    StepOutcome::Done
}

fn construct(agent: &mut Agent, ctx: &mut Ctx, site: StructureId) -> StepOutcome {
    if !near_structure(agent, ctx, site) {
        return StepOutcome::Failed(FailReason::TargetGone);
    }
    let rate = work_rate(agent, ctx, Skill::Construction, true);
    let per_tick = game_data().balance.work.construction_per_tick;
    let Some(structure) = ctx.world.structure_mut(site) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    if structure.is_built() {
        return StepOutcome::Done;
    }
    if !structure.has_all_materials() {
        return StepOutcome::Failed(FailReason::StockEmpty);
    }
    agent.activity = Activity::Working(Skill::Construction);
    structure.progress += per_tick * rate;
    let finished = structure.progress >= structure.def().work;
    let def = structure.def();
    practise(agent, ctx, Skill::Construction);
    if finished {
        ctx.world.complete_structure(site);
        ctx.colony.stats.structures_built += 1;
        agent.stats.structures_built += 1;
        let importance = if def.capstone.is_some() { 2 } else { 1 };
        let text = line(
            "structure_built",
            ctx.calendar.tick,
            &[("building", &def.name), ("name", &agent.given_name)],
        );
        ctx.log(Category::Colony, importance, text, vec![agent.id]);
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn repair(agent: &mut Agent, ctx: &mut Ctx, site: StructureId) -> StepOutcome {
    if !near_structure(agent, ctx, site) {
        return StepOutcome::Failed(FailReason::TargetGone);
    }
    let rate = work_rate(agent, ctx, Skill::Construction, true);
    let per_tick = game_data().balance.work.repair_per_tick;
    let Some(structure) = ctx.world.structure_mut(site) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    agent.activity = Activity::Working(Skill::Construction);
    structure.condition = (structure.condition + per_tick * rate).min(100.0);
    let done = structure.condition >= 99.0;
    practise(agent, ctx, Skill::Construction);
    if done {
        StepOutcome::Done
    } else {
        StepOutcome::Running
    }
}

fn harvest(agent: &mut Agent, ctx: &mut Ctx, node_id: u32) -> StepOutcome {
    let Some(node) = ctx.world.node(node_id) else {
        return StepOutcome::Failed(FailReason::Depleted);
    };
    if node.tile.steps(agent.tile) > 1 {
        return StepOutcome::Failed(FailReason::TargetGone);
    }
    let (kind, resource) = (node.kind, node.resource());
    if node.is_depleted() {
        return StepOutcome::Failed(FailReason::Depleted);
    }
    let skill = gather_skill(kind);
    let base = game_data()
        .balance
        .work
        .harvest_per_tick
        .get(&resource)
        .copied()
        .unwrap_or(0.2);
    let rate = base * work_rate(agent, ctx, skill, true);
    let capacity = agent.carry_capacity(ctx.colony.modifiers.carry);
    let room = (capacity - agent.carried(resource)).max(0.0);
    let Some(node) = ctx.world.node_mut(node_id) else {
        return StepOutcome::Failed(FailReason::Depleted);
    };
    let taken = rate.min(node.amount).min(room);
    node.amount -= taken;
    let depleted = node.is_depleted();
    let regrows = node.regrows();
    let amount = node.amount;
    agent.activity = Activity::Working(skill);
    set_carry(agent, ctx, resource, taken);
    practise(agent, ctx, skill);
    if resource == Resource::Relics {
        add_insight(agent, ctx, Branch::Xenology, 0.05);
    }
    if let Some(belief) = agent.beliefs.nodes.get_mut(&node_id) {
        belief.amount = amount;
    }
    if depleted && !regrows {
        ctx.world.remove_node(node_id);
        agent.beliefs.forget_node(node_id);
    }
    let full = agent.carried(resource) >= capacity - 0.01;
    let enough_relics = resource == Resource::Relics && agent.carried(resource) >= 1.0;
    if full || depleted || enough_relics {
        if agent.carried(resource) > 0.01 {
            return StepOutcome::Done;
        }
        return StepOutcome::Failed(FailReason::Depleted);
    }
    StepOutcome::Running
}

fn forage(agent: &mut Agent, ctx: &mut Ctx, node_id: u32, step_ticks: u32) -> StepOutcome {
    let Some(node) = ctx.world.node_mut(node_id) else {
        return StepOutcome::Failed(FailReason::Depleted);
    };
    if node.amount < 1.0 {
        return StepOutcome::Failed(FailReason::Depleted);
    }
    agent.activity = Activity::Eating;
    if step_ticks < 8 {
        return StepOutcome::Running;
    }
    node.amount -= 1.0;
    let restore = game_data().balance.needs.forage_restore;
    agent.needs.change(Need::Food, restore);
    agent
        .mind
        .add("ate_foraged", None, Calendar::ticks_per_day());
    StepOutcome::Done
}

fn pick_up(agent: &mut Agent, ctx: &mut Ctx, tile: crate::world::Tile) -> StepOutcome {
    let capacity = agent.carry_capacity(ctx.colony.modifiers.carry);
    let Some(index) = ctx.world.items.iter().position(|i| i.tile == tile) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let item = &mut ctx.world.items[index];
    let resource = item.resource;
    let taken = item.amount.min(capacity);
    item.amount -= taken;
    if item.amount < 0.01 {
        ctx.world.items.remove(index);
    }
    set_carry(agent, ctx, resource, taken);
    StepOutcome::Done
}

fn tend(agent: &mut Agent, ctx: &mut Ctx, plot: StructureId) -> StepOutcome {
    let farming = &game_data().balance.farming;
    let rate = work_rate(agent, ctx, Skill::Farming, true);
    let fertility = ctx
        .world
        .structure(plot)
        .map(|s| ctx.world.map.fertility_at(s.footprint.origin))
        .unwrap_or(30.0);
    let yield_scale = ctx.colony.modifiers.farm_yield;
    let Some(structure) = ctx.world.structure_mut(plot) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let spec_yield = structure
        .def()
        .farm
        .as_ref()
        .map(|f| f.yield_food)
        .unwrap_or(0.0);
    let Some(crop) = structure.crop.as_mut() else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    agent.activity = Activity::Working(Skill::Farming);
    match crop.stage {
        CropStage::Growing => return StepOutcome::Done,
        CropStage::Fallow => {
            crop.work += rate;
            if crop.work >= farming.plant_work {
                crop.stage = CropStage::Growing;
                crop.growth = 0.0;
                crop.work = 0.0;
            }
        }
        CropStage::Ripe => {
            crop.work += rate;
            if crop.work >= farming.harvest_work {
                crop.stage = CropStage::Fallow;
                crop.work = 0.0;
                crop.growth = 0.0;
                let food = spec_yield * (0.6 + fertility / 125.0) * yield_scale;
                let capacity = agent.carry_capacity(ctx.colony.modifiers.carry);
                let carried = food.min(capacity);
                let tile = agent.tile;
                ctx.world.drop_item(tile, Resource::Food, food - carried);
                set_carry(agent, ctx, Resource::Food, carried);
                practise(agent, ctx, Skill::Farming);
                return StepOutcome::Done;
            }
        }
    }
    let done = crop.stage == CropStage::Growing;
    practise(agent, ctx, Skill::Farming);
    if done {
        StepOutcome::Done
    } else {
        StepOutcome::Running
    }
}

fn cook(agent: &mut Agent, ctx: &mut Ctx, kitchen: StructureId, step_ticks: u32) -> StepOutcome {
    let food = agent.carried(Resource::Food);
    if food < 0.5 {
        return StepOutcome::Failed(FailReason::StockEmpty);
    }
    if !near_structure(agent, ctx, kitchen) {
        return StepOutcome::Failed(FailReason::TargetGone);
    }
    agent.inside = Some(kitchen);
    agent.activity = Activity::Working(Skill::Cooking);
    practise(agent, ctx, Skill::Cooking);
    let rate = work_rate(agent, ctx, Skill::Cooking, false);
    let needed = game_data().balance.work.cook_ticks as f32 / rate.max(0.2);
    if (step_ticks as f32) < needed {
        return StepOutcome::Running;
    }
    let meals = food * game_data().balance.work.meals_per_food * (0.9 + rate * 0.1);
    ctx.colony.stock.add(Resource::Meals, meals);
    agent.carrying = None;
    StepOutcome::Done
}

fn craft(agent: &mut Agent, ctx: &mut Ctx, station: StructureId) -> StepOutcome {
    let Some(recipe_id) = ctx
        .world
        .structure(station)
        .and_then(|s| s.craft_recipe.clone())
    else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let Some(recipe) = game_data().recipe(&recipe_id) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let started = ctx
        .world
        .structure(station)
        .map(|s| s.craft_progress > 0.0)
        .unwrap_or(false);
    if !started {
        let inputs = recipe.input_bag();
        if !ctx.colony.stock.covers(&inputs) {
            return StepOutcome::Failed(FailReason::StockEmpty);
        }
        ctx.colony.stock.subtract(&inputs);
    }
    agent.inside = Some(station);
    agent.activity = Activity::Working(recipe.skill);
    let rate = work_rate(agent, ctx, recipe.skill, false);
    practise(agent, ctx, recipe.skill);
    let Some(structure) = ctx.world.structure_mut(station) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let unpowered = if structure.is_operational() {
        1.0
    } else {
        game_data().balance.work.unpowered_factor
    };
    structure.craft_progress += rate * unpowered;
    if structure.craft_progress >= recipe.work {
        structure.craft_progress = 0.0;
        ctx.colony.stock.add_bag(&recipe.output_bag());
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn research(agent: &mut Agent, ctx: &mut Ctx, lab: StructureId, step_ticks: u32) -> StepOutcome {
    let Some(structure) = ctx.world.structure(lab) else {
        return StepOutcome::Failed(FailReason::TargetGone);
    };
    let unpowered = if structure.is_operational() {
        1.0
    } else {
        game_data().balance.work.unpowered_factor
    };
    let lab_rate = structure.def().research * unpowered;
    if structure.def().indoor {
        agent.inside = Some(lab);
    }
    agent.activity = Activity::Working(Skill::Science);
    if !contribute(agent, ctx, lab_rate) {
        return StepOutcome::Done;
    }
    practise(agent, ctx, Skill::Science);
    if step_ticks > 150 {
        StepOutcome::Done
    } else {
        StepOutcome::Running
    }
}

fn study_relic(agent: &mut Agent, ctx: &mut Ctx, lab: StructureId, step_ticks: u32) -> StepOutcome {
    if ctx.colony.stock.get(Resource::Relics) < 1.0 {
        return StepOutcome::Failed(FailReason::StockEmpty);
    }
    agent.inside = Some(lab);
    agent.activity = Activity::Working(Skill::Science);
    practise(agent, ctx, Skill::Science);
    if step_ticks < 80 {
        return StepOutcome::Running;
    }
    ctx.colony.stock.take(Resource::Relics, 1.0);
    ctx.colony.research.relics_studied += 1.0;
    let points = game_data().balance.research.relic_study_points;
    add_insight(agent, ctx, Branch::Xenology, points);
    if ctx.rng.chance(0.5) {
        if let Some(tech) = super::research::reveal_any(ctx.colony, ctx.rng, true) {
            let name = game_data()
                .tech(&tech)
                .map(|t| t.name.clone())
                .unwrap_or(tech);
            let text = line(
                "relic_decoded",
                ctx.calendar.tick,
                &[("name", &agent.given_name), ("tech", &name)],
            );
            ctx.log(Category::Discovery, 2, text, vec![agent.id]);
        }
    }
    StepOutcome::Done
}

pub fn run(agent: &mut Agent, ctx: &mut Ctx, step: &Step, step_ticks: u32) -> StepOutcome {
    match step {
        Step::TakeStock { resource, amount } => take_stock(agent, ctx, *resource, *amount),
        Step::TakeMeal => take_meal(agent, ctx),
        Step::Deposit => deposit(agent, ctx),
        Step::Deliver(site) => deliver(agent, ctx, *site),
        Step::Construct(site) => construct(agent, ctx, *site),
        Step::Repair(site) => repair(agent, ctx, *site),
        Step::Harvest(node) => harvest(agent, ctx, *node),
        Step::Forage(node) => forage(agent, ctx, *node, step_ticks),
        Step::PickUp(tile) => pick_up(agent, ctx, *tile),
        Step::Tend(plot) => tend(agent, ctx, *plot),
        Step::Cook(kitchen) => cook(agent, ctx, *kitchen, step_ticks),
        Step::Craft(station) => craft(agent, ctx, *station),
        Step::Research(lab) => research(agent, ctx, *lab, step_ticks),
        Step::StudyRelic(lab) => study_relic(agent, ctx, *lab, step_ticks),
        _ => StepOutcome::Failed(FailReason::Interrupted),
    }
}
