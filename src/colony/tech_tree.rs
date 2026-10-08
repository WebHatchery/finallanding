//! The per-run technology tree, regenerated from the pool for every seed.
//!
//! Prerequisites are drawn from earlier tiers (mostly the same branch), about a
//! third of technologies start hidden, forks lock their alternatives, and costs
//! fall as the colony practises a branch.

use crate::data::techs::TechEffect;
use crate::data::{game_data, Branch, NEED_COUNT, SKILL_COUNT};
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechStatus {
    Hidden,
    Locked,
    Waiting,
    Available,
    Researched,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechNode {
    pub id: String,
    pub prereqs: Vec<String>,
    pub revealed: bool,
    pub researched: bool,
    pub locked: bool,
    pub researched_day: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TechTree {
    pub nodes: Vec<TechNode>,
}

fn choose_prereqs(tier: u8, branch: Branch, rng: &mut SeededRng) -> Vec<String> {
    let data = game_data();
    if tier <= 1 {
        return Vec::new();
    }
    let cross = data.balance.research.cross_branch_chance;
    let count = if rng.chance(0.4) { 2 } else { 1 };
    let mut chosen: Vec<String> = Vec::new();
    for _ in 0..count {
        let source_tier = if tier > 2 && rng.chance(0.3) {
            tier - 2
        } else {
            tier - 1
        };
        let same_branch = !rng.chance(cross);
        let candidates: Vec<&str> = data
            .techs
            .iter()
            .filter(|t| t.tier == source_tier && t.capstone.is_none())
            .filter(|t| !same_branch || t.branch == branch)
            .filter(|t| !chosen.contains(&t.id))
            .map(|t| t.id.as_str())
            .collect();
        let pool: Vec<&str> = if candidates.is_empty() {
            data.techs
                .iter()
                .filter(|t| t.tier == source_tier && !chosen.contains(&t.id))
                .map(|t| t.id.as_str())
                .collect()
        } else {
            candidates
        };
        if let Some(pick) = rng.choose(&pool) {
            chosen.push((*pick).to_owned());
        }
    }
    chosen
}

impl TechTree {
    pub fn generate(rng: &mut SeededRng) -> Self {
        let data = game_data();
        let hidden_fraction = data.balance.research.hidden_fraction;
        let nodes = data
            .techs
            .iter()
            .map(|def| {
                let mut prereqs = def.fixed_prereqs.clone();
                for extra in choose_prereqs(def.tier, def.branch, rng) {
                    if !prereqs.contains(&extra) {
                        prereqs.push(extra);
                    }
                }
                let hidden_chance = hidden_fraction * (def.tier as f32 / 3.0).min(1.3);
                let revealed = if def.always_known || def.capstone.is_some() {
                    true
                } else if def.relic_only {
                    false
                } else {
                    !rng.chance(hidden_chance)
                };
                TechNode {
                    id: def.id.clone(),
                    prereqs,
                    revealed,
                    researched: false,
                    locked: false,
                    researched_day: None,
                }
            })
            .collect();
        TechTree { nodes }
    }

    pub fn node(&self, id: &str) -> Option<&TechNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    fn node_mut(&mut self, id: &str) -> Option<&mut TechNode> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    pub fn is_researched(&self, id: &str) -> bool {
        self.node(id).is_some_and(|node| node.researched)
    }

    pub fn status(&self, id: &str) -> TechStatus {
        let Some(node) = self.node(id) else {
            return TechStatus::Hidden;
        };
        if node.researched {
            TechStatus::Researched
        } else if node.locked {
            TechStatus::Locked
        } else if !node.revealed {
            TechStatus::Hidden
        } else if node.prereqs.iter().all(|p| self.is_researched(p)) {
            TechStatus::Available
        } else {
            TechStatus::Waiting
        }
    }

    pub fn available(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|node| self.status(&node.id) == TechStatus::Available)
            .map(|node| node.id.as_str())
            .collect()
    }

    pub fn researched_count(&self) -> usize {
        self.nodes.iter().filter(|node| node.researched).count()
    }

    pub fn researched_in(&self, branch: Branch) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.researched)
            .filter(|node| {
                game_data()
                    .tech(&node.id)
                    .is_some_and(|t| t.branch == branch)
            })
            .count()
    }

    /// Mark researched and lock fork alternatives. Returns locked tech ids.
    pub fn complete(&mut self, id: &str, day: u32) -> Vec<String> {
        let fork = game_data().tech(id).and_then(|t| t.fork.clone());
        if let Some(node) = self.node_mut(id) {
            node.researched = true;
            node.revealed = true;
            node.researched_day = Some(day);
        }
        let mut locked = Vec::new();
        if let Some(fork) = fork {
            for node in self.nodes.iter_mut() {
                let same_fork = game_data()
                    .tech(&node.id)
                    .is_some_and(|t| t.fork.as_deref() == Some(fork.as_str()));
                if same_fork && node.id != id && !node.researched {
                    node.locked = true;
                    locked.push(node.id.clone());
                }
            }
        }
        locked
    }

    pub fn reveal(&mut self, id: &str) -> bool {
        match self.node_mut(id) {
            Some(node) if !node.revealed => {
                node.revealed = true;
                true
            }
            _ => false,
        }
    }

    /// Hidden technologies in a branch whose prerequisites are mostly known:
    /// the candidates for a eureka. Relic-only techs need relics instead.
    pub fn discoverable(&self, branch: Option<Branch>, allow_relic: bool) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|node| !node.revealed && !node.locked)
            .filter(|node| {
                let Some(def) = game_data().tech(&node.id) else {
                    return false;
                };
                let branch_ok = branch.is_none_or(|b| def.branch == b);
                let relic_ok = allow_relic || !def.relic_only;
                let known_prereqs = node
                    .prereqs
                    .iter()
                    .filter(|p| self.node(p).is_some_and(|n| n.revealed))
                    .count();
                branch_ok && relic_ok && known_prereqs * 2 >= node.prereqs.len()
            })
            .map(|node| node.id.clone())
            .collect()
    }

    /// Research cost after difficulty and branch practice.
    pub fn cost(&self, id: &str, difficulty_scale: f32) -> f32 {
        let data = game_data();
        let Some(def) = data.tech(id) else {
            return f32::MAX;
        };
        let research = &data.balance.research;
        let practice = (self.researched_in(def.branch) as f32
            * research.practice_discount_per_tech)
            .min(research.max_practice_discount);
        def.cost * difficulty_scale * (1.0 - practice)
    }
}

