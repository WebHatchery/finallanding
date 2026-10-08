//! World input (tap, drag, wheel) and keyboard shortcuts, turned into
//! camera movement and UI actions.

use super::actions::{Overlay, Selection, UiAction};
use super::camera::point_center;
use super::context::Ui;
use super::play_screen::is_over_fixed_ui;
use super::world_view::ghost_origin;
use crate::state::play::{Drag, PlayState};
use crate::world::Tile;
use macroquad::prelude::*;

const DRAG_THRESHOLD: f32 = 10.0;
const KEY_PAN_SPEED: f32 = 900.0;

/// What is under a screen point, most specific first.
pub fn pick(play: &PlayState, screen: Vec2) -> Option<Selection> {
    let world = play.camera.screen_to_world(screen);
    let reach = 18.0 / play.camera.zoom.max(0.5) + 6.0;
    let agent = play
        .sim
        .agents
        .iter()
        .filter(|a| a.is_present() && a.inside.is_none())
        .map(|a| (a.id, point_center(a.position).distance(world)))
        .filter(|(_, d)| *d < reach)
        .min_by(|a, b| a.1.total_cmp(&b.1));
    if let Some((id, _)) = agent {
        return Some(Selection::Agent(id));
    }
    let creature = play
        .sim
        .world
        .creatures
        .iter()
        .find(|c| point_center(c.position).distance(world) < reach * 1.3);
    if let Some(creature) = creature {
        return Some(Selection::Creature(creature.id));
    }
    let tile = play.camera.tile_at(screen);
    if let Some(structure) = play.sim.world.structure_covering(tile) {
        return Some(Selection::Structure(structure.id));
    }
    play.sim.world.map.node_on(tile).map(Selection::Node)
}

fn tap(ui: &mut Ui, play: &PlayState, screen: Vec2) {
    if let Some(tool) = &play.tool {
        let tile: Tile = play.camera.tile_at(screen);
        ui.act(UiAction::PlaceAt(ghost_origin(tool, tile)));
        return;
    }
    match pick(play, screen) {
        Some(selection) => ui.act(UiAction::Select(selection)),
        None => ui.act(UiAction::Deselect),
    }
}

/// Pointer handling over the map. Mutates only the camera and drag state.
pub fn world_pointer(ui: &mut Ui, play: &mut PlayState) {
    let pointer = ui.pointer;
    let over_ui = is_over_fixed_ui(play, pointer.position) || ui.is_over_ui(pointer.position);
    if !over_ui && ui.wheel.abs() > 0.01 {
        let factor = if ui.wheel > 0.0 { 1.12 } else { 1.0 / 1.12 };
        play.camera.zoom_at(factor, pointer.position);
    }
    if pointer.down {
        match &mut play.drag {
            Some(drag) => {
                let delta = pointer.position - drag.last;
                if (pointer.position - drag.start).length() > DRAG_THRESHOLD {
                    drag.moved = true;
                }
                if drag.moved {
                    play.camera.pan_screen(delta);
                    play.follow = false;
                }
                drag.last = pointer.position;
            }
            None if !over_ui => {
                play.drag = Some(Drag {
                    start: pointer.position,
                    last: pointer.position,
                    moved: false,
                });
            }
            None => {}
        }
    }
    if pointer.released {
        if let Some(drag) = play.drag.take() {
            if !drag.moved && !over_ui {
                tap(ui, play, pointer.position);
            }
        }
    } else if !pointer.down {
        play.drag = None;
    }
    if is_mouse_button_released(MouseButton::Right) {
        ui.act(if play.tool.is_some() {
            UiAction::DisarmTool
        } else {
            UiAction::Deselect
        });
    }
    let map = &play.sim.world.map;
    play.camera.clamp(map.width, map.height);
}

fn keyboard_pan(play: &mut PlayState, dt: f32) {
    let mut direction = Vec2::ZERO;
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        direction.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        direction.x += 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        direction.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        direction.y += 1.0;
    }
    if direction != Vec2::ZERO {
        play.camera.target += direction * KEY_PAN_SPEED * dt / play.camera.zoom;
        play.follow = false;
    }
}

/// Keyboard shortcuts; each mirrors a visible control.
pub fn keyboard(ui: &mut Ui, play: &mut PlayState, dt: f32) {
    keyboard_pan(play, dt);
    let shortcuts = [
        (KeyCode::Space, UiAction::TogglePause),
        (KeyCode::Key1, UiAction::SetSpeed(1)),
        (KeyCode::Key2, UiAction::SetSpeed(2)),
        (KeyCode::Key3, UiAction::SetSpeed(3)),
        (KeyCode::B, UiAction::ToggleBuild),
        (KeyCode::C, UiAction::Open(Overlay::Colony)),
        (KeyCode::R, UiAction::Open(Overlay::Research)),
        (KeyCode::P, UiAction::Open(Overlay::Colonists)),
        (KeyCode::L, UiAction::Open(Overlay::Relations)),
        (KeyCode::X, UiAction::Open(Overlay::Expeditions)),
        (KeyCode::J, UiAction::Open(Overlay::Chronicle)),
        (KeyCode::F1, UiAction::Open(Overlay::Help)),
        (KeyCode::Q, UiAction::ZoomBy(0.85)),
        (KeyCode::E, UiAction::ZoomBy(1.18)),
    ];
    for (key, action) in shortcuts {
        if is_key_pressed(key) {
            ui.act(action);
        }
    }
    if is_key_pressed(KeyCode::Escape) {
        let action = if play.tool.is_some() {
            UiAction::DisarmTool
        } else if play.overlay.is_some() {
            UiAction::CloseOverlay
        } else if play.build_open {
            UiAction::ToggleBuild
        } else {
            UiAction::Deselect
        };
        ui.act(action);
    }
}
