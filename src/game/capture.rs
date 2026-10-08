//! Named scenes for the screenshot harness: each puts the game into a state
//! worth reviewing at 1920×1080.

use super::Game;
use crate::autoplay;
use crate::colony::RunSetup;
use crate::data::{game_data, CreatureKind, NodeKind};
use crate::sim::Sim;
use crate::state::play::PlayState;
use crate::state::{Screen, SetupState, TitleState};
use crate::ui::actions::{InspectorTab, Overlay, Selection};
use crate::world::{Calendar, Creature, CreatureMood, Point, ResourceNode, Terrain, Tile, World};
use std::collections::VecDeque;

const PLANS_PER_DAY: u64 = 6;

/// Play a colony with the scripted colony AI to a given day.
fn played_to(day: u32) -> Sim {
    let mut sim = Sim::new(RunSetup {
        colony_name: "New Meridian".into(),
        site: "verdant_basin".into(),
        difficulty: "standard".into(),
        seed: 11,
    });
    let step = Calendar::ticks_per_day() / PLANS_PER_DAY;
    while sim.calendar.day() < day && !sim.colony.campaign.is_over() {
        autoplay::plan(&mut sim);
        sim.run_ticks(step);
    }
    sim
}

/// Advance to a time of day so lighting and activities read as intended.
fn at_hour(mut sim: Sim, hour: f32) -> Sim {
    let ticks_per_day = Calendar::ticks_per_day();
    let target = (hour / 24.0 * ticks_per_day as f32) as u64;
    while sim.calendar.tick_of_day() != target {
        sim.step();
    }
    sim
}

fn busiest_mind(sim: &Sim) -> Option<u32> {
    sim.present()
        .filter(|a| !a.is_child && a.intention.as_ref().is_some_and(|i| i.plan.is_some()))
        .max_by_key(|a| a.decisions.entries.len() + a.beliefs.relations.len())
        .map(|a| a.id)
}

/// Clear the map to explored grass with nothing on it.
fn clear_ground(world: &mut World) {
    let structures: Vec<u32> = world.structures.iter().map(|s| s.id).collect();
    for id in structures {
        world.remove_structure(id);
    }
    let nodes: Vec<u32> = world.nodes.iter().map(|n| n.id).collect();
    for id in nodes {
        world.remove_node(id);
    }
    world.items.clear();
    for index in 0..world.map.terrain.len() {
        world.map.terrain[index] = Terrain::Grass;
        world.map.explored[index] = true;
    }
}

/// Every building, landscape feature and creature on cleared grass, so each
/// painted or drawn asset can be checked against the ground it stands on.
fn asset_gallery(mut sim: Sim) -> Sim {
    let world = &mut sim.world;
    clear_ground(world);
    // The first row starts clear of the act tracker in the upper left.
    let (left, right) = (4, 60);
    let (mut x, mut y, mut row_height) = (left + 14, 4, 0);
    for def in &game_data().buildings {
        let [w, h] = def.size;
        if x + w > right {
            x = left;
            y += row_height + 2;
            row_height = 0;
        }
        world.add_structure(def, Tile::new(x, y), 1, true);
        x += w + 2;
        row_height = row_height.max(h);
    }
    let mut x = left;
    let y = y + row_height + 3;
    for kind in NodeKind::ALL {
        for depleted in [false, true] {
            let id = world.allocate_node_id();
            world.add_node(ResourceNode {
                id,
                kind,
                tile: Tile::new(x, y),
                amount: if depleted { 0.0 } else { 40.0 },
                max_amount: 40.0,
                find: game_data().finds_of(kind).next().map(|f| f.id.clone()),
            });
            x += 2;
        }
        x += 1;
    }
    for kind in [
        CreatureKind::Skitter,
        CreatureKind::Ridgeback,
        CreatureKind::Driftmaw,
    ] {
        let tile = Tile::new(x, y);
        let id = world.next_creature_id;
        world.next_creature_id += 1;
        world.creatures.push(Creature {
            id,
            kind,
            tile,
            position: Point::of_tile(tile),
            path: VecDeque::new(),
            move_budget: 0.0,
            health: 10.0,
            mood: CreatureMood::Approaching,
            target: None,
            ticks_alive: 0,
            stolen: 0.0,
        });
        x += 3;
    }
    sim
}

/// Where the species gallery starts, so the scene can frame it.
const SPECIES_ORIGIN: Tile = Tile::new(10, 8);

