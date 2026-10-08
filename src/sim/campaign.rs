//! Campaign progression: objectives, acts, climaxes, endings and failure.

use super::chronicle_text::line;
use super::event_effects::fire_event;
use super::Sim;
use crate::colony::campaign::{CampaignState, Outcome};
use crate::colony::chronicle::Category;
use crate::data::campaign::ObjectiveKind;
use crate::data::game_data;

/// Pairs of survivors who both count the other as a friend.
pub fn friendship_count(sim: &Sim) -> u32 {
    let threshold = game_data().balance.social.friend_threshold;
    let alive: Vec<_> = sim.agents.iter().filter(|a| a.is_alive()).collect();
    let mut count = 0;
    for (index, a) in alive.iter().enumerate() {
        for b in alive.iter().skip(index + 1) {
            if a.beliefs.opinion_of(b.id) >= threshold && b.beliefs.opinion_of(a.id) >= threshold {
                count += 1;
            }
        }
    }
    count
}

fn capstone_building(sim: &Sim) -> Option<&'static str> {
    let ending = sim.colony.campaign.chosen_ending.as_ref()?;
    game_data()
        .ending(ending)
        .map(|e| e.capstone_building.as_str())
}

/// Whether an objective is met right now.
pub fn is_met(sim: &Sim, kind: &ObjectiveKind) -> bool {
    let colony = &sim.colony;
    match kind {
        ObjectiveKind::Beds { count } => {
            sim.world.built().map(|s| s.def().beds).sum::<u32>() >= *count
        }
        ObjectiveKind::Stock { resource, amount } => colony.stock.get(*resource) >= *amount,
        ObjectiveKind::TechCount { count } => colony.tree.researched_count() as u32 >= *count,
        ObjectiveKind::BranchTechs { branch, count } => {
            colony.tree.researched_in(*branch) as u32 >= *count
        }
        ObjectiveKind::Population { count } => sim.population() >= *count,
        ObjectiveKind::Structure { building, count } => {
            sim.world.count_built(building) as u32 >= *count
        }
        ObjectiveKind::Expeditions { count } => colony.expeditions.completed >= *count,
        ObjectiveKind::Explored { fraction } => sim.world.map.explored_fraction() >= *fraction,
        ObjectiveKind::Relics { count } => colony.stats.relics_found >= *count,
        ObjectiveKind::PowerSurplus { amount } => colony.power.surplus() >= *amount,
        ObjectiveKind::SurviveSeason { year, .. } => {
            colony.campaign.winters_survived.contains(year)
        }
        ObjectiveKind::AverageMood { at_least } => sim.average_mood() >= *at_least,
        ObjectiveKind::Friendships { count } => friendship_count(sim) >= *count,
        ObjectiveKind::VoteHeld => colony.campaign.vote.is_some(),
        ObjectiveKind::CapstoneTech => colony
            .campaign
            .chosen_ending
            .as_ref()
            .and_then(|e| game_data().ending(e))
            .is_some_and(|e| colony.tree.is_researched(&e.capstone_tech)),
        ObjectiveKind::CapstoneBuilt => colony.campaign.capstone_built_day.is_some(),
        ObjectiveKind::DaysAfterCapstone { days } => colony
            .campaign
            .capstone_built_day
            .is_some_and(|built| sim.calendar.day() >= built + days),
    }
}

/// Progress text for objectives with a count, for the act tracker.
pub fn progress_text(sim: &Sim, kind: &ObjectiveKind) -> Option<String> {
    let colony = &sim.colony;
    let pair = |have: f32, want: f32| Some(format!("{:.0}/{:.0}", have.min(want), want));
    match kind {
        ObjectiveKind::Beds { count } => pair(
            sim.world.built().map(|s| s.def().beds).sum::<u32>() as f32,
            *count as f32,
        ),
        ObjectiveKind::Stock { resource, amount } => pair(colony.stock.get(*resource), *amount),
        ObjectiveKind::TechCount { count } => {
            pair(colony.tree.researched_count() as f32, *count as f32)
        }
        ObjectiveKind::BranchTechs { branch, count } => {
            pair(colony.tree.researched_in(*branch) as f32, *count as f32)
        }
        ObjectiveKind::Population { count } => pair(sim.population() as f32, *count as f32),
        ObjectiveKind::Structure { building, count } => {
            pair(sim.world.count_built(building) as f32, *count as f32)
        }
        ObjectiveKind::Expeditions { count } => {
            pair(colony.expeditions.completed as f32, *count as f32)
        }
        ObjectiveKind::Explored { fraction } => Some(format!(
            "{:.0}%/{:.0}%",
            (sim.world.map.explored_fraction() * 100.0).min(fraction * 100.0),
            fraction * 100.0
        )),
        ObjectiveKind::Relics { count } => pair(colony.stats.relics_found, *count),
        ObjectiveKind::Friendships { count } => pair(friendship_count(sim) as f32, *count as f32),
        ObjectiveKind::AverageMood { at_least } => pair(sim.average_mood(), *at_least),
        ObjectiveKind::DaysAfterCapstone { days } => {
            colony.campaign.capstone_built_day.map(|built| {
                format!(
                    "{}/{}",
                    sim.calendar.day().saturating_sub(built).min(*days),
                    days
                )
            })
        }
        _ => None,
    }
}

