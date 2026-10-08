//! Event deck definitions drawn by the colony director.

use super::kinds::{Branch, CreatureKind, Season, Weather};
use super::resources::Resource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Calm,
    Notable,
    Danger,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventEffect {
    Weather { weather: Weather, days: f32 },
    Creatures { kind: CreatureKind, count: u32 },
    Resource { resource: Resource, amount: f32 },
    Arrivals { count: u32 },
    Illness { count: u32 },
    Blight { fraction: f32 },
    Wreckage { nodes: u32 },
    Insight { branch: Branch, amount: f32 },
    RevealTech,
    Damage { fraction: f32 },
    Mood { amount: f32, days: f32 },
    Expedition { site: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventDef {
    pub id: String,
    pub title: String,
    pub text: String,
    pub severity: Severity,
    pub weight: f32,
    #[serde(default)]
    pub min_day: u32,
    #[serde(default)]
    pub max_day: Option<u32>,
    #[serde(default)]
    pub acts: Vec<u8>,
    #[serde(default)]
    pub seasons: Vec<Season>,
    #[serde(default)]
    pub sites: Vec<String>,
    #[serde(default)]
    pub cooldown_days: u32,
    #[serde(default)]
    pub once: bool,
    /// Scripted events are only fired by the campaign, never drawn at random.
    #[serde(default)]
    pub scripted: bool,
    #[serde(default)]
    pub min_population: usize,
    pub effects: Vec<EventEffect>,
}

/// Off-map destinations survivors can travel to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpeditionSiteDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub danger: f32,
    pub days: f32,
    pub rewards: Vec<(Resource, f32)>,
    #[serde(default)]
    pub relics: f32,
    #[serde(default)]
    pub min_act: u8,
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default)]
    pub reveals_tech: bool,
    #[serde(default)]
    pub recruits: bool,
}
