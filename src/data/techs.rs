//! Technology pool definitions. The per-run tree is generated from this pool.

use super::kinds::{Branch, Need, Skill};
use serde::{Deserialize, Serialize};

/// Passive colony-wide effects granted when a technology is researched.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TechEffect {
    WorkSpeed { skill: Skill, amount: f32 },
    NeedDecay { need: Need, amount: f32 },
    FarmYield { amount: f32 },
    ResearchSpeed { amount: f32 },
    Carry { amount: f32 },
    Mood { amount: f32 },
    HealRate { amount: f32 },
    ExpeditionSafety { amount: f32 },
    Defence { amount: f32 },
    Sight { amount: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub branch: Branch,
    pub tier: u8,
    pub cost: f32,
    /// Mutually exclusive group: researching one member locks the rest.
    #[serde(default)]
    pub fork: Option<String>,
    /// Ending this capstone technology enables.
    #[serde(default)]
    pub capstone: Option<String>,
    /// Prerequisites that are always present regardless of the run seed.
    #[serde(default)]
    pub fixed_prereqs: Vec<String>,
    /// Never hidden at the start of a run (core survival knowledge).
    #[serde(default)]
    pub always_known: bool,
    /// Only expeditions and relic study can reveal it.
    #[serde(default)]
    pub relic_only: bool,
    /// Species whose gathering inspires this technology; any one will do.
    /// Empty for knowledge the crew brought with them.
    #[serde(default)]
    pub inspired_by: Vec<String>,
    /// Units of those species the colony must gather before it is inspired.
    #[serde(default)]
    pub inspiration: f32,
    #[serde(default)]
    pub effects: Vec<TechEffect>,
}

impl TechDef {
    /// Revealed by gathering native species rather than by study alone.
    pub fn is_inspired(&self) -> bool {
        !self.inspired_by.is_empty()
    }
}
