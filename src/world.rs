//! The physical world: terrain, structures, landscape features, fauna and weather.

pub mod calendar;
pub mod features;
pub mod geometry;
pub mod mapgen;
pub mod structures;
pub mod terrain;
pub mod weather;

pub use calendar::Calendar;
pub use features::{Creature, CreatureMood, GroundItem, ResourceNode};
pub use geometry::{AgentId, CreatureId, Footprint, NodeId, Point, StructureId, Tile};
pub use structures::{BuildStage, Crop, CropStage, Structure};
pub use terrain::{Terrain, WorldMap};
pub use weather::WeatherState;

use crate::data::buildings::BuildingDef;
use crate::data::{Resource, ResourceBag};
use macroquad_toolkit::pathfinding::{find_path_with, Heuristic, Pos};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Why a blueprint cannot be placed at a location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementIssue {
    OutOfBounds,
    Unexplored,
    BlockedTerrain,
    Occupied,
    UniqueExists,
}

impl PlacementIssue {
    pub fn label_key(self) -> &'static str {
        match self {
            PlacementIssue::OutOfBounds => "place_out_of_bounds",
            PlacementIssue::Unexplored => "place_unexplored",
            PlacementIssue::BlockedTerrain => "place_blocked_terrain",
            PlacementIssue::Occupied => "place_occupied",
            PlacementIssue::UniqueExists => "place_unique",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub map: WorldMap,
    pub structures: Vec<Structure>,
    pub nodes: Vec<ResourceNode>,
    pub items: Vec<GroundItem>,
    pub creatures: Vec<Creature>,
    pub weather: WeatherState,
    pub next_structure_id: StructureId,
    pub next_node_id: NodeId,
    pub next_creature_id: u32,
    pub landing_tile: Tile,
    /// The native species of this landing, drawn per run. Expeditions can
    /// bring samples of others home.
    #[serde(default)]
    pub species: Vec<String>,
}

impl World {
    pub fn structure(&self, id: StructureId) -> Option<&Structure> {
        self.structures.iter().find(|s| s.id == id)
    }

    pub fn structure_mut(&mut self, id: StructureId) -> Option<&mut Structure> {
        self.structures.iter_mut().find(|s| s.id == id)
    }

    pub fn node(&self, id: NodeId) -> Option<&ResourceNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut ResourceNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn built(&self) -> impl Iterator<Item = &Structure> {
        self.structures.iter().filter(|s| s.is_built())
    }

    pub fn count_built(&self, kind: &str) -> usize {
        self.built().filter(|s| s.kind == kind).count()
    }

    pub fn check_placement(&self, def: &BuildingDef, origin: Tile) -> Result<(), PlacementIssue> {
        if def.unique && self.structures.iter().any(|s| s.kind == def.id) {
            return Err(PlacementIssue::UniqueExists);
        }
        let footprint = Footprint::new(origin, def.size);
        for tile in footprint.tiles() {
            let Some(index) = self.map.index(tile) else {
                return Err(PlacementIssue::OutOfBounds);
            };
            if !self.map.explored[index] {
                return Err(PlacementIssue::Unexplored);
            }
            if !self.map.terrain[index].is_buildable() {
                return Err(PlacementIssue::BlockedTerrain);
            }
            if self.map.structure_at[index].is_some() || self.map.node_at[index].is_some() {
                return Err(PlacementIssue::Occupied);
            }
        }
        Ok(())
    }

    /// Place a blueprint (or an instantly built structure) and claim its tiles.
    pub fn add_structure(
        &mut self,
        def: &BuildingDef,
        origin: Tile,
        day: u32,
        built: bool,
    ) -> StructureId {
        let id = self.next_structure_id;
        self.next_structure_id += 1;
        let mut structure = Structure::new(id, def, origin, day);
        if built {
            structure.stage = BuildStage::Built;
            structure.progress = def.work;
            structure.delivered = def.cost_bag();
        }
        for tile in structure.footprint.tiles() {
            if let Some(index) = self.map.index(tile) {
                self.map.structure_at[index] = Some(id);
                self.map.blocked[index] = built && !def.walkable;
            }
        }
        self.structures.push(structure);
        id
    }

    /// Finish construction: the footprint now blocks movement unless walkable.
    pub fn complete_structure(&mut self, id: StructureId) {
        let Some(structure) = self.structure_mut(id) else {
            return;
        };
        structure.stage = BuildStage::Built;
        let walkable = structure.def().walkable;
        let tiles: Vec<Tile> = structure.footprint.tiles().collect();
        for tile in tiles {
            self.map.set_blocked(tile, !walkable);
        }
    }

