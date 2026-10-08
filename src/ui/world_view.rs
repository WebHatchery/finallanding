//! The colony map: terrain, features, buildings, people, fauna, weather and
//! the placement ghost, drawn under the world camera.

pub mod actors;
pub mod features;
pub mod glyphs;
pub mod terrain;

use super::actions::Selection;
use super::art::Art;
use super::camera::{tile_center, tile_size};
use super::context::{label_centered, text_width};
use super::theme::*;
use crate::data::{game_data, Resource, Weather};
use crate::state::play::PlayState;
use crate::world::{Footprint, Tile};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::VirtualUi;

const FOG_REFRESH_TICKS: u64 = 20;

fn refresh_cache(play: &mut PlayState) {
    if play.render.terrain.is_none() {
        play.render.terrain = Some(terrain::bake_terrain(
            &play.sim.world,
            play.sim.colony.setup.seed,
        ));
    }
    let tick = play.sim.calendar.tick;
    if play.render.fog.is_none()
        || tick >= play.render.fog_tick + FOG_REFRESH_TICKS
        || tick < play.render.fog_tick
    {
        play.render.fog = Some(terrain::bake_fog(&play.sim.world));
        play.render.fog_tick = tick;
    }
}

/// Where the placement ghost sits for a footprint centred under the pointer.
pub fn ghost_origin(building: &str, pointer_tile: Tile) -> Tile {
    let size = game_data()
        .building(building)
        .map(|b| b.size)
        .unwrap_or([1, 1]);
    pointer_tile.offset(-(size[0] - 1) / 2, -(size[1] - 1) / 2)
}

fn draw_items(play: &PlayState) {
    for item in &play.sim.world.items {
        let c = tile_center(item.tile);
        let color = resource_color(item.resource);
        draw_rectangle(
            c.x - 6.0,
            c.y - 4.0,
            12.0,
            10.0,
            macroquad_toolkit::colors::darken(color, 0.3),
        );
        draw_rectangle(c.x - 5.0, c.y - 6.0, 10.0, 8.0, color);
    }
}

fn draw_ghost(play: &PlayState, pointer_tile: Tile) {
    let Some(building) = &play.tool else {
        return;
    };
    let Some(def) = game_data().building(building) else {
        return;
    };
    let origin = ghost_origin(building, pointer_tile);
    let valid = play.sim.world.check_placement(def, origin).is_ok();
    let color = if valid { GOOD } else { BAD };
    let s = tile_size();
    let fp = Footprint::new(origin, def.size);
    let rect = Rect::new(
        fp.origin.x as f32 * s,
        fp.origin.y as f32 * s,
        fp.width as f32 * s,
        fp.height as f32 * s,
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, with_alpha(color, 0.25));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, color);
}

fn draw_selection_outline(play: &PlayState) {
    let s = tile_size();
    match play.selection {
        Some(Selection::Structure(id)) => {
            if let Some(rect) = features::structure_rect(&play.sim.world, id) {
                draw_rectangle_lines(
                    rect.x - 3.0,
                    rect.y - 3.0,
                    rect.w + 6.0,
                    rect.h + 6.0,
                    3.0,
                    SELECT,
                );
                if let Some(structure) = play.sim.world.structure(id) {
                    for resident in structure
                        .residents
                        .iter()
                        .filter_map(|r| play.sim.agent(*r))
                    {
                        let to = super::camera::point_center(resident.position);
                        let from = rect.center();
                        draw_line(from.x, from.y, to.x, to.y, 1.5, with_alpha(SELECT, 0.4));
                    }
                }
            }
        }
        Some(Selection::Node(id)) => {
            if let Some(node) = play.sim.world.node(id) {
                let c = tile_center(node.tile);
                draw_circle_lines(c.x, c.y, s * 0.7, 2.5, SELECT);
            }
        }
        Some(Selection::Creature(id)) => {
            if let Some(creature) = play.sim.world.creatures.iter().find(|c| c.id == id) {
                let c = super::camera::point_center(creature.position);
                draw_circle_lines(c.x, c.y, s * 0.8, 2.5, BAD);
            }
        }
        _ => {}
    }
}

