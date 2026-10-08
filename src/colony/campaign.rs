//! Campaign progress: the current act, objectives, the vote and the outcome.

use crate::world::AgentId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Outcome {
    Victory { ending: String, day: u32 },
    Failure { reason: String, day: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ballot {
    pub agent: AgentId,
    pub support: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteResult {
    pub ending: String,
    pub ballots: Vec<Ballot>,
    pub day: u32,
}

impl VoteResult {
    pub fn supporters(&self) -> usize {
        self.ballots.iter().filter(|b| b.support > 0.0).count()
    }

    pub fn passed(&self) -> bool {
        self.supporters() * 2 > self.ballots.len()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignState {
    pub act: u8,
    pub act_started_day: u32,
    pub completed: Vec<String>,
    pub climax_fired: bool,
    pub chosen_ending: Option<String>,
    pub vote: Option<VoteResult>,
    pub capstone_built_day: Option<u32>,
    pub final_crisis_fired: bool,
    pub outcome: Option<Outcome>,
    pub winters_survived: Vec<u32>,
    pub low_mood_days: u32,
}

impl Default for CampaignState {
    fn default() -> Self {
        Self {
            act: 1,
            act_started_day: 1,
            completed: Vec::new(),
            climax_fired: false,
            chosen_ending: None,
            vote: None,
            capstone_built_day: None,
            final_crisis_fired: false,
            outcome: None,
            winters_survived: Vec::new(),
            low_mood_days: 0,
        }
    }
}

impl CampaignState {
    pub fn objective_key(act: u8, id: &str) -> String {
        format!("{act}:{id}")
    }

    pub fn is_complete(&self, act: u8, id: &str) -> bool {
        self.completed.contains(&Self::objective_key(act, id))
    }

    pub fn is_over(&self) -> bool {
        self.outcome.is_some()
    }
}
