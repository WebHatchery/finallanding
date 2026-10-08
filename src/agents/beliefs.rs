//! What a survivor believes about the world and the people in it.
//!
//! Beliefs come only from perception, from messages, and from the colony's
//! daily broadcast. They can be stale or wrong, which is what makes plans fail
//! and forces survivors to adapt.

use crate::data::{game_data, NodeKind, Resource, ResourceBag};
use crate::world::{AgentId, CreatureId, NodeId, Tile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NodeBelief {
    pub kind: NodeKind,
    pub tile: Tile,
    pub amount: f32,
    pub seen_tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ThreatBelief {
    pub creature: CreatureId,
    pub tile: Tile,
    pub dangerous: bool,
    pub seen_tick: u64,
}

/// Directional feelings toward another survivor.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub opinion: f32,
    pub familiarity: f32,
    pub romance: f32,
    pub last_chat_tick: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bond {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    Rival,
    Enemy,
}

impl Bond {
    pub fn label_key(self) -> &'static str {
        match self {
            Bond::Stranger => "bond_stranger",
            Bond::Acquaintance => "bond_acquaintance",
            Bond::Friend => "bond_friend",
            Bond::CloseFriend => "bond_close_friend",
            Bond::Rival => "bond_rival",
            Bond::Enemy => "bond_enemy",
        }
    }
}

impl Relation {
    pub fn bond(&self) -> Bond {
        let social = &game_data().balance.social;
        if self.opinion <= social.enemy_threshold {
            Bond::Enemy
        } else if self.opinion <= social.rival_threshold {
            Bond::Rival
        } else if self.opinion >= social.close_friend_threshold {
            Bond::CloseFriend
        } else if self.opinion >= social.friend_threshold {
            Bond::Friend
        } else if self.familiarity >= 10.0 {
            Bond::Acquaintance
        } else {
            Bond::Stranger
        }
    }

    pub fn change_opinion(&mut self, amount: f32) {
        self.opinion = (self.opinion + amount).clamp(-100.0, 100.0);
    }
}

/// A target that recently failed, remembered so plans avoid repeating it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FailedTarget {
    Node(NodeId),
    Structure(u32),
    Tile(Tile),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Beliefs {
    pub nodes: BTreeMap<NodeId, NodeBelief>,
    pub threats: Vec<ThreatBelief>,
    pub relations: BTreeMap<AgentId, Relation>,
    /// The stockpile as last heard in the daily broadcast or seen in person.
    pub stock: ResourceBag,
    /// Recently failed targets and when; a list because JSON maps need string keys.
    pub failed: Vec<(FailedTarget, u64)>,
    /// Survivors seen this perception pass, with where they were.
    pub nearby: Vec<(AgentId, Tile)>,
}

impl Beliefs {
    pub fn relation(&self, other: AgentId) -> Option<&Relation> {
        self.relations.get(&other)
    }

    pub fn relation_mut(&mut self, other: AgentId) -> &mut Relation {
        self.relations.entry(other).or_default()
    }

    pub fn opinion_of(&self, other: AgentId) -> f32 {
        self.relations.get(&other).map(|r| r.opinion).unwrap_or(0.0)
    }

    pub fn note_node(&mut self, id: NodeId, belief: NodeBelief) {
        self.nodes.insert(id, belief);
    }

    pub fn forget_node(&mut self, id: NodeId) {
        self.nodes.remove(&id);
    }

    pub fn note_failure(&mut self, target: FailedTarget, tick: u64) {
        self.failed.retain(|(existing, _)| *existing != target);
        self.failed.push((target, tick));
    }

    pub fn recently_failed(&self, target: FailedTarget, now: u64, window: u64) -> bool {
        self.failed
            .iter()
            .any(|(failed, tick)| *failed == target && now.saturating_sub(*tick) < window)
    }

    /// Known nodes of a kind with something left, nearest first.
    pub fn known_nodes(&self, kind: NodeKind, from: Tile) -> Vec<(NodeId, Tile)> {
        let mut found: Vec<(NodeId, Tile)> = self
            .nodes
            .iter()
            .filter(|(_, belief)| belief.kind == kind && belief.amount >= 1.0)
            .map(|(id, belief)| (*id, belief.tile))
            .collect();
        found.sort_by(|a, b| a.1.distance(from).total_cmp(&b.1.distance(from)));
        found
    }

    pub fn believes_stock(&self, resource: Resource, amount: f32) -> bool {
        self.stock.get(resource) >= amount
    }

    pub fn note_threat(&mut self, belief: ThreatBelief) {
        if let Some(existing) = self
            .threats
            .iter_mut()
            .find(|threat| threat.creature == belief.creature)
        {
            *existing = belief;
        } else {
            self.threats.push(belief);
        }
    }

    /// The nearest believed threat, and how far it is.
    pub fn nearest_threat(&self, from: Tile) -> Option<(&ThreatBelief, f32)> {
        self.threats
            .iter()
            .map(|threat| (threat, threat.tile.distance(from)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Drop stale beliefs: old node sightings, departed threats, old failures.
    pub fn forget_stale(&mut self, now: u64, ticks_per_day: u64) {
        let forget_after =
            (game_data().balance.agents.belief_forget_days * ticks_per_day as f32) as u64;
        self.nodes
            .retain(|_, belief| now.saturating_sub(belief.seen_tick) < forget_after);
        self.threats
            .retain(|threat| now.saturating_sub(threat.seen_tick) < ticks_per_day / 12);
        self.failed
            .retain(|(_, tick)| now.saturating_sub(*tick) < ticks_per_day);
    }
}