fn draw_world_layers(play: &PlayState, art: &Art, pointer_tile: Option<Tile>) {
    let s = tile_size();
    let world = &play.sim.world;
    let size = vec2(world.map.width as f32 * s, world.map.height as f32 * s);
    if let Some(texture) = &play.render.terrain {
        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    }
    let (min, max) = play.camera.visible_tiles();
    let visible = |tile: Tile| {
        tile.x >= min.x - 3 && tile.y >= min.y - 3 && tile.x <= max.x + 3 && tile.y <= max.y + 3
    };
    let time = play.clock;
    let night = 1.0 - play.sim.calendar.daylight_level();
    for node in world.nodes.iter().filter(|n| visible(n.tile)) {
        features::draw_node(node, time);
    }
    let mut structures: Vec<_> = world
        .structures
        .iter()
        .filter(|st| visible(st.footprint.origin))
        .collect();
    structures.sort_by_key(|st| st.footprint.origin.y + st.footprint.height);
    for structure in structures {
        features::draw_structure(structure, art, time, night);
    }
    draw_items(play);
    draw_selection_outline(play);
    if let Some(Selection::Agent(id)) = play.selection {
        if let Some(agent) = play.sim.agent(id) {
            actors::draw_agent_overlay(agent, &play.sim.agents);
        }
    }
    for agent in play
        .sim
        .agents
        .iter()
        .filter(|a| a.is_present() && visible(a.tile))
    {
        let selected = play.selection == Some(Selection::Agent(agent.id));
        actors::draw_agent(agent, world, time, selected);
    }
    for creature in world.creatures.iter().filter(|c| visible(c.tile)) {
        let max = game_data()
            .balance
            .creatures
            .get(&creature.kind)
            .map(|b| b.health)
            .unwrap_or(1.0);
        actors::draw_creature(creature, max, time);
    }
    if let Some(tile) = pointer_tile {
        draw_ghost(play, tile);
    }
    if let Some(fog) = &play.render.fog {
        draw_texture_ex(
            fog,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    }
    if night > 0.02 {
        let pad = 4000.0;
        draw_rectangle(
            -pad,
            -pad,
            size.x + pad * 2.0,
            size.y + pad * 2.0,
            Color::new(0.02, 0.03, 0.09, night * 0.5),
        );
    }
}

fn draw_weather(play: &PlayState) {
    let weather = play.sim.world.weather.current;
    let (count, color, slant, speed) = match weather {
        Weather::Rain => (140, Color::new(0.6, 0.7, 0.85, 0.35), 0.25, 900.0),
        Weather::Storm => (260, Color::new(0.65, 0.75, 0.9, 0.45), 0.5, 1300.0),
        Weather::Snow => (160, Color::new(0.95, 0.95, 1.0, 0.7), 0.1, 120.0),
        Weather::Ashfall => (180, Color::new(0.5, 0.48, 0.45, 0.6), 0.15, 90.0),
        _ => return,
    };
    let t = play.clock;
    for i in 0..count {
        let seed = i as f32 * 12.9898;
        let x = ((seed.sin() * 43_758.547).fract().abs() * VIRTUAL_WIDTH + t * speed * slant)
            % VIRTUAL_WIDTH;
        let y = (((seed * 1.7).cos() * 9_631.13).fract().abs() * VIRTUAL_HEIGHT + t * speed)
            % VIRTUAL_HEIGHT;
        if speed > 300.0 {
            draw_line(x, y, x - 10.0 * slant, y - 14.0, 1.2, color);
        } else {
            draw_circle(x, y, 1.8, color);
        }
    }
    if weather == Weather::Storm && (t * 0.7).sin() > 0.995 {
        draw_rectangle(
            0.0,
            0.0,
            VIRTUAL_WIDTH,
            VIRTUAL_HEIGHT,
            Color::new(0.9, 0.95, 1.0, 0.25),
        );
    }
}

/// Names and short labels drawn in screen space so text stays crisp.
fn draw_labels(play: &PlayState, pointer_tile: Option<Tile>) {
    let zoom = play.camera.zoom;
    for structure in play.sim.world.structures.iter() {
        let selected = play.selection == Some(Selection::Structure(structure.id));
        let hovered = pointer_tile.is_some_and(|t| structure.footprint.contains(t));
        if zoom < 1.7 && !selected && !hovered {
            continue;
        }
        let s = tile_size();
        let fp = structure.footprint;
        let top = play.camera.world_to_screen(vec2(
            (fp.origin.x as f32 + fp.width as f32 * 0.5) * s,
            fp.origin.y as f32 * s,
        ));
        if !(0.0..VIRTUAL_WIDTH).contains(&top.x) || !(0.0..VIRTUAL_HEIGHT).contains(&top.y) {
            continue;
        }
        let name = &structure.def().name;
        let width = text_width(name, TEXT_SMALL) + 12.0;
        draw_rectangle(
            top.x - width * 0.5,
            top.y - 26.0,
            width,
            22.0,
            Color::new(0.04, 0.05, 0.06, 0.7),
        );
        label_centered(
            name,
            top.x,
            top.y - 25.0,
            TEXT_SMALL,
            if structure.is_built() { TEXT } else { CYAN },
        );
    }
    for agent in play
        .sim
        .agents
        .iter()
        .filter(|a| a.is_present() && a.inside.is_none())
    {
        let selected = play.selection == Some(Selection::Agent(agent.id));
        if zoom < 1.5 && !selected {
            continue;
        }
        let at = play
            .camera
            .world_to_screen(super::camera::point_center(agent.position));
        label_centered(
            &agent.given_name,
            at.x,
            at.y + 12.0 * zoom,
            TEXT_SMALL,
            if selected { SELECT } else { TEXT },
        );
    }
}

fn draw_pointer_hint(play: &PlayState, pointer_tile: Tile, pointer: Vec2) {
    let Some(building) = &play.tool else {
        return;
    };
    let Some(def) = game_data().building(building) else {
        return;
    };
    let origin = ghost_origin(building, pointer_tile);
    let text = match play.sim.world.check_placement(def, origin) {
        Ok(()) => {
            let short: Vec<String> = def
                .cost_bag()
                .nonzero()
                .filter(|(r, a)| play.sim.colony.stock.get(*r) < *a && *r != Resource::Meals)
                .map(|(r, _)| game_data().label(r.key()).to_owned())
                .collect();
            if short.is_empty() {
                game_data().label("ghost_place").to_owned()
            } else {
                format!("{} {}", game_data().label("ghost_short"), short.join(", "))
            }
        }
        Err(issue) => game_data().label(issue.label_key()).to_owned(),
    };
    let width = text_width(&text, TEXT_SMALL) + 16.0;
    draw_rectangle(
        pointer.x + 18.0,
        pointer.y + 14.0,
        width,
        26.0,
        Color::new(0.04, 0.05, 0.06, 0.85),
    );
    super::context::label(&text, pointer.x + 26.0, pointer.y + 17.0, TEXT_SMALL, TEXT);
}

/// Draw the world. `pointer` is the pointer in virtual-screen space when it
/// is over the map rather than the UI.
pub fn draw(play: &mut PlayState, art: &Art, frame: &VirtualUi, pointer: Option<Vec2>) {
    refresh_cache(play);
    let pointer_tile = pointer.map(|p| play.camera.tile_at(p));
    set_camera(&play.camera.macroquad_camera(frame));
    draw_world_layers(play, art, pointer_tile);
    frame.begin();
    draw_weather(play);
    draw_labels(play, pointer_tile);
    if let (Some(tile), Some(p)) = (pointer_tile, pointer) {
        draw_pointer_hint(play, tile, p);
    }
}
