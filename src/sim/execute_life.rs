//! Life steps: eating, sleeping, warming up, relaxing, recovering, hiding,
//! defending, scouting and mental breaks.

use super::execute_work::practise;
use super::expeditions::volunteer;
use super::Ctx;
use crate::agents::plans::{FailReason, FoodSource, Step, StepOutcome};
use crate::agents::{Activity, Agent};
use crate::data::{game_data, Need, Resource, Skill};
use crate::world::{Calendar, StructureId, Tile};

const EAT_TICKS: u32 = 10;

fn enter_if_indoor(agent: &mut Agent, ctx: &Ctx, place: Option<StructureId>) {
    let structure = place.and_then(|id| ctx.world.structure(id));
    agent.inside = structure
        .filter(|s| s.def().indoor && s.footprint.distance_to(agent.tile) <= 1.6)
        .map(|s| s.id);
}

fn nearby_indoor(agent: &Agent, ctx: &Ctx) -> Option<StructureId> {
    ctx.world
        .built()
        .find(|s| s.def().indoor && s.footprint.distance_to(agent.tile) <= 1.6)
        .map(|s| s.id)
}

fn eat(agent: &mut Agent, ctx: &mut Ctx, source: FoodSource, step_ticks: u32) -> StepOutcome {
    let carried = match agent.carrying {
        Some((Resource::Meals, amount)) => Some((Resource::Meals, amount)),
        Some((Resource::Food, amount)) => Some((Resource::Food, amount)),
        _ => None,
    };
    let Some((resource, amount)) = carried else {
        return StepOutcome::Failed(FailReason::StockEmpty);
    };
    agent.activity = Activity::Eating;
    if agent.inside.is_none() {
        agent.inside = ctx
            .world
            .built()
            .find(|s| s.def().kitchen && s.footprint.distance_to(agent.tile) <= 1.6)
            .map(|s| s.id);
    }
    if step_ticks < EAT_TICKS {
        return StepOutcome::Running;
    }
    let needs = &game_data().balance.needs;
    let portion = ctx.colony.food_use().sqrt();
    let (restore, thought) = match (resource, source) {
        (Resource::Meals, _) => (needs.meal_restore, "ate_meal"),
        (_, FoodSource::Carried) => (needs.raw_food_restore, "ate_raw"),
        _ => (needs.raw_food_restore, "ate_raw"),
    };
    let eaten = amount.min(1.0);
    agent
        .needs
        .change(Need::Food, restore * portion * eaten.max(0.5));
    agent.mind.add(thought, None, Calendar::ticks_per_day());
    agent.mind.remove("went_hungry");
    let left = amount - eaten;
    agent.carrying = if left > 0.05 {
        Some((resource, left))
    } else {
        None
    };
    StepOutcome::Done
}

fn claim_bed(agent: &mut Agent, ctx: &mut Ctx, bed: StructureId) -> bool {
    let Some(structure) = ctx.world.structure_mut(bed) else {
        return false;
    };
    if structure.residents.contains(&agent.id) {
        return true;
    }
    let has_family = agent.partner.is_some() || !agent.parents.is_empty();
    if structure.free_beds() == 0 || (structure.def().family_only && !has_family) {
        return false;
    }
    if let Some(old) = agent.home {
        let previous_comfort = ctx
            .world
            .structure(old)
            .map(|s| s.def().comfort)
            .unwrap_or(-1.0);
        let new_comfort = ctx
            .world
            .structure(bed)
            .map(|s| s.def().comfort)
            .unwrap_or(0.0);
        let joins_partner = agent.partner.is_some_and(|p| {
            ctx.world
                .structure(bed)
                .is_some_and(|s| s.residents.contains(&p))
        });
        let family_move = agent.partner.is_some()
            && ctx
                .world
                .structure(bed)
                .is_some_and(|s| s.def().family_only);
        if previous_comfort >= new_comfort && !joins_partner && !family_move {
            return true;
        }
        if let Some(previous) = ctx.world.structure_mut(old) {
            previous.residents.retain(|r| *r != agent.id);
        }
    }
    if let Some(structure) = ctx.world.structure_mut(bed) {
        structure.residents.push(agent.id);
    }
    agent.home = Some(bed);
    true
}

