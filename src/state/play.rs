//! The state of a colony being played: the simulation plus everything the
//! player is looking at.

use crate::colony::chronicle::Category;
use crate::data::{game_data, BuildingCategory};
use crate::sim::Sim;
use crate::ui::actions::{InspectorTab, Overlay, Selection};
use crate::ui::camera::WorldCamera;
use crate::world::AgentId;
use macroquad::prelude::{Texture2D, Vec2};

pub const SPEED_PAUSED: usize = 0;

#[derive(Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub category: Category,
    pub born: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Drag {
    pub start: Vec2,
    pub last: Vec2,
    pub moved: bool,
}

/// GPU textures built from the map; rebuilt when missing or stale.
#[derive(Default)]
pub struct RenderCache {
    pub terrain: Option<Texture2D>,
    pub fog: Option<Texture2D>,
    pub fog_tick: u64,
}

pub struct PlayState {
    pub sim: Sim,
    pub camera: WorldCamera,
    pub speed: usize,
    pub last_speed: usize,
    pub tick_accumulator: f32,
    pub selection: Option<Selection>,
    pub follow: bool,
    pub inspector_tab: InspectorTab,
    pub overlay: Option<Overlay>,
    pub build_open: bool,
    pub build_category: BuildingCategory,
    pub tool: Option<String>,
    pub chronicle_filter: Option<Category>,
    pub chronicle_day: Option<u32>,
    pub scroll: f32,
    pub tech_focus: Option<String>,
    pub relation_focus: Option<AgentId>,
    pub help_page: usize,
    pub tutorial_step: Option<usize>,
    pub tracker_open: bool,
    pub toasts: Vec<Toast>,
    pub seen_entries: usize,
    pub drag: Option<Drag>,
    pub last_autosave_day: u32,
    pub status: Option<(String, f32)>,
    pub render: RenderCache,
    pub clock: f32,
}

impl PlayState {
    pub fn new(sim: Sim, show_tutorial: bool) -> Self {
        let camera = WorldCamera::looking_at(sim.world.landing_tile);
        let seen = sim.colony.chronicle.entries.len();
        let day = sim.calendar.day();
        Self {
            sim,
            camera,
            speed: 1,
            last_speed: 1,
            tick_accumulator: 0.0,
            selection: None,
            follow: false,
            inspector_tab: InspectorTab::Mind,
            overlay: None,
            build_open: false,
            build_category: BuildingCategory::Shelter,
            tool: None,
            chronicle_filter: None,
            chronicle_day: None,
            scroll: 0.0,
            tech_focus: None,
            relation_focus: None,
            help_page: 0,
            tutorial_step: show_tutorial.then_some(0),
            tracker_open: true,
            toasts: Vec::new(),
            seen_entries: seen,
            drag: None,
            last_autosave_day: day,
            status: None,
            render: RenderCache::default(),
            clock: 0.0,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.speed == SPEED_PAUSED
            || self.tutorial_step.is_some()
            || self.sim.colony.campaign.is_over()
    }

    /// Advance the simulation by real time at the chosen speed.
    pub fn advance(&mut self, dt: f32) {
        self.clock += dt;
        if self.is_paused() {
            return;
        }
        let rate = game_data().balance.time.speed_ticks_per_second[self.speed.min(3)];
        self.tick_accumulator += dt.min(0.25) * rate;
        let mut steps = 0;
        while self.tick_accumulator >= 1.0 && steps < 30 {
            self.sim.step();
            self.tick_accumulator -= 1.0;
            steps += 1;
        }
    }

    pub fn flash(&mut self, text: String) {
        self.status = Some((text, self.clock));
    }

    /// Fraction of the way to the next tick, for smooth movement.
    pub fn tick_fraction(&self) -> f32 {
        self.tick_accumulator.fract()
    }
}
