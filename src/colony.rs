//! Colony-wide state: the stockpile, priorities, policies, research, expeditions,
//! campaign progress and the chronicle.

pub mod campaign;
pub mod chronicle;
pub mod expeditions;
pub mod jobs;
pub mod research;
pub mod tech_tree;

use crate::agents::goals::{WorkType, WORK_TYPE_COUNT};
use crate::data::society::PolicyEffects;
use crate::data::{game_data, ResourceBag};
use campaign::CampaignState;
use chronicle::Chronicle;
use expeditions::Expeditions;
use jobs::JobBoard;
use research::ResearchState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tech_tree::{Modifiers, TechTree};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PowerBudget {
    pub generated: f32,
    /// Generation at full sun with fuel: what the grid can supply at best.
    pub capacity: f32,
    pub demand: f32,
}

impl PowerBudget {
    pub fn surplus(&self) -> f32 {
        self.generated - self.demand
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ColonyStats {
    pub deaths: u32,
    pub births: u32,
    pub arrivals: u32,
    pub departures: u32,
    pub relics_found: f32,
    pub eurekas: u32,
    pub structures_built: u32,
    pub peak_population: usize,
    pub creatures_repelled: u32,
}

/// The run's fixed setup, chosen on the new-colony screen.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunSetup {
    pub colony_name: String,
    pub site: String,
    pub difficulty: String,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Colony {
    pub setup: RunSetup,
    pub stock: ResourceBag,
    pub priorities: [u8; WORK_TYPE_COUNT],
    pub policies: BTreeMap<String, String>,
    pub power: PowerBudget,
    pub research: ResearchState,
    pub tree: TechTree,
    pub expeditions: Expeditions,
    pub campaign: CampaignState,
    pub chronicle: Chronicle,
    pub stats: ColonyStats,
    pub event_cooldowns: BTreeMap<String, u32>,
    pub events_fired: Vec<String>,
    pub next_event_day: u32,
    #[serde(skip)]
    pub jobs: JobBoard,
    #[serde(skip)]
    pub modifiers: Modifiers,
}

impl Colony {
    pub fn new(setup: RunSetup, tree: TechTree) -> Self {
        let data = game_data();
        let policies = data
            .society
            .policies
            .iter()
            .map(|p| (p.id.clone(), p.default_option.clone()))
            .collect();
        Self {
            setup,
            stock: ResourceBag::default(),
            priorities: [2; WORK_TYPE_COUNT],
            policies,
            power: PowerBudget::default(),
            research: ResearchState::default(),
            tree,
            expeditions: Expeditions::default(),
            campaign: CampaignState::default(),
            chronicle: Chronicle::default(),
            stats: ColonyStats::default(),
            event_cooldowns: BTreeMap::new(),
            events_fired: Vec::new(),
            next_event_day: 4,
            jobs: JobBoard::default(),
            modifiers: Modifiers::default(),
        }
    }

    /// Rebuild derived state after loading a save.
    pub fn refresh_derived(&mut self) {
        self.modifiers = Modifiers::from_tree(&self.tree);
    }

    pub fn priority(&self, work: WorkType) -> u8 {
        self.priorities[work.index()]
    }

    /// Effects of every active policy option, combined.
    pub fn policy_effects(&self) -> Vec<&'static PolicyEffects> {
        let data = game_data();
        self.policies
            .iter()
            .filter_map(|(policy, option)| {
                data.policy(policy)?
                    .options
                    .iter()
                    .find(|o| &o.id == option)
                    .map(|o| &o.effects)
            })
            .collect()
    }

    pub fn food_use(&self) -> f32 {
        self.policy_effects().iter().map(|e| e.food_use).product()
    }

    pub fn work_hours_shift(&self) -> f32 {
        self.policy_effects().iter().map(|e| e.work_hours).sum()
    }

    pub fn curfew(&self) -> bool {
        self.policy_effects().iter().any(|e| e.curfew)
    }

    pub fn child_chance_scale(&self) -> f32 {
        self.policy_effects()
            .iter()
            .map(|e| e.child_chance)
            .product()
    }

    pub fn volunteer_bias(&self) -> f32 {
        self.policy_effects()
            .iter()
            .map(|e| e.volunteer_bias)
            .product()
    }

    pub fn difficulty_value(
        &self,
        pick: impl Fn(&crate::data::campaign::DifficultyDef) -> f32,
    ) -> f32 {
        game_data()
            .difficulty(&self.setup.difficulty)
            .map(pick)
            .unwrap_or(1.0)
    }

    pub fn is_tech_researched(&self, tech: &Option<String>) -> bool {
        tech.as_ref().is_none_or(|id| self.tree.is_researched(id))
    }
}
