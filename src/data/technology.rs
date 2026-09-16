//! technology domain.

use crate::data::mission::MissionItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechId {
    FieldMedicine,
    SurveyScanners,
    ModularHabitats,
    HydroponicPlanning,
    StorageLattice,
    TriageProtocols,
    DroneSurvey,
    NutrientCulture,
    HullRetrofits,
    FabricationJigs,
}

impl TechId {
    pub fn id(&self) -> &'static str {
        match self {
            TechId::FieldMedicine => "field_medicine",
            TechId::SurveyScanners => "survey_scanners",
            TechId::ModularHabitats => "modular_habitats",
            TechId::HydroponicPlanning => "hydroponic_planning",
            TechId::StorageLattice => "storage_lattice",
            TechId::TriageProtocols => "triage_protocols",
            TechId::DroneSurvey => "drone_survey",
            TechId::NutrientCulture => "nutrient_culture",
            TechId::HullRetrofits => "hull_retrofits",
            TechId::FabricationJigs => "fabrication_jigs",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().iter().copied().find(|tech| tech.id() == id)
    }

    pub fn all() -> &'static [TechId] {
        &[
            TechId::FieldMedicine,
            TechId::SurveyScanners,
            TechId::ModularHabitats,
            TechId::HydroponicPlanning,
            TechId::StorageLattice,
            TechId::TriageProtocols,
            TechId::DroneSurvey,
            TechId::NutrientCulture,
            TechId::HullRetrofits,
            TechId::FabricationJigs,
        ]
    }

    pub fn name(&self) -> &'static str {
        &crate::data::config::game_config().technology(*self).name
    }

    pub fn effect_text(&self) -> &'static str {
        &crate::data::config::game_config().technology(*self).effect
    }

    pub fn item_requirements(&self) -> Vec<(MissionItem, u32)> {
        crate::data::config::game_config()
            .technology(*self)
            .items
            .iter()
            .filter_map(|requirement| {
                MissionItem::all()
                    .iter()
                    .copied()
                    .find(|item| item.id() == requirement.id)
                    .map(|item| (item, requirement.count))
            })
            .collect()
    }

    pub fn prerequisite_tech(&self) -> Vec<TechId> {
        crate::data::config::game_config()
            .technology(*self)
            .prerequisites
            .iter()
            .filter_map(|id| TechId::from_id(id))
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TechnologyState {
    pub item_counts: HashMap<MissionItem, u32>,
    pub unlocked: Vec<TechId>,
}

impl TechnologyState {
    pub fn add_item(&mut self, item: MissionItem) -> Vec<TechId> {
        if item.contributes_to_technology() {
            *self.item_counts.entry(item).or_insert(0) += 1;
        }

        self.unlock_available()
    }

    pub fn has(&self, tech_id: TechId) -> bool {
        self.unlocked.contains(&tech_id)
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    pub fn next_locked_tech(&self) -> Option<TechId> {
        TechId::all()
            .iter()
            .copied()
            .find(|tech_id| !self.has(*tech_id))
    }

    pub fn next_research_target(&self) -> Option<TechId> {
        TechId::all()
            .iter()
            .copied()
            .find(|tech_id| !self.has(*tech_id) && self.prerequisites_met(*tech_id))
            .or_else(|| self.next_locked_tech())
    }

    pub fn visible_research_targets(&self, limit: usize) -> Vec<TechId> {
        TechId::all()
            .iter()
            .copied()
            .filter(|tech_id| !self.has(*tech_id))
            .filter(|tech_id| self.prerequisites_met(*tech_id))
            .take(limit)
            .collect()
    }

    pub fn requirement_progress_text(&self, tech_id: TechId) -> String {
        let mut missing = Vec::new();

        for prerequisite in tech_id.prerequisite_tech() {
            if !self.has(prerequisite) {
                missing.push(format!("need {}", prerequisite.name()));
            }
        }

        for (item, required) in tech_id.item_requirements() {
            let count = self.item_count(item);
            if count < required {
                missing.push(format!("{} {}/{}", item.short_name(), count, required));
            }
        }

        if missing.is_empty() {
            "ready to unlock".to_string()
        } else {
            missing.join(" | ")
        }
    }

    pub fn mission_danger_reduction(&self) -> u32 {
        if self.has(TechId::DroneSurvey) {
            18
        } else if self.has(TechId::SurveyScanners) {
            10
        } else {
            0
        }
    }

    pub fn mission_cooldown_reduction(&self) -> u64 {
        if self.has(TechId::DroneSurvey) {
            10
        } else {
            0
        }
    }

    pub fn injury_duration_ticks(&self) -> u64 {
        if self.has(TechId::TriageProtocols) {
            40
        } else if self.has(TechId::FieldMedicine) {
            60
        } else {
            150
        }
    }

    pub fn habitat_capacity_bonus(&self) -> u32 {
        if self.has(TechId::HullRetrofits) {
            2
        } else if self.has(TechId::ModularHabitats) {
            1
        } else {
            0
        }
    }

    pub fn daily_supply_reduction(&self) -> i32 {
        if self.has(TechId::NutrientCulture) {
            2
        } else if self.has(TechId::HydroponicPlanning) {
            1
        } else {
            0
        }
    }

    pub fn storage_capacity_bonus(&self) -> i32 {
        if self.has(TechId::HullRetrofits) {
            35
        } else if self.has(TechId::StorageLattice) {
            20
        } else {
            0
        }
    }

    pub fn salvage_recovery_bonus(&self) -> i32 {
        if self.has(TechId::FabricationJigs) {
            1
        } else {
            0
        }
    }

    fn unlock_available(&mut self) -> Vec<TechId> {
        let mut newly_unlocked = Vec::new();

        for tech_id in TechId::all() {
            if self.has(*tech_id) {
                continue;
            }

            if self.meets_requirements(*tech_id) {
                self.unlocked.push(*tech_id);
                newly_unlocked.push(*tech_id);
            }
        }

        newly_unlocked
    }

    fn meets_requirements(&self, tech_id: TechId) -> bool {
        self.prerequisites_met(tech_id)
            && tech_id
                .item_requirements()
                .iter()
                .all(|(item, required)| self.item_count(*item) >= *required)
    }

    pub fn prerequisites_met(&self, tech_id: TechId) -> bool {
        tech_id
            .prerequisite_tech()
            .iter()
            .all(|prerequisite| self.has(*prerequisite))
    }

    pub fn item_count(&self, item: MissionItem) -> u32 {
        self.item_counts.get(&item).copied().unwrap_or(0)
    }
}
