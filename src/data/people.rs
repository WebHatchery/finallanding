//! Survivor traits, ambitions, names and backstory fragments.

use super::kinds::{Need, Skill};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Personality weights a trait contributes. All default to zero so a trait
/// only states what it changes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TraitModifiers {
    #[serde(default)]
    pub sociability: f32,
    #[serde(default)]
    pub bravery: f32,
    #[serde(default)]
    pub kindness: f32,
    #[serde(default)]
    pub diligence: f32,
    #[serde(default)]
    pub curiosity: f32,
    #[serde(default)]
    pub romance: f32,
    #[serde(default)]
    pub temper: f32,
    #[serde(default)]
    pub optimism: f32,
    #[serde(default)]
    pub leadership: f32,
    #[serde(default)]
    pub night_owl: bool,
    #[serde(default)]
    pub need_decay: BTreeMap<Need, f32>,
    #[serde(default)]
    pub skill_speed: BTreeMap<Skill, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraitDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub modifiers: TraitModifiers,
    /// Traits that cannot appear on the same survivor.
    #[serde(default)]
    pub excludes: Vec<String>,
    /// Traits in others that this survivor finds grating.
    #[serde(default)]
    pub clashes_with: Vec<String>,
    /// Traits in others that this survivor warms to.
    #[serde(default)]
    pub bonds_with: Vec<String>,
    /// Support (+) or opposition (-) for each ending in the Divergence vote.
    #[serde(default)]
    pub ending_affinity: BTreeMap<String, f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbitionKind {
    MasterSkill,
    RaiseFamily,
    BefriendColony,
    MapTheWilds,
    DecodeSpire,
    BuildHome,
    LeadCouncil,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmbitionDef {
    pub id: String,
    pub kind: AmbitionKind,
    pub name: String,
    pub description: String,
    pub target: f32,
    #[serde(default)]
    pub ending_affinity: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleDef {
    pub name: String,
    pub skill: Skill,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameLists {
    pub given: Vec<String>,
    pub family: Vec<String>,
    pub origins: Vec<String>,
    pub former_roles: Vec<RoleDef>,
    pub memories: Vec<String>,
}
