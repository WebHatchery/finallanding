//! Placed buildings: blueprints under construction and finished structures.

use super::geometry::{AgentId, Footprint, StructureId, Tile};
use crate::data::buildings::BuildingDef;
use crate::data::finds::CropTraits;
use crate::data::{game_data, ResourceBag};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStage {
    Blueprint,
    Built,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CropStage {
    Fallow,
    Growing,
    Ripe,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crop {
    pub stage: CropStage,
    pub growth: f32,
    pub work: f32,
}

impl Default for Crop {
    fn default() -> Self {
        Self {
            stage: CropStage::Fallow,
            growth: 0.0,
            work: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Structure {
    pub id: StructureId,
    pub kind: String,
    pub footprint: Footprint,
    pub stage: BuildStage,
    pub delivered: ResourceBag,
    pub progress: f32,
    pub condition: f32,
    pub crop: Option<Crop>,
    /// The native species a native orchard is growing.
    #[serde(default)]
    pub crop_species: Option<String>,
    pub craft_progress: f32,
    pub craft_recipe: Option<String>,
    pub residents: Vec<AgentId>,
    pub powered: bool,
    pub placed_day: u32,
}

impl Structure {
    pub fn new(id: StructureId, def: &BuildingDef, origin: Tile, day: u32) -> Self {
        Self {
            id,
            kind: def.id.clone(),
            footprint: Footprint::new(origin, def.size),
            stage: BuildStage::Blueprint,
            delivered: ResourceBag::default(),
            progress: 0.0,
            condition: 100.0,
            crop: def.farm.as_ref().map(|_| Crop::default()),
            crop_species: None,
            craft_progress: 0.0,
            craft_recipe: None,
            residents: Vec::new(),
            powered: true,
            placed_day: day,
        }
    }

    /// Yield and growing time of what is planted here.
    pub fn crop_traits(&self) -> CropTraits {
        self.crop_species
            .as_deref()
            .and_then(|id| game_data().find(id))
            .and_then(|find| find.crop)
            .unwrap_or(CropTraits {
                yield_scale: 1.0,
                grow_scale: 1.0,
            })
    }

    pub fn def(&self) -> &'static BuildingDef {
        game_data()
            .building(&self.kind)
            .expect("structures are only created from known building definitions")
    }

    pub fn is_built(&self) -> bool {
        self.stage == BuildStage::Built
    }

    pub fn materials_missing(&self) -> ResourceBag {
        self.delivered.shortfall(&self.def().cost_bag())
    }

    pub fn has_all_materials(&self) -> bool {
        self.materials_missing().is_empty()
    }

    pub fn construction_fraction(&self) -> f32 {
        let work = self.def().work.max(1.0);
        (self.progress / work).clamp(0.0, 1.0)
    }

    /// Whether the building's function is currently effective.
    pub fn is_operational(&self) -> bool {
        self.is_built() && self.condition > 15.0 && (self.powered || !self.def().needs_power())
    }

    pub fn free_beds(&self) -> u32 {
        self.def().beds.saturating_sub(self.residents.len() as u32)
    }

    pub fn needs_repair(&self) -> bool {
        self.is_built() && self.condition < 60.0
    }
}
