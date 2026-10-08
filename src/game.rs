//! The game shell: screens, the frame loop, autosave and capture scenes.

mod capture;
mod dispatch;

pub use capture::begin_scene as begin_capture_scene;

use crate::state::play::{PlayState, Toast};
use crate::state::save::{self, Preferences};
use crate::state::{Screen, TitleState};
use crate::ui::art::Art;
use crate::ui::context::Ui;
use crate::ui::theme::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::ui::tracker::toast_seconds;
use crate::ui::{input, play_screen, screens};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{Pointer, VirtualUi};

pub struct Game {
    pub screen: Screen,
    pub art: Art,
    pub preferences: Preferences,
    pub quit_requested: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            screen: Screen::Title(TitleState {
                has_save: save::has_save(),
                message: None,
            }),
            art: Art::load(),
            preferences: save::load_preferences(),
            quit_requested: false,
        }
    }

    /// One frame: draw the current screen, apply what the player asked for,
    /// then advance the simulation.
    pub fn frame(&mut self, dt: f32, pointer_enabled: bool) {
        clear_background(BLACK);
        let frame = VirtualUi::new(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
        let pointer = if pointer_enabled {
            Pointer::read(|p| frame.screen_to_ui(p))
        } else {
            Pointer::default()
        };
        let wheel = if pointer_enabled {
            mouse_wheel().1
        } else {
            0.0
        };
        let mut ui = Ui::new(pointer, wheel, get_time() as f32);
        frame.begin();
        match &mut self.screen {
            Screen::Title(title) => screens::draw_title(&mut ui, &self.art, title),
            Screen::Setup(setup) => screens::draw_setup(&mut ui, &self.art, setup),
            Screen::Playing(play) => {
                play_screen::draw(&mut ui, &self.art, play, &frame);
                if !play.sim.colony.campaign.is_over() {
                    if pointer_enabled {
                        input::world_pointer(&mut ui, play);
                    }
                    input::keyboard(&mut ui, play, dt);
                }
            }
        }
        set_default_camera();
        for action in ui.take_actions() {
            dispatch::apply(self, action);
        }
        if let Screen::Playing(play) = &mut self.screen {
            play.advance(dt);
            follow_selection(play);
            collect_toasts(play);
            autosave(play);
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn follow_selection(play: &mut PlayState) {
    if !play.follow {
        return;
    }
    if let Some(crate::ui::actions::Selection::Agent(id)) = play.selection {
        if let Some(agent) = play.sim.agent(id) {
            let target = crate::ui::camera::point_center(agent.position);
            play.camera.target += (target - play.camera.target) * 0.15;
        }
    }
}

/// Major chronicle entries surface briefly as toasts.
fn collect_toasts(play: &mut PlayState) {
    let entries = &play.sim.colony.chronicle.entries;
    if play.seen_entries > entries.len() {
        play.seen_entries = entries.len();
    }
    for entry in entries.iter().skip(play.seen_entries) {
        if entry.importance >= 2 {
            play.toasts.push(Toast {
                text: entry.text.clone(),
                category: entry.category,
                born: play.clock,
            });
        }
    }
    play.seen_entries = entries.len();
    let clock = play.clock;
    play.toasts.retain(|t| clock - t.born < toast_seconds());
    let excess = play.toasts.len().saturating_sub(6);
    play.toasts.drain(..excess);
}

/// Save once per in-game day and when a run ends.
fn autosave(play: &mut PlayState) {
    let day = play.sim.calendar.day();
    if day != play.last_autosave_day {
        play.last_autosave_day = day;
        if let Err(error) = save::save(&play.sim) {
            eprintln!("autosave failed: {error}");
        }
    }
}
