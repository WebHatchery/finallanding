//! The tile map: terrain, fertility, exploration and occupancy.

use super::geometry::{NodeId, StructureId, Tile};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    Grass,
    Soil,
    Sand,
    Rock,
    Water,
    Snow,
    Ash,
    Scorch,
}

impl Terrain {
    pub fn is_walkable(self) -> bool {
        !matches!(self, Terrain::Rock | Terrain::Water)
    }

    pub fn is_buildable(self) -> bool {
        self.is_walkable()
    }

    /// Path cost multiplier; soft ground slows walking a little.
    pub fn move_cost(self) -> f32 {
        match self {
            Terrain::Sand | Terrain::Snow => 1.4,
            Terrain::Ash => 1.2,
            _ => 1.0,
        }
    }

    pub fn from_key(key: &str) -> Terrain {
        match key {
            "snow" => Terrain::Snow,
            "ash" => Terrain::Ash,
            "sand" => Terrain::Sand,
            "soil" => Terrain::Soil,
            _ => Terrain::Grass,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldMap {
    pub width: i32,
    pub height: i32,
    pub terrain: Vec<Terrain>,
    /// 0-100 soil fertility, used by outdoor farms.
    pub fertility: Vec<u8>,
    /// Tiles any survivor has seen. The player's view of the world.
    pub explored: Vec<bool>,
    pub structure_at: Vec<Option<StructureId>>,
    pub node_at: Vec<Option<NodeId>>,
    /// Structures and nodes that block movement, kept in sync with occupancy.
    pub blocked: Vec<bool>,
}

impl WorldMap {
    pub fn new(width: i32, height: i32, ground: Terrain) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            terrain: vec![ground; size],
            fertility: vec![30; size],
            explored: vec![false; size],
            structure_at: vec![None; size],
            node_at: vec![None; size],
            blocked: vec![false; size],
        }
    }

    pub fn in_bounds(&self, tile: Tile) -> bool {
        tile.x >= 0 && tile.y >= 0 && tile.x < self.width && tile.y < self.height
    }

    pub fn index(&self, tile: Tile) -> Option<usize> {
        self.in_bounds(tile)
            .then(|| (tile.y * self.width + tile.x) as usize)
    }

    pub fn terrain_at(&self, tile: Tile) -> Option<Terrain> {
        self.index(tile).map(|index| self.terrain[index])
    }

    pub fn set_terrain(&mut self, tile: Tile, terrain: Terrain) {
        if let Some(index) = self.index(tile) {
            self.terrain[index] = terrain;
        }
    }

    pub fn fertility_at(&self, tile: Tile) -> f32 {
        self.index(tile)
            .map(|index| self.fertility[index] as f32)
            .unwrap_or(0.0)
    }

    pub fn is_walkable(&self, tile: Tile) -> bool {
        self.index(tile)
            .is_some_and(|index| self.terrain[index].is_walkable() && !self.blocked[index])
    }

    pub fn move_cost(&self, tile: Tile) -> f32 {
        self.terrain_at(tile).map(Terrain::move_cost).unwrap_or(1.0)
    }

    pub fn is_explored(&self, tile: Tile) -> bool {
        self.index(tile).is_some_and(|index| self.explored[index])
    }

    pub fn reveal_circle(&mut self, center: Tile, radius: f32) {
        let reach = radius.ceil() as i32;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let tile = center.offset(dx, dy);
                if center.distance(tile) <= radius {
                    if let Some(index) = self.index(tile) {
                        self.explored[index] = true;
                    }
                }
            }
        }
    }

    pub fn explored_fraction(&self) -> f32 {
        let seen = self.explored.iter().filter(|seen| **seen).count();
        seen as f32 / self.explored.len().max(1) as f32
    }

    pub fn structure_on(&self, tile: Tile) -> Option<StructureId> {
        self.index(tile).and_then(|index| self.structure_at[index])
    }

    pub fn node_on(&self, tile: Tile) -> Option<NodeId> {
        self.index(tile).and_then(|index| self.node_at[index])
    }

    pub fn set_blocked(&mut self, tile: Tile, blocked: bool) {
        if let Some(index) = self.index(tile) {
            self.blocked[index] = blocked;
        }
    }

    /// The walkable tile nearest to `tile` within `radius`, preferring `tile`.
    pub fn nearest_walkable(&self, tile: Tile, radius: i32) -> Option<Tile> {
        if self.is_walkable(tile) {
            return Some(tile);
        }
        let mut best: Option<(i32, Tile)> = None;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let candidate = tile.offset(dx, dy);
                if !self.is_walkable(candidate) {
                    continue;
                }
                let steps = tile.steps(candidate);
                if best.is_none_or(|(best_steps, _)| steps < best_steps) {
                    best = Some((steps, candidate));
                }
            }
        }
        best.map(|(_, found)| found)
    }
}
