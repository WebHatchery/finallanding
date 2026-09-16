//! mission domain.

use crate::data::priority::ColonyPriority;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionItem {
    StructuralAlloy,
    AlienCircuit,
    MedicinalGel,
    NutrientPods,
    SalvageCache,
}

impl MissionItem {
    pub fn all() -> &'static [MissionItem] {
        &[
            MissionItem::StructuralAlloy,
            MissionItem::AlienCircuit,
            MissionItem::MedicinalGel,
            MissionItem::NutrientPods,
            MissionItem::SalvageCache,
        ]
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().iter().copied().find(|item| item.id() == id)
    }

    pub fn id(&self) -> &'static str {
        match self {
            MissionItem::StructuralAlloy => "structural_alloy",
            MissionItem::AlienCircuit => "alien_circuit",
            MissionItem::MedicinalGel => "medicinal_gel",
            MissionItem::NutrientPods => "nutrient_pods",
            MissionItem::SalvageCache => "salvage_cache",
        }
    }

    pub fn name(&self) -> &'static str {
        &crate::data::config::game_config().item(*self).name
    }

    pub fn short_name(&self) -> &'static str {
        &crate::data::config::game_config().item(*self).short_name
    }

    pub fn contributes_to_technology(&self) -> bool {
        !matches!(self, MissionItem::SalvageCache)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    SupplyRun,
    PerimeterScan,
    DeepSurvey,
}

impl MissionType {
    pub fn id(&self) -> &'static str {
        match self {
            MissionType::SupplyRun => "supply_run",
            MissionType::PerimeterScan => "perimeter_scan",
            MissionType::DeepSurvey => "deep_survey",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissionDefinition {
    pub mission_type: MissionType,
    pub name: &'static str,
    pub short_name: &'static str,
    pub duration_minutes: u64,
    pub danger_percent: u32,
    pub cooldown_minutes: u64,
    pub description: &'static str,
    pub reward_profile: &'static str,
}

impl MissionType {
    pub fn all() -> &'static [MissionType] {
        &[
            MissionType::SupplyRun,
            MissionType::PerimeterScan,
            MissionType::DeepSurvey,
        ]
    }

    pub fn definition(&self) -> MissionDefinition {
        let config = crate::data::config::game_config().mission(*self);
        MissionDefinition {
            mission_type: *self,
            name: &config.name,
            short_name: &config.short_name,
            duration_minutes: config.duration_minutes,
            danger_percent: config.danger_percent,
            cooldown_minutes: config.cooldown_minutes,
            description: &config.description,
            reward_profile: &config.reward_profile,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveMission {
    pub id: u32,
    pub colonist_id: u32,
    pub mission_type: MissionType,
    pub started_tick: u64,
    pub completes_at_tick: u64,
    pub danger_percent: u32,
    pub priority: ColonyPriority,
}

impl ActiveMission {
    pub fn remaining_ticks(&self, current_tick: u64) -> u64 {
        self.completes_at_tick.saturating_sub(current_tick)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionState {
    pub active_missions: Vec<ActiveMission>,
    pub next_id: u32,
    #[serde(default)]
    pub next_launch_tick: u64,
}

impl Default for MissionState {
    fn default() -> Self {
        Self {
            active_missions: Vec::new(),
            next_id: 1,
            next_launch_tick: 0,
        }
    }
}

impl MissionState {
    pub fn active_count(&self) -> usize {
        self.active_missions.len()
    }

    pub fn cooldown_remaining(&self, current_tick: u64) -> u64 {
        self.next_launch_tick.saturating_sub(current_tick)
    }
}
