//! The simulation: one tick advances the world, every survivor's BDI cycle,
//! fauna, the environment, the event director and the campaign.

pub mod body;
pub mod campaign;
pub mod chronicle_text;
pub mod cognition;
pub mod commands;
pub mod creatures;
pub mod daily;
pub mod director;
pub mod environment;
pub mod event_effects;
pub mod execute;
pub mod execute_life;
pub mod execute_social;
pub mod execute_work;
pub mod expeditions;
pub mod inbox;
pub mod jobs;
pub mod lifecycle;
pub mod movement;
pub mod perception;
pub mod research;
pub mod social;
pub mod vote;

use crate::agents::deliberation::{AgentSnapshot, View};
use crate::agents::generation::{create_survivor, Arrival};
use crate::agents::messages::Message;
use crate::agents::plans::Topic;
use crate::agents::Agent;
use crate::colony::chronicle::{Category, Entry};
use crate::colony::tech_tree::{Modifiers, TechTree};
use crate::colony::{Colony, RunSetup};
use crate::data::{game_data, Resource, ResourceBag};
use crate::world::mapgen::generate_world;
use crate::world::{AgentId, Calendar, Tile, World};
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};

/// A finished conversation, applied to both survivors after the tick.
#[derive(Clone, Debug)]
pub struct Interaction {
    pub from: AgentId,
    pub to: AgentId,
    pub topic: Topic,
}

/// Side effects gathered while survivors act, applied once per tick so every
/// survivor in a tick sees the same world.
#[derive(Default)]
pub struct Effects {
    pub messages: Vec<Message>,
    pub interactions: Vec<Interaction>,
    pub chronicle: Vec<Entry>,
    pub gifts: Vec<(AgentId, AgentId, Resource, f32)>,
    pub treatments: Vec<(AgentId, AgentId, bool)>,
    pub attacks: Vec<(AgentId, u32, f32)>,
}

/// Mutable access for executing one survivor's step.
pub struct Ctx<'a> {
    pub world: &'a mut World,
    pub colony: &'a mut Colony,
    pub others: &'a [AgentSnapshot],
    pub calendar: Calendar,
    pub rng: &'a mut SeededRng,
    pub effects: &'a mut Effects,
}

