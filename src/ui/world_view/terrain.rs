//! Baking the terrain and fog of war into textures.

use crate::world::{Terrain, Tile, World};
use macroquad::prelude::*;
use macroquad_toolkit::noise::seeded_value;

/// Texture pixels per map tile.
const DETAIL: i32 = 12;
/// How far (in tiles) terrain borders wander from the grid.
const WARP: f32 = 0.7;

fn base_color(terrain: Terrain, fertility: f32) -> [f32; 3] {
    match terrain {
        Terrain::Grass => [0.27 + fertility * 0.03, 0.35 + fertility * 0.07, 0.21],
        Terrain::Soil => [0.31, 0.25, 0.17],
        Terrain::Sand => [0.53, 0.47, 0.35],
        Terrain::Rock => [0.25, 0.25, 0.27],
        Terrain::Water => [0.11, 0.23, 0.30],
        Terrain::Snow => [0.72, 0.76, 0.80],
        Terrain::Ash => [0.26, 0.24, 0.23],
        Terrain::Scorch => [0.17, 0.15, 0.13],
    }
}

fn terrain_of(world: &World, tile: Tile) -> Terrain {
    let clamped = Tile::new(
        tile.x.clamp(0, world.map.width - 1),
        tile.y.clamp(0, world.map.height - 1),
    );
    world.map.terrain_at(clamped).unwrap_or(Terrain::Grass)
}

/// Organic, painterly terrain: borders are domain-warped so the grid never
/// shows, then layered with broad tonal noise and fine grain.
pub fn bake_terrain(world: &World, seed: u64) -> Texture2D {
    let width = world.map.width * DETAIL;
    let height = world.map.height * DETAIL;
    let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);
    let detail = DETAIL as f32;
    // Three warped samples per pixel blend into soft, irregular borders.
    let offsets: [(u64, f32); 3] = [(0x51ed, 1.0), (0x2b7f, 0.6), (0x9c13, 1.4)];
    for py in 0..height {
        for px in 0..width {
            let mut color = [0.0f32; 3];
            let mut water = 0.0;
            let mut rock = 0.0;
            for (salt, strength) in offsets {
                let coarse_x = seeded_value(seed ^ salt, px, py, detail * 2.2) - 0.5;
                let coarse_y = seeded_value(seed ^ salt.rotate_left(7), px, py, detail * 2.2) - 0.5;
                let fine_x = seeded_value(seed ^ salt.rotate_left(13), px, py, detail * 0.7) - 0.5;
                let fine_y = seeded_value(seed ^ salt.rotate_left(19), px, py, detail * 0.7) - 0.5;
                let fx = px as f32 / detail + (coarse_x * 2.0 + fine_x * 0.8) * WARP * strength;
                let fy = py as f32 / detail + (coarse_y * 2.0 + fine_y * 0.8) * WARP * strength;
                let tile = Tile::new(fx.floor() as i32, fy.floor() as i32);
                let terrain = terrain_of(world, tile);
                let sample = base_color(terrain, world.map.fertility_at(tile) / 100.0);
                for (channel, value) in color.iter_mut().zip(sample) {
                    *channel += value / 3.0;
                }
                water += if terrain == Terrain::Water {
                    1.0 / 3.0
                } else {
                    0.0
                };
                rock += if terrain == Terrain::Rock {
                    1.0 / 3.0
                } else {
                    0.0
                };
            }
            let broad = seeded_value(seed, px, py, detail * 3.5) - 0.5;
            let medium = seeded_value(seed ^ 0x77, px, py, detail * 0.6) - 0.5;
            let grain = seeded_value(seed ^ 0x9e37, px, py, 1.0) - 0.5;
            let mut shade = 1.0 + broad * 0.24 + medium * 0.12 + grain * 0.07;
            shade += water * ((px as f32 * 0.07 + py as f32 * 0.03).sin() * 0.5 + 0.5) * 0.06;
            shade += rock * medium.max(0.0) * 0.4;
            image.set_pixel(
                px as u32,
                py as u32,
                Color::new(color[0] * shade, color[1] * shade, color[2] * shade, 1.0),
            );
        }
    }
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Linear);
    texture
}

/// One pixel per tile: dark where nobody has looked yet.
pub fn bake_fog(world: &World) -> Texture2D {
    let mut image = Image::gen_image_color(world.map.width as u16, world.map.height as u16, BLANK);
    for y in 0..world.map.height {
        for x in 0..world.map.width {
            let explored = world.map.is_explored(Tile::new(x, y));
            let alpha = if explored { 0.0 } else { 0.93 };
            image.set_pixel(x as u32, y as u32, Color::new(0.03, 0.035, 0.04, alpha));
        }
    }
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Linear);
    texture
}
