//! The job board: work the colony AI has posted for survivors to choose from.
//!
//! Survivors are never assigned jobs. They read the board (colony comms),
//! weigh it against their needs, skills and relationships, and decide.

use crate::data::{NodeKind, ResourceBag};
use crate::world::{AgentId, CreatureId, StructureId, Tile};

#[derive(Clone, Debug, Default)]
pub struct BuildJob {
    pub site: StructureId,
    pub missing: ResourceBag,
    pub ready_to_build: bool,
}

#[derive(Clone, Debug, Default)]
pub struct JobBoard {
    pub builds: Vec<BuildJob>,
    pub repairs: Vec<StructureId>,
    pub farms: Vec<StructureId>,
    pub items: Vec<Tile>,
    /// 0 when meals are plentiful, rising above 1 when they run short.
    pub cook_demand: f32,
    pub kitchens: Vec<StructureId>,
    pub crafts: Vec<StructureId>,
    pub labs: Vec<StructureId>,
    pub research_open: bool,
    pub relic_study: bool,
    pub patients: Vec<AgentId>,
    pub threats: Vec<(CreatureId, Tile, bool)>,
    /// How much the colony wants each kind of landscape resource (0-2).
    pub gather_demand: Vec<(NodeKind, f32)>,
    pub storage_full: bool,
}

impl JobBoard {
    pub fn demand_for(&self, kind: NodeKind) -> f32 {
        self.gather_demand
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, demand)| *demand)
            .unwrap_or(0.0)
    }

    pub fn build_job(&self, site: StructureId) -> Option<&BuildJob> {
        self.builds.iter().find(|job| job.site == site)
    }
}
