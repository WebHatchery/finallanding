//! Goals (desires) a survivor can adopt, and their identity for backoff.

use crate::data::{NodeKind, Resource};
use crate::world::{AgentId, CreatureId, StructureId};
use serde::{Deserialize, Serialize};

/// What a survivor asks another for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpNeed {
    Food,
    Care,
    Defence(CreatureId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakKind {
    Wander,
    Sulk,
    Binge,
    LashOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Goal {
    Eat,
    Sleep,
    WarmUp,
    Flee,
    SeekCare,
    Relax,
    Build(StructureId),
    Repair(StructureId),
    Haul,
    Farm(StructureId),
    Gather(NodeKind),
    Cook,
    Craft(StructureId),
    Research,
    Treat(AgentId),
    Defend(CreatureId),
    Scout,
    Chat(AgentId),
    Comfort(AgentId),
    Court(AgentId),
    Reconcile(AgentId),
    Help { requester: AgentId, need: HelpNeed },
    Volunteer,
    Break(BreakKind),
    Idle,
}

/// Broad grouping used for the player's work priorities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkType {
    Build,
    Farm,
    Gather,
    Haul,
    Cook,
    Craft,
    Research,
    Medicine,
}

pub const WORK_TYPE_COUNT: usize = 8;

impl WorkType {
    pub const ALL: [WorkType; WORK_TYPE_COUNT] = [
        WorkType::Build,
        WorkType::Farm,
        WorkType::Gather,
        WorkType::Haul,
        WorkType::Cook,
        WorkType::Craft,
        WorkType::Research,
        WorkType::Medicine,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label_key(self) -> &'static str {
        match self {
            WorkType::Build => "work_build",
            WorkType::Farm => "work_farm",
            WorkType::Gather => "work_gather",
            WorkType::Haul => "work_haul",
            WorkType::Cook => "work_cook",
            WorkType::Craft => "work_craft",
            WorkType::Research => "work_research",
            WorkType::Medicine => "work_medicine",
        }
    }
}

/// A stable identity for a goal, used to back off after repeated failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GoalKey {
    pub kind: u8,
    pub target: u32,
}

impl Goal {
    pub fn key(&self) -> GoalKey {
        let (kind, target) = match *self {
            Goal::Eat => (0, 0),
            Goal::Sleep => (1, 0),
            Goal::WarmUp => (2, 0),
            Goal::Flee => (3, 0),
            Goal::SeekCare => (4, 0),
            Goal::Relax => (5, 0),
            Goal::Build(id) => (6, id),
            Goal::Repair(id) => (7, id),
            Goal::Haul => (8, 0),
            Goal::Farm(id) => (9, id),
            Goal::Gather(kind) => (10, kind as u32),
            Goal::Cook => (11, 0),
            Goal::Craft(id) => (12, id),
            Goal::Research => (13, 0),
            Goal::Treat(id) => (14, id),
            Goal::Defend(id) => (15, id),
            Goal::Scout => (16, 0),
            Goal::Chat(id) => (17, id),
            Goal::Comfort(id) => (18, id),
            Goal::Court(id) => (19, id),
            Goal::Reconcile(id) => (20, id),
            Goal::Help { requester, .. } => (21, requester),
            Goal::Volunteer => (22, 0),
            Goal::Break(kind) => (23, kind as u32),
            Goal::Idle => (24, 0),
        };
        GoalKey { kind, target }
    }

    pub fn work_type(&self) -> Option<WorkType> {
        match self {
            Goal::Build(_) | Goal::Repair(_) => Some(WorkType::Build),
            Goal::Farm(_) => Some(WorkType::Farm),
            Goal::Gather(_) | Goal::Scout => Some(WorkType::Gather),
            Goal::Haul => Some(WorkType::Haul),
            Goal::Cook => Some(WorkType::Cook),
            Goal::Craft(_) => Some(WorkType::Craft),
            Goal::Research => Some(WorkType::Research),
            Goal::Treat(_) => Some(WorkType::Medicine),
            _ => None,
        }
    }

    pub fn is_survival(&self) -> bool {
        matches!(
            self,
            Goal::Eat | Goal::Sleep | Goal::WarmUp | Goal::Flee | Goal::SeekCare
        )
    }

    pub fn is_social(&self) -> bool {
        matches!(
            self,
            Goal::Chat(_) | Goal::Comfort(_) | Goal::Court(_) | Goal::Reconcile(_)
        )
    }

    /// The other survivor this goal concerns, if any.
    pub fn partner(&self) -> Option<AgentId> {
        match *self {
            Goal::Chat(id)
            | Goal::Comfort(id)
            | Goal::Court(id)
            | Goal::Reconcile(id)
            | Goal::Treat(id) => Some(id),
            Goal::Help { requester, .. } => Some(requester),
            _ => None,
        }
    }

    /// Label key for the inspector's "Goal" line.
    pub fn label_key(&self) -> &'static str {
        match self {
            Goal::Eat => "goal_eat",
            Goal::Sleep => "goal_sleep",
            Goal::WarmUp => "goal_warm_up",
            Goal::Flee => "goal_flee",
            Goal::SeekCare => "goal_seek_care",
            Goal::Relax => "goal_relax",
            Goal::Build(_) => "goal_build",
            Goal::Repair(_) => "goal_repair",
            Goal::Haul => "goal_haul",
            Goal::Farm(_) => "goal_farm",
            Goal::Gather(_) => "goal_gather",
            Goal::Cook => "goal_cook",
            Goal::Craft(_) => "goal_craft",
            Goal::Research => "goal_research",
            Goal::Treat(_) => "goal_treat",
            Goal::Defend(_) => "goal_defend",
            Goal::Scout => "goal_scout",
            Goal::Chat(_) => "goal_chat",
            Goal::Comfort(_) => "goal_comfort",
            Goal::Court(_) => "goal_court",
            Goal::Reconcile(_) => "goal_reconcile",
            Goal::Help { .. } => "goal_help",
            Goal::Volunteer => "goal_volunteer",
            Goal::Break(_) => "goal_break",
            Goal::Idle => "goal_idle",
        }
    }
}

/// The resource a gather goal produces.
pub fn gathered_resource(kind: NodeKind) -> Resource {
    match kind {
        NodeKind::Wreckage => Resource::Salvage,
        NodeKind::FibreGrove => Resource::Fibre,
        NodeKind::StoneOutcrop => Resource::Stone,
        NodeKind::OreVein => Resource::Metal,
        NodeKind::Glowfruit => Resource::Food,
        NodeKind::Ruin => Resource::Relics,
    }
}
