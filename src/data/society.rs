//! Policies, thoughts, and chronicle text loaded from JSON.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PolicyEffects {
    #[serde(default = "one")]
    pub food_use: f32,
    #[serde(default)]
    pub mood: f32,
    #[serde(default)]
    pub work_hours: f32,
    #[serde(default)]
    pub curfew: bool,
    #[serde(default = "one")]
    pub child_chance: f32,
    #[serde(default = "one")]
    pub work_speed: f32,
    #[serde(default = "one")]
    pub volunteer_bias: f32,
    /// Extra mood for survivors with these traits while the option is active.
    #[serde(default)]
    pub trait_mood: BTreeMap<String, f32>,
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: PolicyEffects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_option: String,
    pub options: Vec<PolicyOption>,
    #[serde(default)]
    pub tech: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThoughtDef {
    pub id: String,
    pub text: String,
    pub mood: f32,
    pub days: f32,
    #[serde(default = "default_stack")]
    pub max_stack: u32,
}

fn default_stack() -> u32 {
    1
}

/// Player-facing strings: UI labels, chronicle templates with `{name}`-style
/// placeholders (several variants per key), tutorial and help pages.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextDef {
    pub labels: BTreeMap<String, String>,
    pub chronicle: BTreeMap<String, Vec<String>>,
    pub tutorial: Vec<TutorialStep>,
    pub help: Vec<HelpPage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HelpPage {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocietyDef {
    pub policies: Vec<PolicyDef>,
    pub thoughts: Vec<ThoughtDef>,
}
