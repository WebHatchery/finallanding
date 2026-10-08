//! Seeded procedural generation of a landing site.

use super::{ResourceNode, Terrain, Tile, WeatherState, World, WorldMap};
use crate::data::campaign::SiteDef;
use crate::data::{game_data, NodeKind};
use macroquad_toolkit::noise::seeded_value;
use macroquad_toolkit::rng::SeededRng;

/// Radius around the landing kept free of rock and water.
const START_REVEAL_RADIUS: f32 = 15.0;

pub fn generate_world(site: &SiteDef, seed: u64, rng: &mut SeededRng) -> World {
    let balance = &game_data().balance.map;
    let mut map = WorldMap::new(
        balance.width,
        balance.height,
        Terrain::from_key(&site.terrain.ground),
    );
    let landing = Tile::new(
        balance.width / 2 + rng.range_i32(-6, 7),
        balance.height / 2 + rng.range_i32(-4, 5),
    );
    paint_terrain(&mut map, site, seed, landing, balance.clearing_radius);
    let mut world = World {
        map,
        structures: Vec::new(),
        nodes: Vec::new(),
        items: Vec::new(),
        creatures: Vec::new(),
        weather: WeatherState::default(),
        next_structure_id: 1,
        next_node_id: 1,
        next_creature_id: 1,
        landing_tile: landing,
    };
    place_hull(&mut world, landing);
    scatter_crash_debris(&mut world, landing, rng);
    guarantee_starting_supplies(&mut world, landing, rng);
    scatter_nodes(&mut world, site, landing, rng);
    world.map.reveal_circle(landing, START_REVEAL_RADIUS);
    world
}

fn paint_terrain(map: &mut WorldMap, site: &SiteDef, seed: u64, landing: Tile, clearing: i32) {
    let mix = &site.terrain;
    for y in 0..map.height {
        for x in 0..map.width {
            let tile = Tile::new(x, y);
            let elevation =
                seeded_value(seed, x, y, 9.0) * 0.7 + seeded_value(seed ^ 0xA5, x, y, 3.5) * 0.3;
            let moisture = seeded_value(seed ^ 0x51, x, y, 11.0);
            let richness = seeded_value(seed ^ 0x77, x, y, 6.0);
            let index = map.index(tile).unwrap_or(0);
            map.fertility[index] = (richness * 100.0) as u8;
            let near_landing = tile.distance(landing) <= clearing as f32;
            let terrain = if !near_landing && elevation > 1.0 - mix.rock * 2.2 {
                Terrain::Rock
            } else if !near_landing && moisture < mix.water * 2.0 {
                Terrain::Water
            } else if moisture < mix.water * 2.0 + mix.sand {
                Terrain::Sand
            } else if richness > 1.0 - mix.fertile {
                Terrain::Soil
            } else {
                Terrain::from_key(&mix.ground)
            };
            map.terrain[index] = terrain;
            if terrain == Terrain::Soil {
                map.fertility[index] = map.fertility[index].max(70);
            }
        }
    }
    // Scorched ground where the hull slid to a stop.
    for dy in -4..=4 {
        for dx in -9..=9 {
            let tile = landing.offset(dx, dy);
            if (dx as f32 / 9.0).powi(2) + (dy as f32 / 4.0).powi(2) <= 1.0 {
                map.set_terrain(tile, Terrain::Scorch);
            }
        }
    }
}

fn place_hull(world: &mut World, landing: Tile) {
    let Some(def) = game_data().building("meridian_hull") else {
        return;
    };
    let origin = landing.offset(-def.size[0] / 2, -def.size[1] / 2);
    world.add_structure(def, origin, 1, true);
}

fn random_tile_near(rng: &mut SeededRng, center: Tile, min: f32, max: f32) -> Tile {
    let angle = rng.range_f32(0.0, std::f32::consts::TAU);
    let distance = rng.range_f32(min, max);
    Tile::new(
        center.x + (angle.cos() * distance).round() as i32,
        center.y + (angle.sin() * distance).round() as i32,
    )
}

fn node_amount(kind: NodeKind, rng: &mut SeededRng) -> f32 {
    match kind {
        NodeKind::Wreckage => rng.range_f32(60.0, 120.0),
        NodeKind::FibreGrove => 40.0,
        NodeKind::StoneOutcrop => rng.range_f32(150.0, 250.0),
        NodeKind::OreVein => rng.range_f32(110.0, 190.0),
        NodeKind::Glowfruit => 30.0,
        NodeKind::Ruin => rng.range_f32(3.0, 6.0).round(),
    }
}

fn try_place_node(world: &mut World, kind: NodeKind, tile: Tile, rng: &mut SeededRng) -> bool {
    let Some(index) = world.map.index(tile) else {
        return false;
    };
    let free = world.map.terrain[index].is_walkable()
        && world.map.structure_at[index].is_none()
        && world.map.node_at[index].is_none();
    let crowded = tile
        .neighbours()
        .iter()
        .any(|n| world.map.structure_on(*n).is_some());
    if !free || crowded {
        return false;
    }
    let amount = node_amount(kind, rng);
    let id = world.allocate_node_id();
    world.add_node(ResourceNode {
        id,
        kind,
        tile,
        amount,
        max_amount: amount,
    });
    true
}

fn place_near(
    world: &mut World,
    kind: NodeKind,
    center: Tile,
    range: (f32, f32),
    rng: &mut SeededRng,
) {
    for _ in 0..40 {
        let tile = random_tile_near(rng, center, range.0, range.1);
        if try_place_node(world, kind, tile, rng) {
            return;
        }
    }
}

fn scatter_crash_debris(world: &mut World, landing: Tile, rng: &mut SeededRng) {
    for _ in 0..9 {
        place_near(world, NodeKind::Wreckage, landing, (4.0, 13.0), rng);
    }
}

/// Every run starts with food, fibre and stone within a short walk, so the
/// first days test planning rather than luck.
fn guarantee_starting_supplies(world: &mut World, landing: Tile, rng: &mut SeededRng) {
    for _ in 0..3 {
        place_near(world, NodeKind::Glowfruit, landing, (6.0, 13.0), rng);
        place_near(world, NodeKind::FibreGrove, landing, (5.0, 13.0), rng);
    }
    for _ in 0..2 {
        place_near(world, NodeKind::StoneOutcrop, landing, (8.0, 15.0), rng);
    }
    place_near(world, NodeKind::OreVein, landing, (12.0, 20.0), rng);
}

fn scatter_nodes(world: &mut World, site: &SiteDef, landing: Tile, rng: &mut SeededRng) {
    let area = (world.map.width * world.map.height) as f32;
    for (kind, density) in &site.node_density {
        let count = (density * area / 1000.0).round() as usize;
        let min_distance = if *kind == NodeKind::Ruin { 28.0 } else { 10.0 };
        let mut placed = 0;
        let mut attempts = 0;
        while placed < count && attempts < count * 30 {
            attempts += 1;
            let tile = Tile::new(
                rng.range_i32(1, world.map.width - 1),
                rng.range_i32(1, world.map.height - 1),
            );
            if tile.distance(landing) < min_distance {
                continue;
            }
            if try_place_node(world, *kind, tile, rng) {
                placed += 1;
                if matches!(kind, NodeKind::FibreGrove | NodeKind::Glowfruit) && rng.chance(0.6) {
                    place_near(world, *kind, tile, (1.0, 2.5), rng);
                }
            }
        }
    }
}
