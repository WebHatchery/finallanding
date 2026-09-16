//! Typed, validated game configuration loaded through the shared toolkit.

use crate::data::building::BuildingType;
use crate::data::colonist::{JobPreference, Trait};
use crate::data::mission::{MissionItem, MissionType};
use crate::data::technology::TechId;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Clone, Debug, Deserialize)]
pub struct GameConfig {
    pub grid: GridConfig,
    pub resources: ResourceConfig,
    pub time: TimeConfig,
    pub scenario: ScenarioConfig,
    pub work_thresholds: WorkThresholdConfig,
    pub buildings: Vec<BuildingConfig>,
    pub missions: Vec<MissionConfig>,
    pub mission_items: Vec<MissionItemConfig>,
    pub technology: Vec<TechnologyConfig>,
    pub survivors: Vec<SurvivorConfig>,
    pub starting_relationships: Vec<StartingRelationshipConfig>,
    pub text: GameText,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GridConfig {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResourceConfig {
    pub starting_supplies: i32,
    pub starting_salvage: i32,
    pub base_storage_capacity: i32,
    pub storage_capacity_bonus: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeConfig {
    pub ticks_per_day: u64,
    pub ticks_per_hour: u64,
    pub seconds_per_tick: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScenarioConfig {
    pub target_day: u32,
    pub required_tech_unlocks: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkThresholdConfig {
    pub exploration: u32,
    pub workshop: u32,
    pub kitchen: u32,
    pub hauling: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BuildingConfig {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub width: u32,
    pub height: u32,
    pub salvage_cost: i32,
    pub planning_role: String,
    pub purpose: String,
    pub impact: String,
    pub color: [u8; 3],
}

#[derive(Clone, Debug, Deserialize)]
pub struct MissionConfig {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub duration_minutes: u64,
    pub danger_percent: u32,
    pub cooldown_minutes: u64,
    pub description: String,
    pub reward_profile: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MissionItemConfig {
    pub id: String,
    pub name: String,
    pub short_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TechnologyConfig {
    pub id: String,
    pub name: String,
    pub effect: String,
    pub prerequisites: Vec<String>,
    pub items: Vec<ItemRequirementConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ItemRequirementConfig {
    pub id: String,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SurvivorConfig {
    pub id: u32,
    pub name: String,
    #[serde(rename = "trait")]
    pub trait_data: String,
    pub job: String,
    pub position: [i32; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct StartingRelationshipConfig {
    pub first_id: u32,
    pub second_id: u32,
    pub value: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GameText {
    pub menu_title: String,
    pub menu_premise: Vec<String>,
    pub menu_controls: Vec<String>,
    pub start_game: String,
    pub continue_game: String,
    pub settings: String,
    pub exit: String,
    pub undo: String,
    pub cancel: String,
    pub filter_room: String,
    pub log_search: String,
    pub log_export: String,
    pub restart: String,
    pub back_to_menu: String,
    pub intro_building: String,
    pub intro_assignments: String,
    pub intro_missions: String,
    pub intro_restart: String,
    pub intro_continue: String,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
}

impl GameText {
    pub fn label<'a>(&'a self, key: &'a str) -> &'a str {
        self.labels.get(key).map(String::as_str).unwrap_or(key)
    }

    pub fn fill(&self, key: &str, values: &[String]) -> String {
        let mut text = self.label(key).to_string();
        for value in values {
            text = text.replacen("{}", value, 1);
        }
        text
    }
}

pub fn game_config() -> &'static GameConfig {
    static CONFIG: OnceLock<GameConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let config: GameConfig =
            macroquad_toolkit::include_json!("../../assets/data/game_config.json")
                .expect("embedded Final Landing configuration must be valid JSON");
        config
            .validate()
            .expect("embedded Final Landing configuration must pass validation");
        config
    })
}

impl GameConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_collection_ids()?;
        if self.grid.width < 8 || self.grid.height < 8 || self.grid.cell_size <= 0.0 {
            return Err("grid must be at least 8x8 with a positive cell size".to_string());
        }
        if self.time.ticks_per_day == 0
            || self.time.ticks_per_hour == 0
            || self.time.seconds_per_tick <= 0.0
        {
            return Err("time configuration must use positive values".to_string());
        }
        for building in &self.buildings {
            if building.width == 0 || building.height == 0 || building.salvage_cost < 0 {
                return Err(format!(
                    "building '{}' has invalid size or cost",
                    building.id
                ));
            }
        }
        for mission in &self.missions {
            if mission.duration_minutes == 0 || mission.cooldown_minutes == 0 {
                return Err(format!(
                    "mission '{}' has invalid duration or cooldown",
                    mission.id
                ));
            }
        }
        for survivor in &self.survivors {
            if survivor.position[0] < 0
                || survivor.position[1] < 0
                || survivor.position[0] >= self.grid.width as i32
                || survivor.position[1] >= self.grid.height as i32
            {
                return Err(format!(
                    "survivor '{}' starts outside the grid",
                    survivor.name
                ));
            }
            parse_trait(&survivor.trait_data)?;
            parse_job(&survivor.job)?;
        }
        for technology in &self.technology {
            for prerequisite in &technology.prerequisites {
                parse_tech(prerequisite)?;
            }
            for item in &technology.items {
                if item.count == 0 {
                    return Err(format!(
                        "technology '{}' has a zero item requirement",
                        technology.id
                    ));
                }
                parse_item(&item.id)?;
            }
        }
        Ok(())
    }

    fn validate_collection_ids(&self) -> Result<(), String> {
        validate_ids(
            self.buildings.iter().map(|entry| entry.id.as_str()),
            "building",
        )?;
        validate_ids(
            self.missions.iter().map(|entry| entry.id.as_str()),
            "mission",
        )?;
        validate_ids(
            self.mission_items.iter().map(|entry| entry.id.as_str()),
            "mission item",
        )?;
        validate_ids(
            self.technology.iter().map(|entry| entry.id.as_str()),
            "technology",
        )
    }

    pub fn building(&self, building_type: BuildingType) -> &BuildingConfig {
        let id = building_type.id();
        self.buildings
            .iter()
            .find(|entry| entry.id == id)
            .expect("validated building definition must exist")
    }

    pub fn mission(&self, mission_type: MissionType) -> &MissionConfig {
        self.missions
            .iter()
            .find(|entry| entry.id == mission_type.id())
            .expect("validated mission definition must exist")
    }

    pub fn item(&self, item: MissionItem) -> &MissionItemConfig {
        self.mission_items
            .iter()
            .find(|entry| entry.id == item.id())
            .expect("validated mission item definition must exist")
    }

    pub fn technology(&self, tech_id: TechId) -> &TechnologyConfig {
        self.technology
            .iter()
            .find(|entry| entry.id == tech_id.id())
            .expect("validated technology definition must exist")
    }
}

fn validate_ids<'a>(ids: impl Iterator<Item = &'a str>, kind: &str) -> Result<(), String> {
    let mut seen = Vec::new();
    for id in ids {
        if id.is_empty() || seen.iter().any(|known| known == &id) {
            return Err(format!("duplicate or empty {kind} id '{id}'"));
        }
        seen.push(id);
    }
    Ok(())
}

fn parse_item(id: &str) -> Result<MissionItem, String> {
    MissionItem::from_id(id).ok_or_else(|| format!("unknown mission item id '{id}'"))
}

fn parse_tech(id: &str) -> Result<TechId, String> {
    TechId::from_id(id).ok_or_else(|| format!("unknown technology id '{id}'"))
}

fn parse_trait(id: &str) -> Result<Trait, String> {
    Trait::from_id(id).ok_or_else(|| format!("unknown survivor trait id '{id}'"))
}

fn parse_job(id: &str) -> Result<JobPreference, String> {
    JobPreference::from_id(id).ok_or_else(|| format!("unknown survivor job id '{id}'"))
}
