//! Expeditions: calls for volunteers, departure, and what parties bring home.

use super::chronicle_text::line;
use super::research::{inspire, reveal_any};
use super::{Ctx, Sim};
use crate::agents::{Agent, LifeState};
use crate::colony::chronicle::Category;
use crate::colony::expeditions::{ActiveExpedition, ExpeditionCall, ExpeditionReport};
use crate::data::events::ExpeditionSiteDef;
use crate::data::{game_data, Resource, Skill};
use crate::world::{AgentId, Calendar};

/// Sites the colony knows of and can reach in the current act.
pub fn available_sites(sim: &Sim) -> Vec<&'static ExpeditionSiteDef> {
    let act = sim.colony.campaign.act;
    let expeditions = &sim.colony.expeditions;
    game_data()
        .expedition_sites
        .iter()
        .filter(|site| act >= site.min_act || expeditions.revealed_sites.contains(&site.id))
        .filter(|site| site.repeatable || !expeditions.visited_sites.contains(&site.id))
        .collect()
}

pub fn open_call(sim: &mut Sim, site: &str) -> bool {
    let has_gate = sim.world.built().any(|s| s.def().expedition);
    let known = available_sites(sim).iter().any(|s| s.id == site);
    if !has_gate || !known || sim.colony.expeditions.call.is_some() {
        return false;
    }
    sim.colony.expeditions.call = Some(ExpeditionCall {
        site: site.to_owned(),
        opened_tick: sim.calendar.tick,
        volunteers: Vec::new(),
    });
    true
}

pub fn cancel_call(sim: &mut Sim) {
    if let Some(call) = sim.colony.expeditions.call.take() {
        for id in call.volunteers {
            if let Some(agent) = sim.agent_mut(id) {
                agent.volunteered = false;
            }
        }
    }
}

/// A survivor signs up at the gate (the Volunteer goal's final step).
pub fn volunteer(agent: &mut Agent, ctx: &mut Ctx) {
    let max = game_data().balance.expeditions.max_party;
    if let Some(call) = ctx.colony.expeditions.call.as_mut() {
        if call.volunteers.len() < max && !call.volunteers.contains(&agent.id) {
            call.volunteers.push(agent.id);
            agent.volunteered = true;
        }
    }
}

fn depart(sim: &mut Sim, call: ExpeditionCall) {
    let Some(site) = game_data().expedition_site(&call.site) else {
        return;
    };
    let id = sim.colony.expeditions.next_id;
    sim.colony.expeditions.next_id += 1;
    let safety = sim.colony.modifiers.expedition_safety;
    let days = site.days * (1.0 - safety * 0.3).max(0.5);
    for member in &call.volunteers {
        if let Some(agent) = sim.agent_mut(*member) {
            agent.life = LifeState::Away { expedition: id };
            agent.intention = None;
            agent.path.clear();
            agent.inside = None;
            agent.volunteered = false;
            if let Some((resource, amount)) = agent.carrying.take() {
                sim.colony.stock.add(resource, amount);
            }
        }
    }
    let names = names_of(sim, &call.volunteers);
    let text = line(
        "expedition_departs",
        sim.calendar.tick,
        &[("names", &names), ("site", &site.name)],
    );
    sim.record(Category::Story, 2, text, call.volunteers.clone());
    sim.colony.expeditions.active.push(ActiveExpedition {
        id,
        site: call.site,
        members: call.volunteers,
        depart_tick: sim.calendar.tick,
        return_tick: sim.calendar.tick + Calendar::ticks_for_days(days),
    });
}

