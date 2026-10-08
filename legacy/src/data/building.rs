//! building domain.

use crate::data::types::Position;
use serde::{Deserialize, Serialize};

/// The 5 building types for the MVP
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Habitat,         // Sleeping, recovery
    MessHall,        // Eating, social
    Workshop,        // Building/crafting
    Storage,         // Resource buffer
    ExplorationGate, // Sends colonists out
}

impl BuildingType {
    pub fn id(&self) -> &'static str {
        match self {
            BuildingType::Habitat => "habitat",
            BuildingType::MessHall => "mess_hall",
            BuildingType::Workshop => "workshop",
            BuildingType::Storage => "storage",
            BuildingType::ExplorationGate => "exploration_gate",
        }
    }

    /// Returns the size (width, height) in grid cells for this building type
    pub fn size(&self) -> (u32, u32) {
        let config = crate::data::config::game_config().building(*self);
        (config.width, config.height)
    }

    /// Returns the display name for UI
    pub fn name(&self) -> &'static str {
        &crate::data::config::game_config().building(*self).name
    }

    pub fn salvage_cost(&self) -> i32 {
        crate::data::config::game_config()
            .building(*self)
            .salvage_cost
    }

    pub fn planning_role(&self) -> &'static str {
        &crate::data::config::game_config()
            .building(*self)
            .planning_role
    }

    pub fn purpose(&self) -> &'static str {
        &crate::data::config::game_config().building(*self).purpose
    }

    pub fn placement_impact(&self) -> &'static str {
        &crate::data::config::game_config().building(*self).impact
    }

    /// Returns the color for rendering (RGBA as u32)
    pub fn color(&self) -> (u8, u8, u8) {
        let color = crate::data::config::game_config().building(*self).color;
        (color[0], color[1], color[2])
    }

    /// All building types for iteration
    pub fn all() -> &'static [BuildingType] {
        &[
            BuildingType::Habitat,
            BuildingType::MessHall,
            BuildingType::Workshop,
            BuildingType::Storage,
            BuildingType::ExplorationGate,
        ]
    }
}

/// A placed building in the world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub building_type: BuildingType,
    pub position: Position, // Top-left corner of the building
}

impl Building {
    pub fn new(id: u32, building_type: BuildingType, position: Position) -> Self {
        Self {
            id,
            building_type,
            position,
        }
    }

    /// Get the size of this building
    pub fn size(&self) -> (u32, u32) {
        self.building_type.size()
    }

    /// Check if this building occupies a specific grid cell
    pub fn occupies(&self, pos: Position) -> bool {
        let (width, height) = self.size();
        pos.x >= self.position.x
            && pos.x < self.position.x + width as i32
            && pos.y >= self.position.y
            && pos.y < self.position.y + height as i32
    }

    /// Get all grid cells occupied by this building
    pub fn occupied_cells(&self) -> Vec<Position> {
        let (width, height) = self.size();
        let mut cells = Vec::new();
        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                cells.push(Position::new(self.position.x + dx, self.position.y + dy));
            }
        }
        cells
    }
}