    /// Remove a structure, returning the materials already delivered to it.
    pub fn remove_structure(&mut self, id: StructureId) -> Option<ResourceBag> {
        let position = self.structures.iter().position(|s| s.id == id)?;
        let structure = self.structures.remove(position);
        for tile in structure.footprint.tiles() {
            if let Some(index) = self.map.index(tile) {
                self.map.structure_at[index] = None;
                self.map.blocked[index] = false;
            }
        }
        Some(structure.delivered)
    }

    pub fn add_node(&mut self, node: ResourceNode) {
        if let Some(index) = self.map.index(node.tile) {
            self.map.node_at[index] = Some(node.id);
            self.map.blocked[index] = node.blocks_movement();
        }
        self.nodes.push(node);
    }

    pub fn allocate_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    /// Remove a spent, non-regrowing node and free its tile.
    pub fn remove_node(&mut self, id: NodeId) {
        if let Some(position) = self.nodes.iter().position(|n| n.id == id) {
            let node = self.nodes.remove(position);
            if let Some(index) = self.map.index(node.tile) {
                self.map.node_at[index] = None;
                self.map.blocked[index] = false;
            }
        }
    }

    pub fn drop_item(&mut self, tile: Tile, resource: Resource, amount: f32) {
        if amount <= 0.01 {
            return;
        }
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.tile == tile && item.resource == resource)
        {
            item.amount += amount;
            return;
        }
        self.items.push(GroundItem {
            tile,
            resource,
            amount,
        });
    }

    /// Total storage capacity of built storage structures.
    pub fn storage_capacity(&self) -> f32 {
        self.built()
            .filter(|s| s.is_operational() || !s.def().needs_power())
            .map(|s| s.def().storage)
            .sum()
    }

    /// A walkable tile beside the structure, nearest to `from`.
    pub fn access_tile(&self, id: StructureId, from: Tile) -> Option<Tile> {
        let structure = self.structure(id)?;
        if structure.def().walkable && structure.is_built() {
            let center = structure.footprint.origin;
            return structure
                .footprint
                .tiles()
                .min_by_key(|tile| tile.steps(from))
                .or(Some(center));
        }
        structure
            .footprint
            .ring()
            .into_iter()
            .filter(|tile| self.map.is_walkable(*tile))
            .min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)))
    }

    /// A walkable tile beside a node, nearest to `from`.
    pub fn node_access_tile(&self, id: NodeId, from: Tile) -> Option<Tile> {
        let node = self.node(id)?;
        if !node.blocks_movement() {
            return Some(node.tile);
        }
        node.tile
            .neighbours()
            .into_iter()
            .filter(|tile| self.map.is_walkable(*tile))
            .min_by(|a, b| a.distance(from).total_cmp(&b.distance(from)))
    }

    /// Shortest walking route, excluding the start tile.
    pub fn find_path(&self, from: Tile, to: Tile) -> Option<VecDeque<Tile>> {
        if from == to {
            return Some(VecDeque::new());
        }
        let map = &self.map;
        let path = find_path_with(
            Pos::new(from.x, from.y),
            Pos::new(to.x, to.y),
            map.width as usize,
            map.height as usize,
            |pos| {
                let tile = Tile::new(pos.x, pos.y);
                tile == from || map.is_walkable(tile)
            },
            |pos| map.move_cost(Tile::new(pos.x, pos.y)),
            Heuristic::Euclidean,
            true,
        )?;
        Some(
            path.waypoints
                .into_iter()
                .skip(1)
                .map(|pos| Tile::new(pos.x, pos.y))
                .collect(),
        )
    }

    /// Structure whose footprint covers the tile, if any.
    pub fn structure_covering(&self, tile: Tile) -> Option<&Structure> {
        self.map
            .structure_on(tile)
            .and_then(|id| self.structure(id))
    }

    /// Strength of heat reaching a tile from fires, burners and heated buildings.
    pub fn heat_at(&self, tile: Tile) -> f32 {
        self.built()
            .filter(|s| s.def().heat > 0.0 && s.is_operational())
            .map(|s| {
                let distance = s.footprint.distance_to(tile);
                (1.0 - distance / s.def().heat.max(0.5)).max(0.0)
            })
            .fold(0.0, f32::max)
    }
}
