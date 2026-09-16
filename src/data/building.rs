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
    /// Returns the size (width, height) in grid cells for this building type
    pub fn size(&self) -> (u32, u32) {
        match self {
            BuildingType::Habitat => (2, 2),
            BuildingType::MessHall => (3, 2),
            BuildingType::Workshop => (2, 3),
            BuildingType::Storage => (2, 2),
            BuildingType::ExplorationGate => (2, 2),
        }
    }

    /// Returns the display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            BuildingType::Habitat => "Habitat",
            BuildingType::MessHall => "Mess Hall",
            BuildingType::Workshop => "Workshop",
            BuildingType::Storage => "Storage",
            BuildingType::ExplorationGate => "Exploration Gate",
        }
    }

    pub fn salvage_cost(&self) -> i32 {
        match self {
            BuildingType::Habitat => 8,
            BuildingType::MessHall => 12,
            BuildingType::Workshop => 10,
            BuildingType::Storage => 6,
            BuildingType::ExplorationGate => 14,
        }
    }

    pub fn planning_role(&self) -> &'static str {
        match self {
            BuildingType::Habitat => "Recovery",
            BuildingType::MessHall => "Food",
            BuildingType::Workshop => "Salvage",
            BuildingType::Storage => "Storage",
            BuildingType::ExplorationGate => "Exploration",
        }
    }

    pub fn purpose(&self) -> &'static str {
        match self {
            BuildingType::Habitat => "Beds and recovery space for tired or hurt colonists.",
            BuildingType::MessHall => "Meal work that lowers the next daily supply draw.",
            BuildingType::Workshop => "Builder work that converts wreckage into salvage.",
            BuildingType::Storage => "Raises the supply cap and organizes hauling work.",
            BuildingType::ExplorationGate => {
                "Launches scans for tech items and emergency resources."
            }
        }
    }

    pub fn placement_impact(&self) -> &'static str {
        match self {
            BuildingType::Habitat => "Adds 2 recovery beds before technology bonuses.",
            BuildingType::MessHall => {
                "Lets cooks prepare meals and creates meal-time social contact."
            }
            BuildingType::Workshop => "Lets builders recover salvage from the crash site.",
            BuildingType::Storage => "Adds supply capacity and lets haulers recover salvage.",
            BuildingType::ExplorationGate => {
                "Enables perimeter scans for resources, injuries, and technology items."
            }
        }
    }

    /// Returns the color for rendering (RGBA as u32)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            BuildingType::Habitat => (100, 149, 237), // Cornflower blue
            BuildingType::MessHall => (255, 165, 0),  // Orange
            BuildingType::Workshop => (139, 69, 19),  // Saddle brown
            BuildingType::Storage => (128, 128, 128), // Gray
            BuildingType::ExplorationGate => (147, 112, 219), // Medium purple
        }
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
