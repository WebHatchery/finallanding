//! Grid coordinates and small spatial helpers.

use serde::{Deserialize, Serialize};

pub type AgentId = u32;
pub type StructureId = u32;
pub type NodeId = u32;
pub type CreatureId = u32;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
}

impl Tile {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance(self, other: Tile) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Chebyshev distance: the number of king moves between tiles.
    pub fn steps(self, other: Tile) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    pub fn offset(self, dx: i32, dy: i32) -> Tile {
        Tile::new(self.x + dx, self.y + dy)
    }

    pub fn neighbours(self) -> [Tile; 8] {
        [
            self.offset(1, 0),
            self.offset(-1, 0),
            self.offset(0, 1),
            self.offset(0, -1),
            self.offset(1, 1),
            self.offset(-1, -1),
            self.offset(1, -1),
            self.offset(-1, 1),
        ]
    }
}

/// A smooth position in tile units, used for rendering movement between tiles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn of_tile(tile: Tile) -> Self {
        Self {
            x: tile.x as f32,
            y: tile.y as f32,
        }
    }

    pub fn lerp(self, other: Point, t: f32) -> Point {
        Point {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

/// An axis-aligned footprint of tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    pub origin: Tile,
    pub width: i32,
    pub height: i32,
}

impl Footprint {
    pub fn new(origin: Tile, size: [i32; 2]) -> Self {
        Self {
            origin,
            width: size[0],
            height: size[1],
        }
    }

    pub fn contains(&self, tile: Tile) -> bool {
        tile.x >= self.origin.x
            && tile.y >= self.origin.y
            && tile.x < self.origin.x + self.width
            && tile.y < self.origin.y + self.height
    }

    pub fn tiles(&self) -> impl Iterator<Item = Tile> + '_ {
        (0..self.height)
            .flat_map(move |dy| (0..self.width).map(move |dx| self.origin.offset(dx, dy)))
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.origin.x as f32 + (self.width as f32 - 1.0) * 0.5,
            y: self.origin.y as f32 + (self.height as f32 - 1.0) * 0.5,
        }
    }

    /// Tiles touching the footprint's edge from outside, including corners.
    pub fn ring(&self) -> Vec<Tile> {
        let mut ring = Vec::new();
        for x in -1..=self.width {
            ring.push(self.origin.offset(x, -1));
            ring.push(self.origin.offset(x, self.height));
        }
        for y in 0..self.height {
            ring.push(self.origin.offset(-1, y));
            ring.push(self.origin.offset(self.width, y));
        }
        ring
    }

    pub fn distance_to(&self, tile: Tile) -> f32 {
        let cx = tile.x.clamp(self.origin.x, self.origin.x + self.width - 1);
        let cy = tile.y.clamp(self.origin.y, self.origin.y + self.height - 1);
        tile.distance(Tile::new(cx, cy))
    }

    pub fn overlaps(&self, other: &Footprint) -> bool {
        self.origin.x < other.origin.x + other.width
            && other.origin.x < self.origin.x + self.width
            && self.origin.y < other.origin.y + other.height
            && other.origin.y < self.origin.y + self.height
    }
}
