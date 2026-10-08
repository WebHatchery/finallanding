//! Deaths, departures and births: the colony's changing membership.

use super::chronicle_text::line;
use super::Sim;
use crate::agents::generation::{create_child, Arrival};
use crate::agents::LifeState;
use crate::colony::chronicle::Category;
use crate::data::game_data;
use crate::world::{AgentId, Calendar};

/// Remove someone from the colony's homes and partnerships, and let everyone
/// who knew them feel the loss.
fn mourn(sim: &mut Sim, id: AgentId, thought_for_friends: &str) {
    for structure in sim.world.structures.iter_mut() {
        structure.residents.retain(|r| *r != id);
    }
    let ticks = Calendar::ticks_per_day();
    for other in sim.agents.iter_mut().filter(|a| a.is_alive() && a.id != id) {
        if other.partner == Some(id) {
            other.partner = None;
            other.mind.add("partner_died", Some(id), ticks);
        } else if other.beliefs.opinion_of(id) > 30.0 {
            other.mind.add(thought_for_friends, Some(id), ticks);
        } else if thought_for_friends == "friend_died" {
            other.mind.add("death_witness", Some(id), ticks);
        }
        if other
            .help_task
            .is_some_and(|(requester, _)| requester == id)
        {
            other.help_task = None;
        }
    }
}

pub fn resolve_deaths(sim: &mut Sim) {
    let dead: Vec<(AgentId, String)> = sim
        .agents
        .iter()
        .filter(|a| a.is_alive() && a.health.is_dead())
        .map(|a| (a.id, a.given_name.clone()))
        .collect();
    for (id, name) in dead {
        let day = sim.calendar.day();
        let cause = cause_of_death(sim, id);
        if let Some(agent) = sim.agent_mut(id) {
            agent.life = LifeState::Dead {
                day,
                cause: cause.clone(),
            };
            agent.intention = None;
            agent.path.clear();
            agent.inside = None;
        }
        sim.colony.stats.deaths += 1;
        let label = game_data().label(&cause).to_owned();
        let text = line(
            "death",
            sim.calendar.tick,
            &[("name", &name), ("cause", &label)],
        );
        sim.record(Category::Loss, 2, text, vec![id]);
        mourn(sim, id, "friend_died");
    }
}

fn cause_of_death(sim: &Sim, id: AgentId) -> String {
    let Some(agent) = sim.agent(id) else {
        return "cause_unknown".into();
    };
    if agent.needs.get(crate::data::Need::Food) <= 1.0 {
        "cause_starvation".into()
    } else if agent.needs.get(crate::data::Need::Warmth) <= 1.0 {
        "cause_cold".into()
    } else if agent.health.ill {
        "cause_illness".into()
    } else if agent.age_years > game_data().balance.health.old_age_years {
        "cause_age".into()
    } else {
        "cause_wounds".into()
    }
}

/// A survivor who has been miserable too long leaves the colony, unless
/// family keeps them.
pub fn depart(sim: &mut Sim, id: AgentId) {
    let day = sim.calendar.day();
    let Some(agent) = sim.agent_mut(id) else {
        return;
    };
    agent.life = LifeState::Departed { day };
    agent.intention = None;
    agent.path.clear();
    let name = agent.given_name.clone();
    sim.colony.stats.departures += 1;
    let text = line("departure", sim.calendar.tick, &[("name", &name)]);
    sim.record(Category::Loss, 2, text, vec![id]);
    mourn(sim, id, "frustrated");
}

/// Partners with a home and a spare bed may have a child.
pub fn consider_births(sim: &mut Sim) {
    let social = &game_data().balance.social;
    let chance = social.child_chance_per_day * sim.colony.child_chance_scale();
    let max_population = game_data().balance.colony.max_population;
    let couples: Vec<(AgentId, AgentId)> = sim
        .present()
        .filter(|a| !a.is_child && a.age_years < 48.0)
        .filter_map(|a| a.partner.filter(|p| *p > a.id).map(|p| (a.id, p)))
        .collect();
    for (a, b) in couples {
        let shared_home = sim
            .agent(a)
            .and_then(|x| x.home)
            .filter(|h| sim.agent(b).and_then(|y| y.home) == Some(*h));
        let has_room = shared_home
            .and_then(|h| sim.world.structure(h))
            .is_some_and(|s| s.free_beds() > 0);
        let both_home = sim.agent(b).is_some_and(|x| x.is_present());
        if !has_room || !both_home || sim.population() >= max_population || !sim.rng.chance(chance)
        {
            continue;
        }
        let (Some(pa), Some(pb)) = (sim.agent(a).cloned(), sim.agent(b).cloned()) else {
            continue;
        };
        let id = sim.next_agent_id;
        sim.next_agent_id += 1;
        let taken: Vec<String> = sim.agents.iter().map(|x| x.given_name.clone()).collect();
        let arrival = Arrival {
            id,
            tile: pa.tile,
            day: sim.calendar.day(),
            taken_names: &taken,
        };
        let mut child = create_child(&arrival, (&pa, &pb), &mut sim.rng);
        child.home = pa.home;
        child.beliefs.stock = sim.colony.stock;
        child.beliefs.relation_mut(a).opinion = 60.0;
        child.beliefs.relation_mut(b).opinion = 60.0;
        if let Some(home) = pa.home.and_then(|h| sim.world.structure_mut(h)) {
            home.residents.push(id);
        }
        let name = child.given_name.clone();
        sim.agents.push(child);
        sim.colony.stats.births += 1;
        let ticks = Calendar::ticks_per_day();
        for parent in [a, b] {
            if let Some(p) = sim.agent_mut(parent) {
                p.mind.add("child_born", Some(id), ticks);
                p.beliefs.relation_mut(id).opinion = 70.0;
            }
        }
        let text = line(
            "birth",
            sim.calendar.tick,
            &[
                ("name", &name),
                ("a", &pa.given_name),
                ("b", &pb.given_name),
            ],
        );
        sim.record(Category::Life, 2, text, vec![a, b, id]);
    }
}
