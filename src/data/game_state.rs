//! Reusable colony data and clock values, independent of runtime orchestration.

use super::grid::Grid;
use crate::data::event_log::{ColonyLogEntry, LogCategory, SocialHistoryEntry};
use crate::data::incident::IncidentState;
use crate::data::mission::MissionState;
use crate::data::priority::PriorityState;
use crate::data::resources::ResourceState;
use crate::data::scenario::ScenarioState;
use crate::data::technology::TechnologyState;
use serde::{Deserialize, Serialize};

const MAX_EVENT_LOG_ENTRIES: usize = 80;
const MAX_SOCIAL_HISTORY_ENTRIES: usize = 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSpeed {
    Paused = 0,
    Normal = 1,
    Fast = 2,
    SuperFast = 4,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeState {
    pub speed: TimeSpeed,
    pub day_length_ticks: u64,
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            speed: TimeSpeed::Normal,
            day_length_ticks: crate::data::config::game_config().time.ticks_per_day,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ColonyData {
    pub tick: u64,
    pub time: TimeState,
    pub grid: Grid,

    pub colonists: Vec<crate::data::colonist::Colonist>,
    pub event_log: Vec<ColonyLogEntry>,
    pub social_history: Vec<SocialHistoryEntry>,
    pub resources: ResourceState,
    pub missions: MissionState,
    pub incidents: IncidentState,
    pub priority: PriorityState,
    pub technology: TechnologyState,
    pub scenario: ScenarioState,
}

impl ColonyData {
    pub fn new() -> Self {
        Self {
            tick: 0,
            time: TimeState::default(),
            grid: Grid::default(),

            colonists: Vec::new(),
            event_log: Vec::new(),
            social_history: Vec::new(),
            resources: ResourceState::default(),
            missions: MissionState::default(),
            incidents: IncidentState::default(),
            priority: PriorityState::default(),
            technology: TechnologyState::default(),
            scenario: ScenarioState::default(),
        }
    }

    pub fn push_log(
        &mut self,
        category: LogCategory,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) {
        let ticks_per_day = crate::data::config::game_config().time.ticks_per_day;
        let ticks_per_hour = crate::data::config::game_config().time.ticks_per_hour;
        let day = (self.tick / ticks_per_day) as u32 + 1;
        let tick_in_day = self.tick % ticks_per_day;
        let hour = (tick_in_day / ticks_per_hour) as u32;
        let minute = (tick_in_day % ticks_per_hour) as u32;
        self.event_log.push(ColonyLogEntry::new(
            day,
            hour,
            minute,
            category,
            title.into(),
            detail.into(),
        ));

        if self.event_log.len() > MAX_EVENT_LOG_ENTRIES {
            let overflow = self.event_log.len() - MAX_EVENT_LOG_ENTRIES;
            self.event_log.drain(0..overflow);
        }
    }

    pub fn push_social_history(&mut self, entry: SocialHistoryEntry) {
        self.social_history.push(entry);

        if self.social_history.len() > MAX_SOCIAL_HISTORY_ENTRIES {
            let overflow = self.social_history.len() - MAX_SOCIAL_HISTORY_ENTRIES;
            self.social_history.drain(0..overflow);
        }
    }
}
