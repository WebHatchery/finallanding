//! Native species: the particular foods, fibres, stones, ores and ruins a
//! landing site holds. Every run draws its own set, and gathering them is
//! what inspires most of the colony's technologies.

use super::kinds::NodeKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How a cultivable food behaves once it is planted in an orchard.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CropTraits {
    /// Harvest size relative to the orchard's base yield.
    pub yield_scale: f32,
    /// Growing time relative to the orchard's base growing time.
    pub grow_scale: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FindDef {
    pub id: String,
    /// The kind of landscape feature this species grows or lies in.
    pub node: NodeKind,
    pub name: String,
    pub description: String,
    pub color: [u8; 3],
    /// Gathering yield relative to the node kind's base rate.
    pub yield_scale: f32,
    /// Terrain the species clusters beside, if it has a preference.
    #[serde(default)]
    pub near: Option<String>,
    /// Present on foods that can be planted.
    #[serde(default)]
    pub crop: Option<CropTraits>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FindsFile {
    /// How many species of each node kind a landing site holds.
    pub species_per_run: BTreeMap<NodeKind, usize>,
    pub finds: Vec<FindDef>,
}
