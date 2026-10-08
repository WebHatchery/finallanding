//! Building and crafting recipe definitions loaded from JSON.

use super::kinds::{Branch, BuildingCategory, Skill};
use super::resources::{Resource, ResourceBag};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FarmSpec {
    pub yield_food: f32,
    pub grow_days: f32,
    #[serde(default)]
    pub indoor: bool,
    /// Grows whichever native food the colony has gathered most, with that
    /// species' yield and growing time.
    #[serde(default)]
    pub native: bool,
}

/// Static description of a building type. Behaviour is selected by the
/// optional function fields; most buildings use only one or two.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: BuildingCategory,
    pub size: [i32; 2],
    pub cost: BTreeMap<Resource, f32>,
    pub work: f32,
    pub color: [u8; 3],
    #[serde(default)]
    pub tech: Option<String>,
    #[serde(default)]
    pub beds: u32,
    #[serde(default)]
    pub comfort: f32,
    /// Beds reserved for partners and their children.
    #[serde(default)]
    pub family_only: bool,
    #[serde(default)]
    pub storage: f32,
    #[serde(default)]
    pub indoor: bool,
    #[serde(default)]
    pub heat: f32,
    #[serde(default)]
    pub kitchen: bool,
    #[serde(default)]
    pub seats: u32,
    #[serde(default)]
    pub farm: Option<FarmSpec>,
    #[serde(default)]
    pub research: f32,
    #[serde(default)]
    pub research_branch: Option<Branch>,
    #[serde(default)]
    pub station: Option<String>,
    #[serde(default)]
    pub care_beds: u32,
    #[serde(default)]
    pub power: f32,
    #[serde(default)]
    pub solar: bool,
    #[serde(default)]
    pub fuel_per_day: f32,
    #[serde(default)]
    pub recreation: f32,
    #[serde(default)]
    pub beauty: f32,
    #[serde(default)]
    pub defence: f32,
    #[serde(default)]
    pub defence_range: f32,
    #[serde(default)]
    pub walkable: bool,
    #[serde(default)]
    pub expedition: bool,
    #[serde(default)]
    pub capstone: Option<String>,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub buildable: Option<bool>,
    #[serde(default)]
    pub sprite: Option<usize>,
}

impl BuildingDef {
    pub fn cost_bag(&self) -> ResourceBag {
        ResourceBag::from_map(&self.cost)
    }

    pub fn needs_power(&self) -> bool {
        self.power < 0.0
    }

    pub fn is_player_buildable(&self) -> bool {
        self.buildable.unwrap_or(true)
    }

    pub fn area(&self) -> i32 {
        self.size[0] * self.size[1]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecipeDef {
    pub id: String,
    pub name: String,
    pub station: String,
    pub inputs: BTreeMap<Resource, f32>,
    pub outputs: BTreeMap<Resource, f32>,
    pub work: f32,
    pub skill: Skill,
    #[serde(default)]
    pub tech: Option<String>,
    /// Keep producing while the stock of the main output is below this.
    pub target_stock: f32,
}

impl RecipeDef {
    pub fn input_bag(&self) -> ResourceBag {
        ResourceBag::from_map(&self.inputs)
    }

    pub fn output_bag(&self) -> ResourceBag {
        ResourceBag::from_map(&self.outputs)
    }

    pub fn main_output(&self) -> Option<Resource> {
        self.outputs
            .iter()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(resource, _)| *resource)
    }
}
