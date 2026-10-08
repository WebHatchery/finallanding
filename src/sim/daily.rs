//! Once-a-day processes: the morning broadcast, ageing, mental breaks,
//! departures, births, ambitions, the director and the colony AI's report.

use super::chronicle_text::line;
use super::{director, environment, lifecycle, Sim};
use crate::agents::generation::{ambition_kind, mastery_skill};
use crate::agents::goals::BreakKind;
use crate::agents::Agent;
use crate::colony::chronicle::{Category, DailyReport};
use crate::data::game_data;
use crate::data::people::AmbitionKind;
use crate::world::{AgentId, Calendar};

/// The colony AI broadcasts the stockpile each morning; survivors update
/// their beliefs from it.
fn broadcast(sim: &mut Sim) {
    let stock = sim.colony.stock;
    let now = sim.calendar.tick;
    let ticks = Calendar::ticks_per_day();
    for agent in sim.agents.iter_mut().filter(|a| a.is_present()) {
        agent.beliefs.stock = stock;
        agent.beliefs.forget_stale(now, ticks);
        let decay = game_data().balance.social.opinion_decay_per_day;
        for relation in agent.beliefs.relations.values_mut() {
            relation.opinion -= relation.opinion.signum() * decay.min(relation.opinion.abs());
        }
    }
}

fn age(sim: &mut Sim) {
    let per_day = 1.0 / Calendar::days_per_year() as f32;
    let growth_days = game_data().balance.agents.child_growth_days;
    let adult = game_data().balance.agents.adult_age_years;
    let day = sim.calendar.day();
    let mut grown = Vec::new();
    for agent in sim.agents.iter_mut().filter(|a| a.is_alive()) {
        if agent.is_child {
            agent.age_years = adult * (day - agent.joined_day) as f32 / growth_days as f32;
            if day - agent.joined_day >= growth_days {
                agent.is_child = false;
                agent.age_years = adult;
                grown.push((agent.id, agent.given_name.clone()));
            }
        } else {
            agent.age_years += per_day;
        }
    }
    for (id, name) in grown {
        let text = line("came_of_age", sim.calendar.tick, &[("name", &name)]);
        sim.record(Category::Life, 2, text, vec![id]);
    }
}

fn break_kind(agent: &Agent) -> BreakKind {
    if agent.personality.temper > 0.3 {
        BreakKind::LashOut
    } else if agent.has_trait("gourmand") || agent.has_trait("lazy") {
        BreakKind::Binge
    } else if agent.personality.sociability < 0.0 {
        BreakKind::Sulk
    } else {
        BreakKind::Wander
    }
}

/// Low mood can break a survivor for a while, and lasting misery drives them away.
fn mental_health(sim: &mut Sim) {
    let mood = &game_data().balance.mood;
    let now = sim.calendar.tick;
    let ticks = Calendar::ticks_per_day();
    let mut breaks = Vec::new();
    let mut leaving = Vec::new();
    for agent in sim
        .agents
        .iter_mut()
        .filter(|a| a.is_present() && !a.is_child)
    {
        if agent.mood < mood.break_threshold && agent.mental_break.is_none() {
            let chance = mood.break_chance_per_day
                * if agent.mood < mood.extreme_threshold {
                    1.5
                } else {
                    1.0
                };
            if sim.rng.chance(chance.min(1.0)) {
                let kind = break_kind(agent);
                let until = now + mood.break_ticks as u64 * 2;
                agent.mental_break = Some((kind, until));
                agent.next_deliberation = now;
                breaks.push((agent.id, agent.given_name.clone()));
            }
        }
        if agent.mood < mood.departure_threshold {
            agent.low_mood_ticks += ticks;
        } else {
            agent.low_mood_ticks = agent.low_mood_ticks.saturating_sub(ticks);
        }
        let has_family = agent.partner.is_some();
        if agent.low_mood_ticks as f32 >= mood.departure_days * ticks as f32 && !has_family {
            leaving.push(agent.id);
        }
    }
    for (id, name) in breaks {
        let text = line("mental_break", now, &[("name", &name)]);
        sim.record(Category::Mind, 1, text, vec![id]);
    }
    for id in leaving {
        lifecycle::depart(sim, id);
    }
}