pub fn begin_act(sim: &mut Sim, number: u8) {
    let Some(act) = game_data().act(number) else {
        return;
    };
    sim.colony.campaign.act = number;
    sim.colony.campaign.act_started_day = sim.calendar.day();
    sim.colony.campaign.climax_fired = false;
    let text = line(
        "act_begins",
        sim.calendar.tick,
        &[("number", &number.to_string()), ("act", &act.name)],
    );
    sim.record(Category::Story, 2, text, Vec::new());
    if let Some(event) = &act.opening_event {
        fire_event(sim, event);
    }
}

fn track_capstone(sim: &mut Sim) {
    let Some(building) = capstone_building(sim) else {
        return;
    };
    let built = sim.world.count_built(building) > 0;
    if built && sim.colony.campaign.capstone_built_day.is_none() {
        sim.colony.campaign.capstone_built_day = Some(sim.calendar.day());
    }
    let progress = sim
        .world
        .structures
        .iter()
        .find(|s| s.kind == building)
        .map(|s| s.construction_fraction())
        .unwrap_or(0.0);
    if (progress >= 0.5 || built) && !sim.colony.campaign.final_crisis_fired {
        sim.colony.campaign.final_crisis_fired = true;
        let crisis = sim
            .colony
            .campaign
            .chosen_ending
            .as_ref()
            .and_then(|e| game_data().ending(e))
            .map(|e| e.final_crisis_event.clone());
        if let Some(crisis) = crisis {
            fire_event(sim, &crisis);
        }
    }
}

fn check_failure(sim: &mut Sim) -> bool {
    let day = sim.calendar.day();
    let reason = if sim.population() == 0 {
        Some("failure_extinct")
    } else if sim.colony.campaign.low_mood_days >= game_data().balance.colony.collapse_days {
        Some("failure_collapse")
    } else {
        None
    };
    if let Some(reason) = reason {
        sim.colony.campaign.outcome = Some(Outcome::Failure {
            reason: reason.to_owned(),
            day,
        });
        let text = game_data().label(reason).to_owned();
        sim.record(Category::Loss, 2, text, Vec::new());
        return true;
    }
    false
}

fn finish(sim: &mut Sim) {
    let day = sim.calendar.day();
    let ending = sim
        .colony
        .campaign
        .chosen_ending
        .clone()
        .unwrap_or_default();
    let name = game_data()
        .ending(&ending)
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let text = line("victory", sim.calendar.tick, &[("ending", &name)]);
    sim.record(Category::Story, 2, text, Vec::new());
    sim.colony.campaign.outcome = Some(Outcome::Victory { ending, day });
}

pub fn evaluate(sim: &mut Sim) {
    if sim.colony.campaign.is_over() || check_failure(sim) {
        return;
    }
    track_capstone(sim);
    let number = sim.colony.campaign.act;
    let Some(act) = game_data().act(number) else {
        return;
    };
    let mut newly = Vec::new();
    for objective in &act.objectives {
        if !sim.colony.campaign.is_complete(number, &objective.id) && is_met(sim, &objective.kind) {
            newly.push((
                CampaignState::objective_key(number, &objective.id),
                objective.text.clone(),
            ));
        }
    }
    for (key, text) in newly {
        sim.colony.campaign.completed.push(key);
        let entry = line(
            "objective_complete",
            sim.calendar.tick,
            &[("objective", &text)],
        );
        sim.record(Category::Colony, 1, entry, Vec::new());
    }
    let done = act
        .objectives
        .iter()
        .filter(|o| sim.colony.campaign.is_complete(number, &o.id))
        .count();
    if done * 2 >= act.objectives.len() && !sim.colony.campaign.climax_fired {
        sim.colony.campaign.climax_fired = true;
        if let Some(event) = &act.climax_event {
            fire_event(sim, event);
        }
    }
    if done == act.objectives.len() {
        if game_data().act(number + 1).is_some() {
            begin_act(sim, number + 1);
        } else {
            finish(sim);
        }
    }
}
