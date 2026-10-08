//! Plans: ordered steps a survivor carries out to achieve a goal.

use super::goals::BreakKind;
use super::messages::MessageKind;
use crate::data::Resource;
use crate::world::{AgentId, CreatureId, NodeId, StructureId, Tile};
use serde::{Deserialize, Serialize};

/// Where a movement step is headed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Target {
    Tile(Tile),
    Structure(StructureId),
    Node(NodeId),
    Agent(AgentId),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum FoodSource {
    Meal,
    Raw,
    Carried,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Topic {
    SmallTalk,
    Comfort,
    Flirt,
    Reconcile,
    Insult,
}

/// One primitive action. Steps run over several ticks until they succeed or
/// fail; a failed step fails the whole plan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Step {
    GoTo(Target),
    TakeStock {
        resource: Resource,
        amount: f32,
    },
    TakeMeal,
    Deposit,
    Deliver(StructureId),
    Construct(StructureId),
    Repair(StructureId),
    Harvest(NodeId),
    Forage(NodeId),
    PickUp(Tile),
    Eat(FoodSource),
    Sleep(Option<StructureId>),
    Warm {
        ticks: u32,
    },
    Relax {
        place: Option<StructureId>,
        ticks: u32,
    },
    Tend(StructureId),
    Cook(StructureId),
    Craft(StructureId),
    Research(StructureId),
    StudyRelic(StructureId),
    Treat(AgentId),
    Recover {
        place: Option<StructureId>,
        ticks: u32,
    },
    Talk {
        with: AgentId,
        topic: Topic,
    },
    Send {
        to: AgentId,
        kind: MessageKind,
    },
    WaitForHelp {
        ticks: u32,
    },
    Give(AgentId),
    Attack(CreatureId),
    Hide {
        ticks: u32,
    },
    Scout(Tile),
    JoinExpedition,
    Break {
        kind: BreakKind,
        ticks: u32,
    },
    Wait {
        ticks: u32,
    },
}

/// Identifies a plan in the library so failed plans can be excluded.
pub type PlanId = &'static str;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub steps: Vec<Step>,
    pub cursor: usize,
    /// Ticks spent in the current step.
    pub step_ticks: u32,
}

impl Plan {
    pub fn new(id: PlanId, steps: Vec<Step>) -> Self {
        Self {
            id: id.to_owned(),
            steps,
            cursor: 0,
            step_ticks: 0,
        }
    }

    pub fn current(&self) -> Option<&Step> {
        self.steps.get(self.cursor)
    }

    pub fn advance(&mut self) {
        self.cursor += 1;
        self.step_ticks = 0;
    }

    pub fn is_complete(&self) -> bool {
        self.cursor >= self.steps.len()
    }
}

/// A plan the library proposes, with how much the survivor prefers it.
#[derive(Clone, Debug)]
pub struct PlanOption {
    pub id: PlanId,
    pub steps: Vec<Step>,
    pub preference: f32,
}

/// Why a step failed; used to correct beliefs and choose another plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailReason {
    NoPath,
    TargetGone,
    StockEmpty,
    Depleted,
    Occupied,
    Refused,
    TimedOut,
    Interrupted,
    TooDangerous,
}

impl FailReason {
    pub fn label_key(self) -> &'static str {
        match self {
            FailReason::NoPath => "fail_no_path",
            FailReason::TargetGone => "fail_target_gone",
            FailReason::StockEmpty => "fail_stock_empty",
            FailReason::Depleted => "fail_depleted",
            FailReason::Occupied => "fail_occupied",
            FailReason::Refused => "fail_refused",
            FailReason::TimedOut => "fail_timed_out",
            FailReason::Interrupted => "fail_interrupted",
            FailReason::TooDangerous => "fail_too_dangerous",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    Running,
    Done,
    Failed(FailReason),
}
