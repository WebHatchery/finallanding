//! Research progress, project choice, insight and eureka discoveries.

use super::chronicle_text::line;
use super::Ctx;
use crate::agents::Agent;
use crate::colony::chronicle::Category;
use crate::colony::tech_tree::Modifiers;
use crate::colony::Colony;
use crate::data::{game_data, Branch, Skill};
use crate::world::Calendar;

/// The project researchers advance: the player's focus if it is available,
/// otherwise the researcher's own interest.
fn choose_project(colony: &Colony, agent: &Agent) -> Option<String> {
    let available = colony.tree.available();
    if let Some(focus) = &colony.research.focus {
        if available.contains(&focus.as_str()) {
            return Some(focus.clone());
        }
    }
    if let Some(active) = &colony.research.active {
        if available.contains(&active.as_str()) {
            return Some(active.clone());
        }
    }
    let scale = colony.difficulty_value(|d| d.research_cost);
    let interest = |id: &str| -> f32 {
        let Some(def) = game_data().tech(id) else {
            return f32::MAX;
        };
        let cost = colony.tree.cost(id, scale);
        let passion = agent
            .skills
            .passions
            .iter()
            .any(|skill| skill.branch() == def.branch);
        let curious = def.branch == Branch::Xenology && agent.personality.curiosity > 0.2;
        let started = colony.research.progress_of(id) > 0.0;
        cost * if passion || curious { 0.6 } else { 1.0 } * if started { 0.5 } else { 1.0 }
    };
    available
        .into_iter()
        .min_by(|a, b| interest(a).total_cmp(&interest(b)))
        .map(str::to_owned)
}

/// Add one tick of research from a survivor at a lab. Returns false when there
/// is nothing left to research.
pub fn contribute(agent: &mut Agent, ctx: &mut Ctx, lab_rate: f32) -> bool {
    let Some(project) = choose_project(ctx.colony, agent) else {
        return false;
    };
    ctx.colony.research.active = Some(project.clone());
    let balance = &game_data().balance.research;
    let speed = agent.skills.speed(Skill::Science, &agent.personality)
        * ctx.colony.modifiers.work_speed[Skill::Science.index()]
        * ctx.colony.modifiers.research_speed
        * agent.health.capacity();
    let points = balance.points_per_tick * lab_rate * speed;
    let total = {
        let progress = ctx
            .colony
            .research
            .progress
            .entry(project.clone())
            .or_insert(0.0);
        *progress += points;
        *progress
    };
    let scale = ctx.colony.difficulty_value(|d| d.research_cost);
    if total >= ctx.colony.tree.cost(&project, scale) {
        complete(ctx, &project, Some(agent.given_name.clone()));
    }
    true
}

pub fn complete(ctx: &mut Ctx, id: &str, credit: Option<String>) {
    let day = ctx.calendar.day();
    let locked = ctx.colony.tree.complete(id, day);
    ctx.colony.modifiers = Modifiers::from_tree(&ctx.colony.tree);
    ctx.colony.research.active = None;
    if ctx.colony.research.focus.as_deref() == Some(id) {
        ctx.colony.research.focus = None;
    }
    let name = game_data().tech(id).map(|t| t.name.as_str()).unwrap_or(id);
    let who = credit.unwrap_or_default();
    let text = line(
        "tech_researched",
        ctx.calendar.tick,
        &[("tech", name), ("name", &who)],
    );
    ctx.log(Category::Discovery, 2, text, Vec::new());
    for other in locked {
        let other_name = game_data()
            .tech(&other)
            .map(|t| t.name.clone())
            .unwrap_or(other);
        let text = line(
            "tech_locked",
            ctx.calendar.tick,
            &[("tech", &other_name), ("chosen", name)],
        );
        ctx.log(Category::Discovery, 1, text, Vec::new());
    }
}

/// Practice in a branch builds insight; enough insight becomes a eureka that
/// reveals a hidden technology, credited to the survivor who had it.
pub fn add_insight(agent: &mut Agent, ctx: &mut Ctx, branch: Branch, amount: f32) {
    let threshold = game_data().balance.research.eureka_threshold;
    let curiosity = 1.0 + agent.personality.curiosity.max(0.0);
    ctx.colony.research.add_insight(branch, amount * curiosity);
    if ctx.colony.research.insight[branch.index()] < threshold {
        return;
    }
    let candidates = ctx.colony.tree.discoverable(Some(branch), false);
    if candidates.is_empty() {
        ctx.colony.research.insight[branch.index()] = threshold;
        return;
    }
    ctx.colony.research.insight[branch.index()] -= threshold;
    let pick = candidates[ctx.rng.below(candidates.len())].clone();
    ctx.colony.tree.reveal(&pick);
    ctx.colony.stats.eurekas += 1;
    agent.stats.eurekas += 1;
    agent.mind.add("eureka", None, Calendar::ticks_per_day());
    let name = game_data()
        .tech(&pick)
        .map(|t| t.name.as_str())
        .unwrap_or(&pick);
    let text = line(
        "eureka",
        ctx.calendar.tick,
        &[("name", &agent.given_name), ("tech", name)],
    );
    ctx.log(Category::Discovery, 2, text, vec![agent.id]);
}

/// Reveal a hidden technology from an event, expedition or relic.
pub fn reveal_any(
    ctx_colony: &mut Colony,
    rng: &mut macroquad_toolkit::rng::SeededRng,
    allow_relic: bool,
) -> Option<String> {
    let candidates = ctx_colony.tree.discoverable(None, allow_relic);
    if candidates.is_empty() {
        return None;
    }
    let pick = candidates[rng.below(candidates.len())].clone();
    ctx_colony.tree.reveal(&pick);
    Some(pick)
}
