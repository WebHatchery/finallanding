//! Seeded creation of survivors: names, traits, skills, passions, ambitions.

use super::beliefs::Beliefs;
use super::body::{Health, Mind, Needs};
use super::intention::DecisionLog;
use super::personality::{Personality, Skills};
use super::{Activity, Agent, AgentStats, AmbitionState, Backstory, LifeState};
use crate::data::people::AmbitionKind;
use crate::data::{game_data, Skill, SKILL_COUNT};
use crate::world::{AgentId, Point, Tile};
use macroquad_toolkit::rng::SeededRng;
use std::collections::VecDeque;

/// Everything needed to place a new survivor in the world.
pub struct Arrival<'a> {
    pub id: AgentId,
    pub tile: Tile,
    pub day: u32,
    pub taken_names: &'a [String],
}

fn pick<'a>(rng: &mut SeededRng, items: &'a [String]) -> &'a str {
    rng.choose(items).map(String::as_str).unwrap_or("")
}

fn unique_given_name(rng: &mut SeededRng, taken: &[String]) -> String {
    let names = &game_data().names.given;
    for _ in 0..30 {
        let candidate = pick(rng, names);
        if !taken.iter().any(|name| name == candidate) {
            return candidate.to_owned();
        }
    }
    pick(rng, names).to_owned()
}

fn roll_traits(rng: &mut SeededRng, count: usize) -> Vec<String> {
    let traits = &game_data().traits;
    let mut chosen: Vec<String> = Vec::new();
    let mut attempts = 0;
    while chosen.len() < count && attempts < 60 {
        attempts += 1;
        let Some(candidate) = rng.choose(traits) else {
            break;
        };
        let conflicts = chosen.iter().any(|existing| {
            existing == &candidate.id
                || candidate.excludes.contains(existing)
                || game_data()
                    .trait_def(existing)
                    .is_some_and(|def| def.excludes.contains(&candidate.id))
        });
        if !conflicts {
            chosen.push(candidate.id.clone());
        }
    }
    chosen
}

fn roll_skills(rng: &mut SeededRng, role_skill: Skill) -> Skills {
    let mut levels = [0u8; SKILL_COUNT];
    for level in levels.iter_mut() {
        *level = rng.range_i32(0, 5) as u8;
    }
    levels[role_skill.index()] = rng.range_i32(5, 9) as u8;
    let mut passions = vec![role_skill];
    let second = Skill::ALL[rng.below(SKILL_COUNT)];
    if second != role_skill {
        passions.push(second);
        levels[second.index()] = levels[second.index()].max(3);
    }
    Skills {
        levels,
        xp: [0.0; SKILL_COUNT],
        passions,
    }
}

fn roll_ambition(rng: &mut SeededRng) -> AmbitionState {
    let ambitions = &game_data().ambitions;
    let id = rng
        .choose(ambitions)
        .map(|ambition| ambition.id.clone())
        .unwrap_or_default();
    AmbitionState {
        id,
        progress: 0.0,
        fulfilled: false,
    }
}

fn blank_agent(arrival: &Arrival, given: String, family: String, traits: Vec<String>) -> Agent {
    let personality = Personality::from_traits(&traits);
    Agent {
        id: arrival.id,
        given_name: given,
        family_name: family,
        age_years: 30.0,
        is_child: false,
        portrait: 0,
        hue: 0.0,
        backstory: Backstory {
            origin: String::new(),
            former_role: String::new(),
            memory: String::new(),
        },
        traits,
        personality,
        skills: Skills::default(),
        ambition: AmbitionState {
            id: String::new(),
            progress: 0.0,
            fulfilled: false,
        },
        needs: Needs::default(),
        health: Health::default(),
        mind: Mind::default(),
        mood: 50.0,
        beliefs: Beliefs::default(),
        intention: None,
        decisions: DecisionLog::default(),
        inbox: Vec::new(),
        carrying: None,
        home: None,
        partner: None,
        parents: Vec::new(),
        life: LifeState::Present,
        tile: arrival.tile,
        position: Point::of_tile(arrival.tile),
        path: VecDeque::new(),
        move_budget: 0.0,
        inside: None,
        activity: Activity::Idle,
        backoff: Vec::new(),
        mental_break: None,
        low_mood_ticks: 0,
        joined_day: arrival.day,
        stats: AgentStats::default(),
        next_deliberation: 0,
        volunteered: false,
        help_task: None,
    }
}

/// Create an adult survivor with a generated history.
pub fn create_survivor(arrival: &Arrival, rng: &mut SeededRng) -> Agent {
    let names = &game_data().names;
    let given = unique_given_name(rng, arrival.taken_names);
    let family = pick(rng, &names.family).to_owned();
    let trait_count = if rng.chance(0.45) { 3 } else { 2 };
    let traits = roll_traits(rng, trait_count);
    let mut agent = blank_agent(arrival, given, family, traits);
    let role = rng.choose(&names.former_roles).cloned();
    let role_skill = role
        .as_ref()
        .map(|r| r.skill)
        .unwrap_or(Skill::Construction);
    agent.skills = roll_skills(rng, role_skill);
    agent.ambition = roll_ambition(rng);
    agent.age_years = rng.range_f32(21.0, 62.0).floor();
    agent.portrait = rng.below(6) as u8;
    agent.hue = rng.range_f32(-25.0, 25.0);
    agent.backstory = Backstory {
        origin: pick(rng, &names.origins).to_owned(),
        former_role: role.map(|r| r.name).unwrap_or_default(),
        memory: pick(rng, &names.memories).to_owned(),
    };
    for need in agent.needs.values.iter_mut() {
        *need = rng.range_f32(60.0, 90.0);
    }
    agent
}

/// Create a child of two survivors, inheriting one trait from each parent.
pub fn create_child(arrival: &Arrival, parents: (&Agent, &Agent), rng: &mut SeededRng) -> Agent {
    let mut traits = Vec::new();
    for parent in [parents.0, parents.1] {
        if let Some(inherited) = rng.choose(&parent.traits) {
            if !traits.contains(inherited) {
                traits.push(inherited.clone());
            }
        }
    }
    let given = unique_given_name(rng, arrival.taken_names);
    let family = parents.0.family_name.clone();
    let mut agent = blank_agent(arrival, given, family, traits);
    agent.skills = Skills {
        passions: vec![Skill::ALL[rng.below(SKILL_COUNT)]],
        ..Skills::default()
    };
    agent.ambition = roll_ambition(rng);
    agent.age_years = 0.0;
    agent.is_child = true;
    agent.portrait = if rng.chance(0.5) {
        parents.0.portrait
    } else {
        parents.1.portrait
    };
    agent.hue = (parents.0.hue + parents.1.hue) * 0.5;
    agent.parents = vec![parents.0.id, parents.1.id];
    agent.backstory = Backstory {
        origin: game_data().label("born_here").to_owned(),
        former_role: String::new(),
        memory: String::new(),
    };
    agent
}

/// The skill an ambition to master a craft targets: the survivor's first passion.
pub fn mastery_skill(agent: &Agent) -> Skill {
    agent
        .skills
        .passions
        .first()
        .copied()
        .unwrap_or(agent.skills.best())
}

pub fn ambition_kind(agent: &Agent) -> Option<AmbitionKind> {
    agent.ambition_def().map(|def| def.kind)
}
