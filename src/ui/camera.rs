//! The world camera: pan and zoom over the tile map in virtual-screen space.

use super::theme::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::data::game_data;
use crate::world::{Point, Tile};
use macroquad::prelude::*;
use macroquad_toolkit::ui::VirtualUi;

pub const MIN_ZOOM: f32 = 0.35;
pub const MAX_ZOOM: f32 = 2.2;

#[derive(Clone, Copy, Debug)]
pub struct WorldCamera {
    /// World position (pixels) at the centre of the screen.
    pub target: Vec2,
    pub zoom: f32,
}

pub fn tile_size() -> f32 {
    game_data().balance.map.tile_size
}

pub fn tile_center(tile: Tile) -> Vec2 {
    let size = tile_size();
    vec2((tile.x as f32 + 0.5) * size, (tile.y as f32 + 0.5) * size)
}

pub fn point_center(point: Point) -> Vec2 {
    let size = tile_size();
    vec2((point.x + 0.5) * size, (point.y + 0.5) * size)
}

fn screen_center() -> Vec2 {
    vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5)
}

impl WorldCamera {
    pub fn looking_at(tile: Tile) -> Self {
        Self {
            target: tile_center(tile),
            zoom: 1.3,
        }
    }

    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        (world - self.target) * self.zoom + screen_center()
    }

    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        (screen - screen_center()) / self.zoom + self.target
    }

    pub fn tile_at(&self, screen: Vec2) -> Tile {
        let world = self.screen_to_world(screen);
        let size = tile_size();
        Tile::new(
            (world.x / size).floor() as i32,
            (world.y / size).floor() as i32,
        )
    }

    pub fn pan_screen(&mut self, delta: Vec2) {
        self.target -= delta / self.zoom;
    }

    /// Zoom keeping the world point under `anchor` fixed on screen.
    pub fn zoom_at(&mut self, factor: f32, anchor: Vec2) {
        let before = self.screen_to_world(anchor);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let after = self.screen_to_world(anchor);
        self.target += before - after;
    }

    /// Keep the map in view.
    pub fn clamp(&mut self, map_width: i32, map_height: i32) {
        let size = tile_size();
        let max = vec2(map_width as f32 * size, map_height as f32 * size);
        self.target = self.target.clamp(Vec2::ZERO, max);
    }

    /// Visible world rectangle, in tiles (inclusive bounds), with a margin.
    pub fn visible_tiles(&self) -> (Tile, Tile) {
        let size = tile_size();
        let top_left = self.screen_to_world(vec2(0.0, 0.0)) / size;
        let bottom_right = self.screen_to_world(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)) / size;
        (
            Tile::new(top_left.x.floor() as i32 - 1, top_left.y.floor() as i32 - 1),
            Tile::new(
                bottom_right.x.ceil() as i32 + 1,
                bottom_right.y.ceil() as i32 + 1,
            ),
        )
    }

    /// A macroquad camera drawing world pixels into the letterboxed viewport.
    pub fn macroquad_camera(&self, frame: &VirtualUi) -> Camera2D {
        let center = screen_center();
        let world_center = self.screen_to_world(center);
        Camera2D {
            target: world_center,
            zoom: vec2(
                2.0 * self.zoom / VIRTUAL_WIDTH,
                2.0 * self.zoom / VIRTUAL_HEIGHT,
            ),
            viewport: Some(frame.viewport()),
            ..Default::default()
        }
    }
}
