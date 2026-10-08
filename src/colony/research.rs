//! Research focus, progress and branch insight.

use crate::data::{Branch, BRANCH_COUNT};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResearchState {
    /// The project the player asked for; researchers may choose their own
    /// when this is empty.
    pub focus: Option<String>,
    /// The project researchers are actually advancing.
    pub active: Option<String>,
    pub progress: BTreeMap<String, f32>,
    pub insight: [f32; BRANCH_COUNT],
    pub relics_studied: f32,
}

impl ResearchState {
    pub fn progress_of(&self, id: &str) -> f32 {
        self.progress.get(id).copied().unwrap_or(0.0)
    }

    pub fn add_insight(&mut self, branch: Branch, amount: f32) {
        self.insight[branch.index()] += amount;
    }
}
