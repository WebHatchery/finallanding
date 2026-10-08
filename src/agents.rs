//! Survivors as BDI agents designed with the Prometheus methodology.
//!
//! Each survivor perceives only its surroundings (situated), generates its own
//! goals (autonomous), reacts to interrupting percepts (reactive), commits to
//! goals across interruptions (proactive), chooses among several plans per goal
//! (flexible), recovers from plan failure (robust), and exchanges messages with
//! others (social). The simulation drives the cycle in `sim::cognition`.

pub mod beliefs;
pub mod body;
pub mod deliberation;
pub mod generation;
pub mod goals;
pub mod intention;
pub mod messages;
pub mod personality;
pub mod plan_library;
pub mod plans;

use crate::data::{game_data, Resource, Skill};
use crate::world::{AgentId, Point, StructureId, Tile};
use beliefs::Beliefs;
use body::{Health, Mind, Needs};
use goals::{BreakKind, Goal, GoalKey, HelpNeed};
use intention::{DecisionLog, Intention};
use messages::Message;
use personality::{Personality, Skills};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LifeState {
    Present,
    Away { expedition: u32 },
    Dead { day: u32, cause: String },
    Departed { day: u32 },
}

/// What the survivor is visibly doing, for rendering and the roster.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Idle,
    Walking,
    Working(Skill),
    Hauling,
    Eating,
    Sleeping,
    Talking,
    Relaxing,
    Fleeing,
    Fighting,
    Recovering,
    Waiting,
    Breaking,
}

impl Activity {
    pub fn label_key(self) -> &'static str {
        match self {
            Activity::Idle => "activity_idle",
            Activity::Walking => "activity_walking",
            Activity::Working(_) => "activity_working",
            Activity::Hauling => "activity_hauling",
            Activity::Eating => "activity_eating",
            Activity::Sleeping => "activity_sleeping",
            Activity::Talking => "activity_talking",
            Activity::Relaxing => "activity_relaxing",
            Activity::Fleeing => "activity_fleeing",
            Activity::Fighting => "activity_fighting",
            Activity::Recovering => "activity_recovering",
            Activity::Waiting => "activity_waiting",
            Activity::Breaking => "activity_breaking",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backstory {
    pub origin: String,
    pub former_role: String,
    pub memory: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmbitionState {
    pub id: String,
    pub progress: f32,
    pub fulfilled: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentStats {
    pub expeditions: u32,
    pub conversations: u32,
    pub structures_built: u32,
    pub eurekas: u32,
    pub creatures_driven_off: u32,
    pub plans_recovered: u32,
    pub helped_others: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub given_name: String,
    pub family_name: String,
    pub age_years: f32,
    pub is_child: bool,
    pub portrait: u8,
    pub hue: f32,
    pub backstory: Backstory,
    pub traits: Vec<String>,
    pub personality: Personality,
    pub skills: Skills,
    pub ambition: AmbitionState,
    pub needs: Needs,
    pub health: Health,
    pub mind: Mind,
    pub mood: f32,
    pub beliefs: Beliefs,
    pub intention: Option<Intention>,
    pub decisions: DecisionLog,
    pub inbox: Vec<Message>,
    pub carrying: Option<(Resource, f32)>,
    pub home: Option<StructureId>,
    pub partner: Option<AgentId>,
    pub parents: Vec<AgentId>,
    pub life: LifeState,
    pub tile: Tile,
    pub position: Point,
    pub path: VecDeque<Tile>,
    pub move_budget: f32,
    pub inside: Option<StructureId>,
    pub activity: Activity,
    pub backoff: Vec<(GoalKey, u64)>,
    pub mental_break: Option<(BreakKind, u64)>,
    pub low_mood_ticks: u64,
    pub joined_day: u32,
    pub stats: AgentStats,
    pub next_deliberation: u64,
    pub volunteered: bool,
    /// A help request this survivor accepted and still owes.
    pub help_task: Option<(AgentId, HelpNeed)>,
}

impl Agent {
    pub fn name(&self) -> String {
        format!("{} {}", self.given_name, self.family_name)
    }

    pub fn is_present(&self) -> bool {
        self.life == LifeState::Present
    }

    pub fn is_alive(&self) -> bool {
        !matches!(
            self.life,
            LifeState::Dead { .. } | LifeState::Departed { .. }
        )
    }

    pub fn current_goal(&self) -> Option<Goal> {
        self.intention.as_ref().map(|intention| intention.goal)
    }

    pub fn is_backed_off(&self, key: GoalKey, now: u64) -> bool {
        self.backoff
            .iter()
            .any(|(goal, until)| *goal == key && *until > now)
    }

    pub fn back_off(&mut self, key: GoalKey, until: u64) {
        self.backoff.retain(|(goal, _)| *goal != key);
        self.backoff.push((key, until));
    }

    pub fn has_trait(&self, id: &str) -> bool {
        self.traits.iter().any(|t| t == id)
    }

    pub fn carry_capacity(&self, bonus: f32) -> f32 {
        let base = game_data().balance.agents.carry_capacity + bonus;
        if self.is_child {
            base * 0.4
        } else {
            base
        }
    }

    pub fn carried(&self, resource: Resource) -> f32 {
        match self.carrying {
            Some((carried, amount)) if carried == resource => amount,
            _ => 0.0,
        }
    }

    pub fn ambition_def(&self) -> Option<&'static crate::data::people::AmbitionDef> {
        game_data().ambition(&self.ambition.id)
    }
}
