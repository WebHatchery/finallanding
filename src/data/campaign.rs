//! Campaign acts, objectives, endings, landing sites and difficulties.

use super::kinds::{Branch, NodeKind, Season};
use super::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObjectiveKind {
    Beds { count: u32 },
    Stock { resource: Resource, amount: f32 },
    TechCount { count: u32 },
    BranchTechs { branch: Branch, count: u32 },
    Population { count: usize },
    Structure { building: String, count: u32 },
    Expeditions { count: u32 },
    Explored { fraction: f32 },
    Relics { count: f32 },
    PowerSurplus { amount: f32 },
    SurviveSeason { season: Season, year: u32 },
    AverageMood { at_least: f32 },
    Friendships { count: u32 },
    VoteHeld,
    CapstoneTech,
    CapstoneBuilt,
    DaysAfterCapstone { days: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectiveDef {
    pub id: String,
    pub text: String,
    pub kind: ObjectiveKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActDef {
    pub number: u8,
    pub name: String,
    pub intro: String,
    pub objectives: Vec<ObjectiveDef>,
    /// Scripted event fired when the act begins.
    #[serde(default)]
    pub opening_event: Option<String>,
    /// Scripted event fired once the act is half complete.
    #[serde(default)]
    pub climax_event: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EndingDef {
    pub id: String,
    pub name: String,
    pub pitch: String,
    pub capstone_tech: String,
    pub capstone_building: String,
    pub epilogue: String,
    pub final_crisis_event: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Climate {
    pub season_temperature: [f32; 4],
    pub temperature_swing: f32,
    pub storm_chance: [f32; 4],
    pub precipitation_chance: [f32; 4],
    #[serde(default)]
    pub snows: bool,
    #[serde(default)]
    pub ashfall: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainMix {
    pub ground: String,
    pub fertile: f32,
    pub rock: f32,
    pub water: f32,
    pub sand: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub climate: Climate,
    pub terrain: TerrainMix,
    pub node_density: BTreeMap<NodeKind, f32>,
    #[serde(default)]
    pub bonus_resources: BTreeMap<Resource, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifficultyDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub need_decay: f32,
    pub event_pressure: f32,
    pub starting_resources: f32,
    pub threat_scale: f32,
    pub research_cost: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignDef {
    pub acts: Vec<ActDef>,
    pub endings: Vec<EndingDef>,
    pub sites: Vec<SiteDef>,
    pub difficulties: Vec<DifficultyDef>,
}