impl Ctx<'_> {
    pub fn view(&self) -> View<'_> {
        View {
            world: self.world,
            colony: self.colony,
            others: self.others,
            calendar: self.calendar,
        }
    }

    pub fn log(&mut self, category: Category, importance: u8, text: String, agents: Vec<AgentId>) {
        self.effects.chronicle.push(Entry {
            day: self.calendar.day(),
            hour: self.calendar.hour() as u8,
            category,
            text,
            agents,
            importance,
        });
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sim {
    pub world: World,
    pub agents: Vec<Agent>,
    pub colony: Colony,
    pub calendar: Calendar,
    pub rng: SeededRng,
    pub next_agent_id: AgentId,
}

impl Sim {
    /// Start a new run: generate the site, the tech tree and the crew.
    pub fn new(setup: RunSetup) -> Sim {
        let data = game_data();
        let mut rng = SeededRng::new(setup.seed.max(1));
        let site = data
            .site(&setup.site)
            .or_else(|| data.campaign.sites.first())
            .expect("campaign data has at least one landing site");
        let world = generate_world(site, setup.seed, &mut rng);
        let tree = TechTree::generate(&mut rng, &world.species);
        let mut colony = Colony::new(setup, tree);
        colony.modifiers = Modifiers::from_tree(&colony.tree);
        let scale = colony.difficulty_value(|d| d.starting_resources);
        let mut stock =
            ResourceBag::from_map(&data.balance.colony.starting_resources).scaled(scale);
        stock.add_bag(&ResourceBag::from_map(&site.bonus_resources));
        colony.stock = stock;
        let mut sim = Sim {
            world,
            agents: Vec::new(),
            colony,
            calendar: Calendar::default(),
            rng,
            next_agent_id: 1,
        };
        for _ in 0..data.balance.colony.starting_survivors {
            let tile = sim.arrival_tile();
            let id = sim.spawn_survivor(tile);
            if let Some(agent) = sim.agent_mut(id) {
                agent.mind.add("landed", None, Calendar::ticks_per_day());
            }
        }
        for agent in sim.agents.iter_mut() {
            agent.beliefs.stock = sim.colony.stock;
        }
        campaign::begin_act(&mut sim, 1);
        jobs::refresh(&mut sim);
        sim
    }

    pub fn agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == id)
    }

    pub fn agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == id)
    }

    pub fn present(&self) -> impl Iterator<Item = &Agent> {
        self.agents.iter().filter(|a| a.is_present())
    }

    /// Survivors alive, whether at home or on an expedition.
    pub fn population(&self) -> usize {
        self.agents.iter().filter(|a| a.is_alive()).count()
    }

    pub fn average_mood(&self) -> f32 {
        let present: Vec<f32> = self.present().map(|a| a.mood).collect();
        if present.is_empty() {
            0.0
        } else {
            present.iter().sum::<f32>() / present.len() as f32
        }
    }

    /// A free tile beside the hull for a new arrival.
    pub fn arrival_tile(&mut self) -> Tile {
        let landing = self.world.landing_tile;
        for _ in 0..30 {
            let tile = landing.offset(self.rng.range_i32(-5, 6), self.rng.range_i32(3, 6));
            if self.world.map.is_walkable(tile) {
                return tile;
            }
        }
        self.world
            .map
            .nearest_walkable(landing, 10)
            .unwrap_or(landing)
    }

    /// Create a generated survivor at a tile and return their id.
    pub fn spawn_survivor(&mut self, tile: Tile) -> AgentId {
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        let taken: Vec<String> = self.agents.iter().map(|a| a.given_name.clone()).collect();
        let arrival = Arrival {
            id,
            tile,
            day: self.calendar.day(),
            taken_names: &taken,
        };
        let mut agent = create_survivor(&arrival, &mut self.rng);
        agent.beliefs.stock = self.colony.stock;
        agent.next_deliberation = self.calendar.tick + id as u64 % 7;
        self.agents.push(agent);
        self.colony.stats.peak_population =
            self.colony.stats.peak_population.max(self.population());
        id
    }

    pub fn record(
        &mut self,
        category: Category,
        importance: u8,
        text: String,
        agents: Vec<AgentId>,
    ) {
        self.colony.chronicle.record(Entry {
            day: self.calendar.day(),
            hour: self.calendar.hour() as u8,
            category,
            text,
            agents,
            importance,
        });
    }

    /// Advance the simulation by one tick.
    pub fn step(&mut self) {
        if self.colony.campaign.is_over() {
            return;
        }
        self.calendar.tick += 1;
        if self.calendar.is_new_day() {
            daily::run(self);
        }
        let ticks_per_hour = (Calendar::ticks_per_day() / 24).max(1);
        if self.calendar.tick.is_multiple_of(ticks_per_hour) {
            environment::hourly(self);
        }
        if self.calendar.tick.is_multiple_of(10) {
            jobs::refresh(self);
        }
        environment::per_tick(self);
        creatures::update(self);
        self.update_agents();
        expeditions::update(self);
        if self.calendar.tick.is_multiple_of(30) {
            campaign::evaluate(self);
        }
    }

    fn update_agents(&mut self) {
        let snapshots: Vec<AgentSnapshot> = self.agents.iter().map(AgentSnapshot::of).collect();
        let mut effects = Effects::default();
        let Sim {
            world,
            agents,
            colony,
            calendar,
            rng,
            ..
        } = self;
        for agent in agents.iter_mut().filter(|a| a.is_present()) {
            let mut ctx = Ctx {
                world,
                colony,
                others: &snapshots,
                calendar: *calendar,
                rng,
                effects: &mut effects,
            };
            body::update(agent, &mut ctx);
            if !agent.is_present() {
                continue;
            }
            let interrupted = perception::perceive(agent, &mut ctx);
            let urgent_message = inbox::process(agent, &mut ctx);
            cognition::think(agent, &mut ctx, interrupted || urgent_message);
            execute::run_step(agent, &mut ctx);
            movement::advance(agent, &mut ctx);
        }
        social::apply_effects(self, effects);
        lifecycle::resolve_deaths(self);
    }

    /// Run many ticks, used by headless tests and the campaign runner.
    pub fn run_ticks(&mut self, ticks: u64) {
        for _ in 0..ticks {
            if self.colony.campaign.is_over() {
                break;
            }
            self.step();
        }
    }

    pub fn run_days(&mut self, days: u32) {
        self.run_ticks(days as u64 * Calendar::ticks_per_day());
    }
}