fn ambition_progress(sim: &Sim, agent: &Agent) -> f32 {
    match ambition_kind(agent) {
        Some(AmbitionKind::MasterSkill) => agent.skills.level(mastery_skill(agent)) as f32,
        Some(AmbitionKind::RaiseFamily) => sim
            .agents
            .iter()
            .filter(|c| c.parents.contains(&agent.id))
            .count() as f32,
        Some(AmbitionKind::BefriendColony) => agent
            .beliefs
            .relations
            .values()
            .filter(|r| r.opinion >= 30.0)
            .count() as f32,
        Some(AmbitionKind::MapTheWilds) => agent.stats.expeditions as f32,
        Some(AmbitionKind::DecodeSpire) => {
            sim.colony.tree.researched_in(crate::data::Branch::Xenology) as f32
        }
        Some(AmbitionKind::BuildHome) => agent
            .home
            .and_then(|h| sim.world.structure(h))
            .map(|s| s.def().comfort)
            .unwrap_or(0.0),
        Some(AmbitionKind::LeadCouncil) => {
            let mut respect = 0.0;
            for other in sim
                .agents
                .iter()
                .filter(|o| o.is_alive() && o.id != agent.id)
            {
                respect += other.beliefs.opinion_of(agent.id).max(0.0) / 10.0;
            }
            respect
        }
        None => 0.0,
    }
}

fn ambitions(sim: &mut Sim) {
    let progress: Vec<(AgentId, f32)> = sim
        .agents
        .iter()
        .filter(|a| a.is_present() && !a.ambition.fulfilled)
        .map(|a| (a.id, ambition_progress(sim, a)))
        .collect();
    let ticks = Calendar::ticks_per_day();
    let mut fulfilled = Vec::new();
    for (id, value) in progress {
        let Some(agent) = sim.agent_mut(id) else {
            continue;
        };
        agent.ambition.progress = value;
        let target = agent.ambition_def().map(|d| d.target).unwrap_or(f32::MAX);
        if value >= target {
            agent.ambition.fulfilled = true;
            agent.mind.add("ambition_fulfilled", None, ticks);
            let name = agent
                .ambition_def()
                .map(|d| d.name.clone())
                .unwrap_or_default();
            fulfilled.push((id, agent.given_name.clone(), name));
        }
    }
    for (id, who, ambition) in fulfilled {
        let text = line(
            "ambition",
            sim.calendar.tick,
            &[("name", &who), ("ambition", &ambition)],
        );
        sim.record(Category::Life, 2, text, vec![id]);
    }
}

fn track_seasons(sim: &mut Sim) {
    let calendar = sim.calendar;
    // The first day of spring means the winter before it was survived.
    if calendar.day_of_season() == 1
        && calendar.season() == crate::data::Season::Spring
        && calendar.day() > 1
    {
        let year = calendar.year() - 1;
        if sim.population() > 0 && !sim.colony.campaign.winters_survived.contains(&year) {
            sim.colony.campaign.winters_survived.push(year);
            let text = line(
                "winter_survived",
                calendar.tick,
                &[("year", &year.to_string())],
            );
            sim.record(Category::Colony, 2, text, Vec::new());
        }
    }
}

/// The colony AI's end-of-day summary: the day's most important moments.
fn write_report(sim: &mut Sim) {
    let day = sim.calendar.day().saturating_sub(1).max(1);
    let mut lines: Vec<String> = sim
        .colony
        .chronicle
        .entries_on(day)
        .filter(|e| e.importance >= 1)
        .map(|e| e.text.clone())
        .take(6)
        .collect();
    let mood = sim.average_mood();
    let mood_key = if mood >= 65.0 {
        "report_mood_high"
    } else if mood >= 40.0 {
        "report_mood_steady"
    } else {
        "report_mood_low"
    };
    lines.push(line(
        mood_key,
        day as u64,
        &[("mood", &format!("{mood:.0}"))],
    ));
    let population = sim.present().count();
    sim.colony.chronicle.reports.push(DailyReport {
        day,
        lines,
        mood,
        population,
    });
    sim.colony.campaign.low_mood_days = if mood < game_data().balance.colony.collapse_mood {
        sim.colony.campaign.low_mood_days + 1
    } else {
        0
    };
}

pub fn run(sim: &mut Sim) {
    let population = sim.population();
    sim.colony.stats.peak_population = sim.colony.stats.peak_population.max(population);
    broadcast(sim);
    write_report(sim);
    age(sim);
    mental_health(sim);
    lifecycle::consider_births(sim);
    ambitions(sim);
    track_seasons(sim);
    environment::daily(sim);
    sim.colony.expeditions.recover_sites();
    director::daily(sim);
}
