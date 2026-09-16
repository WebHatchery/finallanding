//! time system domain.

use crate::systems::time_events::{TimeEvent, TimeEventCollector};

pub struct TimeSystem;

impl TimeSystem {
    pub fn ticks_per_day() -> u64 {
        crate::data::config::game_config().time.ticks_per_day
    }

    pub fn ticks_per_hour() -> u64 {
        crate::data::config::game_config().time.ticks_per_hour
    }

    pub fn get_time_of_day(tick: u64) -> (u32, u32, u32) {
        let ticks_per_day = Self::ticks_per_day();
        let ticks_per_hour = Self::ticks_per_hour();
        let day = (tick / ticks_per_day) as u32 + 1; // Day starts at 1
        let tick_in_day = tick % ticks_per_day;
        let hour = (tick_in_day / ticks_per_hour) as u32;
        let minute = (tick_in_day % ticks_per_hour) as u32;
        (day, hour, minute)
    }

    pub fn is_day(tick: u64) -> bool {
        let (_, hour, _) = Self::get_time_of_day(tick);
        (6..20).contains(&hour) // 6 AM to 8 PM is day
    }

    pub fn is_night(tick: u64) -> bool {
        !Self::is_day(tick)
    }

    /// Call this each frame to collect any time-based events that occurred
    /// between prev_tick and current_tick.
    pub fn collect_events(prev_tick: u64, current_tick: u64, collector: &mut TimeEventCollector) {
        if current_tick <= prev_tick {
            return;
        }

        let (mut last_day, mut last_hour, _) = Self::get_time_of_day(prev_tick);

        for tick in (prev_tick + 1)..=current_tick {
            let (day, hour, _) = Self::get_time_of_day(tick);

            if day > last_day {
                collector.push(TimeEvent::NewDay { day });
                last_day = day;
            }

            if hour != last_hour {
                collector.push(TimeEvent::HourChanged { hour });

                if hour == 6 {
                    collector.push(TimeEvent::DawnBreak);
                }

                if hour == 20 {
                    collector.push(TimeEvent::Dusk);
                }

                last_hour = hour;
            }
        }
    }
}