/// The colony-wide sum of researched technology effects.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Modifiers {
    pub work_speed: [f32; SKILL_COUNT],
    pub need_decay: [f32; NEED_COUNT],
    pub farm_yield: f32,
    pub research_speed: f32,
    pub carry: f32,
    pub mood: f32,
    pub heal_rate: f32,
    pub expedition_safety: f32,
    pub defence: f32,
    pub sight: f32,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            work_speed: [1.0; SKILL_COUNT],
            need_decay: [1.0; NEED_COUNT],
            farm_yield: 1.0,
            research_speed: 1.0,
            carry: 0.0,
            mood: 0.0,
            heal_rate: 1.0,
            expedition_safety: 0.0,
            defence: 1.0,
            sight: 0.0,
        }
    }
}

impl Modifiers {
    pub fn from_tree(tree: &TechTree) -> Self {
        let mut modifiers = Modifiers::default();
        for node in tree.nodes.iter().filter(|n| n.researched) {
            let Some(def) = game_data().tech(&node.id) else {
                continue;
            };
            for effect in &def.effects {
                modifiers.apply(effect);
            }
        }
        modifiers
    }

    fn apply(&mut self, effect: &TechEffect) {
        match effect {
            TechEffect::WorkSpeed { skill, amount } => self.work_speed[skill.index()] += amount,
            TechEffect::NeedDecay { need, amount } => {
                let decay = &mut self.need_decay[need.index()];
                *decay = (*decay + amount).max(0.2);
            }
            TechEffect::FarmYield { amount } => self.farm_yield += amount,
            TechEffect::ResearchSpeed { amount } => self.research_speed += amount,
            TechEffect::Carry { amount } => self.carry += amount,
            TechEffect::Mood { amount } => self.mood += amount,
            TechEffect::HealRate { amount } => self.heal_rate += amount,
            TechEffect::ExpeditionSafety { amount } => self.expedition_safety += amount,
            TechEffect::Defence { amount } => self.defence += amount,
            TechEffect::Sight { amount } => self.sight += amount,
        }
    }
}
