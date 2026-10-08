//! Typed messages exchanged between survivors (the social protocol layer).

use super::beliefs::{NodeBelief, ThreatBelief};
use super::goals::HelpNeed;
use crate::world::{AgentId, NodeId, Tile};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageKind {
    /// Share a belief about a resource node.
    InformNode {
        node: NodeId,
        belief: NodeBelief,
    },
    /// Warn of a creature.
    Warn(ThreatBelief),
    /// Ask for help with a need, at a place.
    RequestHelp {
        need: HelpNeed,
        place: Tile,
    },
    AcceptHelp {
        need: HelpNeed,
    },
    DeclineHelp {
        need: HelpNeed,
    },
    /// A handed-over item has arrived.
    Delivered,
    /// Pass on how the sender feels about a third survivor.
    Gossip {
        about: AgentId,
        opinion: f32,
    },
    /// Romantic proposal; the receiver decides.
    Propose,
    AcceptProposal,
    RejectProposal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub from: AgentId,
    pub to: AgentId,
    pub kind: MessageKind,
    pub tick: u64,
}
