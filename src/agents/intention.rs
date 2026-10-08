//! Intentions: the goal a survivor is committed to and the plan in progress,
//! plus the decision log that makes their reasoning visible to the player.

use super::goals::Goal;
use super::plans::{FailReason, Plan};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intention {
    pub goal: Goal,
    pub plan: Option<Plan>,
    /// Plans already tried for this goal and abandoned after failure.
    pub tried: Vec<String>,
    pub utility: f32,
    pub started_tick: u64,
}

impl Intention {
    pub fn new(goal: Goal, utility: f32, tick: u64) -> Self {
        Self {
            goal,
            plan: None,
            tried: Vec::new(),
            utility,
            started_tick: tick,
        }
    }

    pub fn attempts(&self) -> usize {
        self.tried.len()
    }
}

/// Why the survivor changed course; shown in the inspector's Mind tab.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DecisionKind {
    Adopted {
        goal: Goal,
        utility: f32,
        reason: String,
    },
    ChosePlan {
        plan: String,
    },
    PlanFailed {
        plan: String,
        reason: FailReason,
    },
    GaveUp {
        goal: Goal,
    },
    Achieved {
        goal: Goal,
    },
    Interrupted {
        by: String,
    },
    Message {
        text: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub tick: u64,
    pub kind: DecisionKind,
}

const DECISION_LOG_LENGTH: usize = 12;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DecisionLog {
    pub entries: VecDeque<Decision>,
}

impl DecisionLog {
    pub fn push(&mut self, tick: u64, kind: DecisionKind) {
        self.entries.push_front(Decision { tick, kind });
        self.entries.truncate(DECISION_LOG_LENGTH);
    }
}
