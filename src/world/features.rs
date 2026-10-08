//! Landscape features survivors harvest, loose items on the ground, and fauna.

use super::geometry::{CreatureId, NodeId, Point, Tile};
use crate::data::finds::FindDef;
use crate::data::{game_data, CreatureKind, NodeKind, Resource};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub tile: Tile,
    pub amount: f32,
    pub max_amount: f32,
    /// The native species growing or lying here; wreckage has none.
    #[serde(default)]
    pub find: Option<String>,
}

impl ResourceNode {
    pub fn find_def(&self) -> Option<&'static FindDef> {
        self.find.as_deref().and_then(|id| game_data().find(id))
    }

    /// Gathering yield relative to the node kind's base rate.
    pub fn yield_scale(&self) -> f32 {
        self.find_def().map_or(1.0, |find| find.yield_scale)
    }

    pub fn resource(&self) -> Resource {
        match self.kind {
            NodeKind::Wreckage => Resource::Salvage,
            NodeKind::FibreGrove => Resource::Fibre,
            NodeKind::StoneOutcrop => Resource::Stone,
            NodeKind::OreVein => Resource::Metal,
            NodeKind::Forage => Resource::Food,
            NodeKind::Ruin => Resource::Relics,
        }
    }

    /// Living features regrow; mined and salvaged ones are spent for good.
    pub fn regrows(&self) -> bool {
        matches!(self.kind, NodeKind::FibreGrove | NodeKind::Forage)
    }

    /// Groves and bushes can be walked through; rocks, wrecks and ruins block.
    pub fn blocks_movement(&self) -> bool {
        !matches!(self.kind, NodeKind::FibreGrove | NodeKind::Forage)
    }

    pub fn is_depleted(&self) -> bool {
        self.amount < 0.5
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroundItem {
    pub tile: Tile,
    pub resource: Resource,
    pub amount: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatureMood {
    Approaching,
    Raiding,
    Fleeing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Creature {
    pub id: CreatureId,
    pub kind: CreatureKind,
    pub tile: Tile,
    pub position: Point,
    pub path: VecDeque<Tile>,
    pub move_budget: f32,
    pub health: f32,
    pub mood: CreatureMood,
    pub target: Option<Tile>,
    pub ticks_alive: u32,
    pub stolen: f32,
}