fn sleep(
    agent: &mut Agent,
    ctx: &mut Ctx,
    place: Option<StructureId>,
    step_ticks: u32,
) -> StepOutcome {
    let needs = &game_data().balance.needs;
    let mut comfort = None;
    if let Some(bed) = place {
        if step_ticks == 0 && !claim_bed(agent, ctx, bed) {
            return StepOutcome::Failed(FailReason::Occupied);
        }
        comfort = ctx.world.structure(bed).map(|s| s.def().comfort);
        enter_if_indoor(agent, ctx, Some(bed));
    }
    agent.activity = Activity::Sleeping;
    let rate = match comfort {
        Some(c) => needs.bed_rest_per_day * (1.0 + c * 0.3),
        None => needs.ground_rest_per_day,
    };
    agent.needs.change(Need::Rest, Calendar::per_tick(rate));
    let sleep_time = {
        let time = &game_data().balance.time;
        let owl = if agent.personality.night_owl {
            3.0
        } else {
            0.0
        };
        let hour = (ctx.calendar.hour() - owl).rem_euclid(24.0);
        hour >= time.sleep_hour || hour < time.dawn_hour
    };
    let rested = agent.needs.get(Need::Rest) >= 99.0;
    let too_long = step_ticks as u64 > Calendar::ticks_per_day() * 6 / 10;
    if (rested && !sleep_time) || too_long || (rested && step_ticks > 200) {
        let thought = if comfort.is_some() {
            "slept_bed"
        } else {
            "slept_ground"
        };
        agent.mind.add(thought, None, Calendar::ticks_per_day());
        agent.inside = None;
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn warm(agent: &mut Agent, ctx: &mut Ctx, ticks: u32, step_ticks: u32) -> StepOutcome {
    if agent.inside.is_none() {
        agent.inside = nearby_indoor(agent, ctx);
    }
    agent.activity = Activity::Relaxing;
    let sheltered = agent.inside.is_some() || ctx.world.heat_at(agent.tile) > 0.0;
    if !sheltered {
        // Body heat and movement: slow but better than standing still.
        agent.needs.change(Need::Warmth, Calendar::per_tick(45.0));
    }
    let warm_enough = agent.needs.get(Need::Warmth) >= 85.0;
    if warm_enough || step_ticks >= ticks * 3 {
        agent.inside = None;
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn relax(
    agent: &mut Agent,
    ctx: &mut Ctx,
    place: Option<StructureId>,
    ticks: u32,
    step_ticks: u32,
) -> StepOutcome {
    enter_if_indoor(agent, ctx, place);
    agent.activity = Activity::Relaxing;
    let (quality, beauty) = place
        .and_then(|id| ctx.world.structure(id))
        .map(|s| (s.def().recreation, s.def().beauty))
        .unwrap_or((0.8, 0.0));
    let gain = game_data().balance.needs.recreation_gain_per_day * quality.max(0.3);
    agent
        .needs
        .change(Need::Recreation, Calendar::per_tick(gain));
    if beauty >= 0.5 && step_ticks == 10 {
        agent
            .mind
            .add("beautiful_place", None, Calendar::ticks_per_day());
    }
    if step_ticks >= ticks || agent.needs.get(Need::Recreation) >= 96.0 {
        agent.inside = None;
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn recover(
    agent: &mut Agent,
    ctx: &mut Ctx,
    place: Option<StructureId>,
    ticks: u32,
    step_ticks: u32,
) -> StepOutcome {
    enter_if_indoor(agent, ctx, place);
    agent.activity = Activity::Recovering;
    let threshold = game_data().balance.health.care_threshold;
    let healed = agent.health.hp >= threshold + 10.0 && !agent.health.ill;
    if healed || step_ticks >= ticks {
        if place
            .and_then(|id| ctx.world.structure(id))
            .is_some_and(|s| s.def().care_beds > 0)
        {
            agent.mind.add("cared_for", None, Calendar::ticks_per_day());
        }
        agent.inside = None;
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn hide(agent: &mut Agent, ctx: &mut Ctx, ticks: u32, step_ticks: u32) -> StepOutcome {
    if agent.inside.is_none() {
        agent.inside = nearby_indoor(agent, ctx);
    }
    agent.activity = Activity::Fleeing;
    let threat_near = agent
        .beliefs
        .nearest_threat(agent.tile)
        .is_some_and(|(_, distance)| distance < 6.0);
    if step_ticks >= ticks && (!threat_near || step_ticks >= ticks * 3) {
        agent.inside = None;
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn wait_for_help(agent: &mut Agent, ticks: u32, step_ticks: u32) -> StepOutcome {
    agent.activity = Activity::Waiting;
    if agent.carried(Resource::Food) >= 0.5
        || agent.carried(Resource::Meals) >= 0.5
        || agent.health.treated
    {
        return StepOutcome::Done;
    }
    if step_ticks >= ticks {
        agent
            .mind
            .add("refused_help", None, Calendar::ticks_per_day());
        return StepOutcome::Failed(FailReason::TimedOut);
    }
    StepOutcome::Running
}

fn mental_break(agent: &mut Agent, ticks: u32, step_ticks: u32) -> StepOutcome {
    agent.activity = Activity::Breaking;
    if let Some((Resource::Food, amount)) = agent.carrying {
        agent.needs.change(Need::Food, amount * 10.0);
        agent.carrying = None;
    }
    if step_ticks >= ticks {
        agent.mental_break = None;
        agent
            .mind
            .add("mental_break", None, Calendar::ticks_per_day());
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn scout(agent: &mut Agent, ctx: &mut Ctx, tile: Tile, step_ticks: u32) -> StepOutcome {
    agent.activity = Activity::Working(Skill::Exploration);
    practise(agent, ctx, Skill::Exploration);
    let radius = game_data().balance.agents.sight_radius * 1.3;
    ctx.world.map.reveal_circle(tile, radius);
    if step_ticks >= 10 {
        return StepOutcome::Done;
    }
    StepOutcome::Running
}

fn attack(agent: &mut Agent, ctx: &mut Ctx, creature: u32, step_ticks: u32) -> StepOutcome {
    let Some(target) = ctx
        .world
        .creatures
        .iter()
        .find(|c| c.id == creature)
        .map(|c| c.tile)
    else {
        return StepOutcome::Done;
    };
    if agent.health.hp < 40.0 {
        return StepOutcome::Failed(FailReason::TooDangerous);
    }
    if step_ticks > 300 {
        return StepOutcome::Failed(FailReason::TimedOut);
    }
    agent.inside = None;
    if target.steps(agent.tile) > 1 {
        if agent.path.is_empty() || step_ticks.is_multiple_of(6) {
            let destination = ctx.world.map.nearest_walkable(target, 1).unwrap_or(target);
            match ctx.world.find_path(agent.tile, destination) {
                Some(path) => agent.path = path,
                None => return StepOutcome::Failed(FailReason::NoPath),
            }
        }
        agent.activity = Activity::Walking;
        return StepOutcome::Running;
    }
    agent.path.clear();
    agent.activity = Activity::Fighting;
    let damage = 1.2 * (1.0 + agent.personality.bravery.max(0.0)) * agent.health.capacity();
    ctx.effects.attacks.push((agent.id, creature, damage));
    practise(agent, ctx, Skill::Exploration);
    StepOutcome::Running
}

pub fn run(agent: &mut Agent, ctx: &mut Ctx, step: &Step, step_ticks: u32) -> StepOutcome {
    match step {
        Step::Eat(source) => eat(agent, ctx, *source, step_ticks),
        Step::Sleep(place) => sleep(agent, ctx, *place, step_ticks),
        Step::Warm { ticks } => warm(agent, ctx, *ticks, step_ticks),
        Step::Relax { place, ticks } => relax(agent, ctx, *place, *ticks, step_ticks),
        Step::Recover { place, ticks } => recover(agent, ctx, *place, *ticks, step_ticks),
        Step::Hide { ticks } => hide(agent, ctx, *ticks, step_ticks),
        Step::WaitForHelp { ticks } => wait_for_help(agent, *ticks, step_ticks),
        Step::Break { ticks, .. } => mental_break(agent, *ticks, step_ticks),
        Step::Scout(tile) => scout(agent, ctx, *tile, step_ticks),
        Step::JoinExpedition => {
            volunteer(agent, ctx);
            StepOutcome::Done
        }
        Step::Attack(creature) => attack(agent, ctx, *creature, step_ticks),
        Step::Wait { ticks } => {
            agent.activity = Activity::Idle;
            if step_ticks >= *ticks {
                StepOutcome::Done
            } else {
                StepOutcome::Running
            }
        }
        _ => StepOutcome::Failed(FailReason::Interrupted),
    }
}