/// Every native species on cleared grass, one row per node kind, each row
/// ending with its first species nearly spent.
fn species_gallery(mut sim: Sim) -> Sim {
    let world = &mut sim.world;
    clear_ground(world);
    let mut y = SPECIES_ORIGIN.y;
    for kind in NodeKind::ALL {
        let mut finds: Vec<(Option<String>, f32)> = game_data()
            .finds_of(kind)
            .map(|f| (Some(f.id.clone()), 40.0))
            .collect();
        if finds.is_empty() {
            finds.push((None, 40.0));
        }
        finds.push((finds[0].0.clone(), 0.0));
        for (column, (find, amount)) in finds.into_iter().enumerate() {
            let id = world.allocate_node_id();
            world.add_node(ResourceNode {
                id,
                kind,
                tile: Tile::new(SPECIES_ORIGIN.x + column as i32 * 3, y),
                amount,
                max_amount: 40.0,
                find,
            });
        }
        y += 2;
    }
    sim
}

fn playing(game: &mut Game, sim: Sim) -> &mut PlayState {
    game.screen = Screen::Playing(Box::new(PlayState::new(sim, false)));
    match &mut game.screen {
        Screen::Playing(play) => play,
        _ => unreachable!("screen was just set to playing"),
    }
}

/// Scenes ending in `_min` are the same states captured at the minimum
/// supported window size, so they get their own stable file names.
pub fn begin_scene(game: &mut Game, scene: &str) {
    let scene = scene.strip_suffix("_min").unwrap_or(scene);
    match scene {
        "title" => {
            game.screen = Screen::Title(TitleState {
                has_save: true,
                message: None,
            })
        }
        "setup" => {
            game.screen = Screen::Setup(SetupState {
                site: "verdant_basin".into(),
                difficulty: "standard".into(),
                seed: 4242,
            })
        }
        "landing" => {
            let play = playing(game, at_hour(played_to(1), 9.0));
            play.tutorial_step = Some(0);
        }
        "assets" => {
            let play = playing(game, asset_gallery(at_hour(played_to(1), 11.0)));
            play.speed = crate::state::play::SPEED_PAUSED;
            play.tracker_open = false;
            play.camera.zoom = 0.9;
            play.camera.target = crate::ui::camera::tile_center(Tile::new(32, 19));
        }
        "species" => {
            let play = playing(game, species_gallery(at_hour(played_to(1), 11.0)));
            play.speed = crate::state::play::SPEED_PAUSED;
            play.tracker_open = false;
            play.camera.zoom = 2.0;
            let centre = Tile::new(SPECIES_ORIGIN.x + 10, SPECIES_ORIGIN.y + 5);
            play.camera.target = crate::ui::camera::tile_center(centre);
        }
        "colony" => {
            playing(game, at_hour(played_to(14), 10.0));
        }
        "night" => {
            playing(game, at_hour(played_to(30), 22.5));
        }
        "late" => {
            let play = playing(game, at_hour(played_to(150), 15.0));
            play.camera.zoom = 0.7;
        }
        "mind" | "needs" | "bonds" | "life" => {
            let sim = at_hour(played_to(40), 11.0);
            let focus = busiest_mind(&sim);
            let play = playing(game, sim);
            play.selection = focus.map(Selection::Agent);
            play.follow = true;
            play.inspector_tab = match scene {
                "needs" => InspectorTab::Needs,
                "bonds" => InspectorTab::Bonds,
                "life" => InspectorTab::Life,
                _ => InspectorTab::Mind,
            };
            if let Some(agent) = focus.and_then(|id| play.sim.agent(id)) {
                play.camera.target = crate::ui::camera::point_center(agent.position);
                play.camera.zoom = 1.4;
            }
        }
        "build" => {
            let play = playing(game, at_hour(played_to(8), 11.0));
            play.build_open = true;
            play.tool = Some("habitat_dome".into());
            play.build_category = crate::data::BuildingCategory::Shelter;
        }
        "research" | "colonists" | "relations" | "expeditions" | "chronicle" | "colony_overlay" => {
            let play = playing(game, played_to(90));
            play.overlay = Some(match scene {
                "research" => Overlay::Research,
                "colonists" => Overlay::Colonists,
                "relations" => Overlay::Relations,
                "expeditions" => Overlay::Expeditions,
                "chronicle" => Overlay::Chronicle,
                _ => Overlay::Colony,
            });
            play.tech_focus = play
                .sim
                .colony
                .tree
                .available()
                .first()
                .map(|s| s.to_string());
            play.relation_focus = busiest_mind(&play.sim);
        }
        "council" => {
            let mut sim = played_to(1);
            let step = Calendar::ticks_per_day() / PLANS_PER_DAY;
            let council_ready = |sim: &Sim| {
                sim.colony.campaign.act >= 4 && sim.world.count_built("council_hall") > 0
            };
            while !council_ready(&sim) && sim.calendar.day() < 400 && !sim.colony.campaign.is_over()
            {
                autoplay::plan_without_vote(&mut sim);
                sim.run_ticks(step);
            }
            let play = playing(game, sim);
            play.overlay = Some(Overlay::Council);
        }
        "results" => {
            let sim = played_to(500);
            playing(game, sim);
        }
        _ => eprintln!("unknown capture scene '{scene}'"),
    }
}
