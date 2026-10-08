//! Simulation time: ticks, days, hours, seasons and years.

use crate::data::{game_data, Season};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub tick: u64,
}

impl Calendar {
    pub fn ticks_per_day() -> u64 {
        game_data().balance.time.ticks_per_day as u64
    }

    fn days_per_season() -> u64 {
        game_data().balance.time.days_per_season as u64
    }

    pub fn days_per_year() -> u64 {
        Self::days_per_season() * 4
    }

    /// Day number, starting at 1 on landing.
    pub fn day(&self) -> u32 {
        (self.tick / Self::ticks_per_day()) as u32 + 1
    }

    pub fn tick_of_day(&self) -> u64 {
        self.tick % Self::ticks_per_day()
    }

    /// Fractional hour of day in [0, 24).
    pub fn hour(&self) -> f32 {
        self.tick_of_day() as f32 / Self::ticks_per_day() as f32 * 24.0
    }

    pub fn minute_of_hour(&self) -> u32 {
        ((self.hour().fract()) * 60.0) as u32
    }

    pub fn season(&self) -> Season {
        let index = ((self.day() as u64 - 1) / Self::days_per_season()) % 4;
        Season::ALL[index as usize]
    }

    pub fn day_of_season(&self) -> u32 {
        ((self.day() as u64 - 1) % Self::days_per_season()) as u32 + 1
    }

    pub fn year(&self) -> u32 {
        ((self.day() as u64 - 1) / Self::days_per_year()) as u32 + 1
    }

    pub fn is_new_day(&self) -> bool {
        self.tick_of_day() == 0
    }

    pub fn is_daylight(&self) -> bool {
        let time = &game_data().balance.time;
        let hour = self.hour();
        hour >= time.dawn_hour && hour < time.dusk_hour
    }

    /// 0 at midnight, 1 at noon; used for lighting.
    pub fn daylight_level(&self) -> f32 {
        let time = &game_data().balance.time;
        let hour = self.hour();
        if hour < time.dawn_hour - 1.0 || hour > time.dusk_hour + 1.0 {
            0.0
        } else if hour < time.dawn_hour + 1.0 {
            (hour - (time.dawn_hour - 1.0)) / 2.0
        } else if hour > time.dusk_hour - 1.0 {
            1.0 - (hour - (time.dusk_hour - 1.0)) / 2.0
        } else {
            1.0
        }
    }

    pub fn ticks_for_days(days: f32) -> u64 {
        (days * Self::ticks_per_day() as f32).round().max(1.0) as u64
    }

    pub fn per_tick(per_day: f32) -> f32 {
        per_day / Self::ticks_per_day() as f32
    }
}
