//! The Divergence vote: survivors judge the proposed future by their traits,
//! ambitions, experiences and the opinions of people they trust.

use super::chronicle_text::line;
use super::lifecycle::depart;
use super::Sim;
use crate::agents::Agent;
use crate::colony::campaign::{Ballot, VoteResult};
use crate::colony::chronicle::Category;
use crate::data::{game_data, Branch};
use crate::world::{AgentId, Calendar};
use macroquad_toolkit::math::hash_str;

fn experience(sim: &Sim, ending: &str) -> f32 {
    let colony = &sim.colony;
    match ending {
        "ascendant" => {
            (colony.stats.relics_found / 600.0).min(0.2)
                + colony.tree.researched_in(Branch::Xenology) as f32 * 0.02
        }
        "beacon" => {
            (colony.stats.deaths as f32 * 0.04).min(0.25)
                + colony.tree.researched_in(Branch::Industry) as f32 * 0.02
        }
        "rootbound" => {
            (sim.world.built().filter(|s| s.crop.is_some()).count() as f32 * 0.02).min(0.2)
                + colony.tree.researched_in(Branch::Agronomy) as f32 * 0.02
        }
        _ => 0.0,
    }
}

/// A survivor's own leaning, before listening to anyone else.
fn personal_leaning(sim: &Sim, agent: &Agent, ending: &str) -> f32 {
    let data = game_data();
    let mut support = experience(sim, ending);
    for trait_id in &agent.traits {
        if let Some(def) = data.trait_def(trait_id) {
            support += def.ending_affinity.get(ending).copied().unwrap_or(0.0);
        }
    }
    if let Some(ambition) = agent.ambition_def() {
        support += ambition.ending_affinity.get(ending).copied().unwrap_or(0.0);
    }
    let noise = (hash_str(&format!("{}:{ending}", agent.id)) % 200) as f32 / 1000.0 - 0.1;
    support + noise + (agent.mood - 50.0) / 500.0
}

/// How much a survivor prefers this future over the alternatives: the three
/// futures compete, so support is relative to the survivor's average leaning.
fn relative_leaning(sim: &Sim, agent: &Agent, ending: &str) -> f32 {
    let endings = &game_data().campaign.endings;
    let mean = endings
        .iter()
        .map(|e| personal_leaning(sim, agent, &e.id))
        .sum::<f32>()
        / endings.len().max(1) as f32;
    personal_leaning(sim, agent, ending) - mean
}

/// Each survivor's support (positive) or opposition (negative), including the
/// pull of friends and respected leaders.
pub fn projected_support(sim: &Sim, ending: &str) -> Vec<(AgentId, f32)> {
    let voters: Vec<&Agent> = sim
        .agents
        .iter()
        .filter(|a| a.is_alive() && !a.is_child)
        .collect();
    let leanings: Vec<(AgentId, f32, f32)> = voters
        .iter()
        .map(|a| {
            (
                a.id,
                relative_leaning(sim, a, ending),
                a.personality.leadership,
            )
        })
        .collect();
    voters
        .iter()
        .map(|agent| {
            let own = leanings
                .iter()
                .find(|(id, _, _)| *id == agent.id)
                .map(|l| l.1)
                .unwrap_or(0.0);
            let mut pull = 0.0;
            let mut weight = 0.0;
            for (other, leaning, leadership) in &leanings {
                if *other == agent.id {
                    continue;
                }
                let trust = agent.beliefs.opinion_of(*other).max(0.0) / 100.0 * (1.0 + leadership);
                pull += leaning * trust;
                weight += trust;
            }
            let social = if weight > 0.0 {
                pull / weight * 0.35
            } else {
                0.0
            };
            (agent.id, own + social)
        })
        .collect()
}

pub fn hold_vote(sim: &mut Sim, ending: &str) -> bool {
    if sim.colony.campaign.vote.is_some() || game_data().ending(ending).is_none() {
        return false;
    }
    let support = projected_support(sim, ending);
    let ballots: Vec<Ballot> = support
        .iter()
        .map(|(agent, s)| Ballot {
            agent: *agent,
            support: *s,
        })
        .collect();
    let result = VoteResult {
        ending: ending.to_owned(),
        ballots,
        day: sim.calendar.day(),
    };
    let passed = result.passed();
    let supporters = result.supporters();
    let total = result.ballots.len();
    sim.colony.campaign.chosen_ending = Some(ending.to_owned());
    sim.colony.campaign.vote = Some(result);
    let ticks = Calendar::ticks_per_day();
    let mut leaving = Vec::new();
    for (id, value) in &support {
        let Some(agent) = sim.agent_mut(*id) else {
            continue;
        };
        if *value > 0.0 {
            agent.mind.add("vote_won", None, ticks);
        } else {
            agent.mind.add("vote_lost", None, ticks);
            if *value < -0.35 && agent.partner.is_none() && !passed {
                leaving.push(*id);
            }
        }
    }
    let name = game_data()
        .ending(ending)
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let key = if passed {
        "vote_passed"
    } else {
        "vote_divided"
    };
    let text = line(
        key,
        sim.calendar.tick,
        &[
            ("ending", &name),
            ("yes", &supporters.to_string()),
            ("total", &total.to_string()),
        ],
    );
    sim.record(Category::Story, 2, text, Vec::new());
    for id in leaving.into_iter().take(2) {
        if sim.rng.chance(0.5) {
            depart(sim, id);
        }
    }
    true
}