fn names_of(sim: &Sim, ids: &[AgentId]) -> String {
    ids.iter()
        .filter_map(|id| sim.agent(*id))
        .map(|a| a.given_name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

fn party_skill(sim: &Sim, members: &[AgentId]) -> f32 {
    members
        .iter()
        .filter_map(|id| sim.agent(*id))
        .map(|a| a.skills.level(Skill::Exploration) as f32)
        .sum::<f32>()
        / members.len().max(1) as f32
}

fn resolve_casualties(sim: &mut Sim, site: &ExpeditionSiteDef, members: &[AgentId]) -> Vec<String> {
    let balance = &game_data().balance.expeditions;
    let safety = sim.colony.modifiers.expedition_safety;
    let ticks = Calendar::ticks_per_day();
    let now = sim.calendar.tick;
    let mut notes = Vec::new();
    for id in members {
        let danger = site.danger * (1.0 - safety).max(0.2);
        let dies = sim.rng.chance(danger * balance.death_chance_per_danger);
        let injured = sim.rng.chance(danger * balance.injury_chance_per_danger);
        let Some(agent) = sim.agent_mut(*id) else {
            continue;
        };
        agent.life = LifeState::Present;
        agent.stats.expeditions += 1;
        agent.mind.add("expedition_home", None, ticks);
        if dies {
            agent.health.hp = 0.0;
            notes.push(line("expedition_lost", now, &[("name", &agent.given_name)]));
        } else if injured {
            agent.health.damage(35.0);
            agent.health.injuries += 1;
            agent.mind.add("injured", None, ticks);
            notes.push(line(
                "expedition_injured",
                now,
                &[("name", &agent.given_name)],
            ));
        }
    }
    notes
}

fn return_home(sim: &mut Sim, expedition: ActiveExpedition) {
    let Some(site) = game_data().expedition_site(&expedition.site) else {
        return;
    };
    let balance = &game_data().balance.expeditions;
    let tile = sim
        .world
        .built()
        .find(|s| s.def().expedition)
        .and_then(|s| sim.world.access_tile(s.id, s.footprint.origin))
        .unwrap_or(sim.world.landing_tile);
    for id in &expedition.members {
        if let Some(agent) = sim.agent_mut(*id) {
            agent.tile = tile;
            agent.position = crate::world::Point::of_tile(tile);
        }
    }
    let skill = (1.0 + party_skill(sim, &expedition.members) * 0.04)
        * sim.colony.expeditions.reward_factor(&site.id);
    let mut haul = Vec::new();
    for (resource, amount) in &site.rewards {
        let gained = amount * balance.reward_scale * skill * sim.rng.range_f32(0.7, 1.3);
        sim.colony.stock.add(*resource, gained);
        haul.push(format!(
            "{:.0} {}",
            gained,
            game_data().label(resource.key())
        ));
    }
    if site.relics > 0.0 {
        let relics = (site.relics
            * sim.colony.expeditions.reward_factor(&site.id)
            * sim.rng.range_f32(0.5, 1.5))
        .round();
        sim.colony.stock.add(Resource::Relics, relics);
        sim.colony.stats.relics_found += relics;
        if relics > 0.0 {
            haul.push(format!("{relics:.0} {}", game_data().label("relics")));
        }
    }
    let mut notes = resolve_casualties(sim, site, &expedition.members);
    let room_for_samples = sim.colony.stats.samples_brought_home < balance.max_samples;
    if room_for_samples && sim.rng.chance(balance.sample_chance) {
        notes.extend(bring_home_sample(sim, &site.name));
    }
    if site.reveals_tech && sim.rng.chance(balance.tech_chance) {
        if let Some(tech) = reveal_any(&mut sim.colony, &mut sim.rng, true) {
            let name = game_data()
                .tech(&tech)
                .map(|t| t.name.clone())
                .unwrap_or(tech);
            notes.push(line(
                "expedition_tech",
                sim.calendar.tick,
                &[("tech", &name)],
            ));
        }
    }
    let recruit_chance =
        balance.recruit_chance.max(0.5) * sim.colony.expeditions.reward_factor(&site.id);
    if site.recruits && sim.rng.chance(recruit_chance) {
        let id = sim.spawn_survivor(tile);
        sim.colony.stats.arrivals += 1;
        let name = sim
            .agent(id)
            .map(|a| a.given_name.clone())
            .unwrap_or_default();
        notes.push(line(
            "expedition_recruit",
            sim.calendar.tick,
            &[("name", &name)],
        ));
    }
    *sim.colony
        .expeditions
        .visit_counts
        .entry(site.id.clone())
        .or_insert(0.0) += 1.0;
    if !sim.colony.expeditions.visited_sites.contains(&site.id) {
        sim.colony.expeditions.visited_sites.push(site.id.clone());
    }
    sim.colony.expeditions.completed += 1;
    let names = names_of(sim, &expedition.members);
    let mut summary = line(
        "expedition_returns",
        sim.calendar.tick,
        &[
            ("names", &names),
            ("site", &site.name),
            ("haul", &haul.join(", ")),
        ],
    );
    for note in notes {
        summary.push(' ');
        summary.push_str(&note);
    }
    sim.colony.expeditions.reports.push(ExpeditionReport {
        site: site.name.clone(),
        day: sim.calendar.day(),
        summary: summary.clone(),
    });
    sim.record(Category::Story, 2, summary, expedition.members.clone());
}

pub fn update(sim: &mut Sim) {
    let balance = &game_data().balance.expeditions;
    if let Some(call) = sim.colony.expeditions.call.clone() {
        let elapsed = sim.calendar.tick.saturating_sub(call.opened_tick);
        let full = call.volunteers.len() >= balance.max_party;
        let window_over = elapsed >= balance.volunteer_window_ticks as u64;
        if full || (window_over && call.volunteers.len() >= balance.min_party) {
            sim.colony.expeditions.call = None;
            sim.colony.expeditions.unanswered_calls = 0;
            depart(sim, call);
        } else if window_over {
            sim.colony.expeditions.unanswered_calls += 1;
            cancel_call(sim);
            let site = game_data()
                .expedition_site(&call.site)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let text = line(
                "expedition_no_volunteers",
                sim.calendar.tick,
                &[("site", &site)],
            );
            sim.record(Category::Colony, 1, text, Vec::new());
        }
    }
    let now = sim.calendar.tick;
    let (returning, remaining): (Vec<_>, Vec<_>) =
        std::mem::take(&mut sim.colony.expeditions.active)
            .into_iter()
            .partition(|e| e.return_tick <= now);
    sim.colony.expeditions.active = remaining;
    for expedition in returning {
        return_home(sim, expedition);
    }
}

/// A species that does not grow near the colony, carried home by a party. It
/// joins the landing's species, so the technologies it inspires can appear.
fn bring_home_sample(sim: &mut Sim, site_name: &str) -> Vec<String> {
    let candidates: Vec<String> = game_data()
        .finds
        .iter()
        .filter(|f| !sim.world.species.contains(&f.id))
        .map(|f| f.id.clone())
        .collect();
    let Some(id) = sim.rng.choose(&candidates).cloned() else {
        return Vec::new();
    };
    let Some(find) = game_data().find(&id) else {
        return Vec::new();
    };
    let amount = game_data().balance.expeditions.sample_amount;
    let day = sim.calendar.day();
    sim.world.species.push(id.clone());
    sim.colony.stats.samples_brought_home += 1;
    sim.colony.tree.introduce(&id);
    sim.colony.finds.record(&id, amount, day, site_name);
    let tick = sim.calendar.tick;
    let mut notes = vec![line("expedition_sample", tick, &[("find", &find.name)])];
    for tech in inspire(&mut sim.colony, &id) {
        let name = game_data().tech(&tech).map_or(tech.as_str(), |t| &t.name);
        notes.push(line(
            "tech_inspired",
            tick,
            &[("find", &find.name), ("tech", name)],
        ));
    }
    notes
}
