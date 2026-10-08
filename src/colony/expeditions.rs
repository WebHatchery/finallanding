//! Expedition calls, parties in the field, and their reports.

use crate::world::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An open call for volunteers. Survivors decide for themselves whether to go.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpeditionCall {
    pub site: String,
    pub opened_tick: u64,
    pub volunteers: Vec<AgentId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveExpedition {
    pub id: u32,
    pub site: String,
    pub members: Vec<AgentId>,
    pub depart_tick: u64,
    pub return_tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpeditionReport {
    pub site: String,
    pub day: u32,
    pub summary: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Expeditions {
    pub revealed_sites: Vec<String>,
    pub visited_sites: Vec<String>,
    pub visit_counts: BTreeMap<String, f32>,
    pub call: Option<ExpeditionCall>,
    pub active: Vec<ActiveExpedition>,
    pub completed: u32,
    /// Calls in a row that closed without a party. Each one weighs on the
    /// survivors' sense of duty until someone goes.
    #[serde(default)]
    pub unanswered_calls: u32,
    pub next_id: u32,
    pub reports: Vec<ExpeditionReport>,
}

impl Expeditions {
    /// Repeat visits find less: each earlier visit to a site cuts rewards.
    pub fn reward_factor(&self, site: &str) -> f32 {
        let visits = self.visit_counts.get(site).copied().unwrap_or(0.0);
        0.75f32.powf(visits)
    }

    /// Sites recover between visits: one visit's worth every fifteen days.
    pub fn recover_sites(&mut self) {
        for visits in self.visit_counts.values_mut() {
            *visits = (*visits - 1.0 / 15.0).max(0.0);
        }
    }

    pub fn is_member(&self, agent: AgentId) -> bool {
        self.active.iter().any(|e| e.members.contains(&agent))
    }
}
