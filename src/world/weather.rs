//! The current weather and outdoor temperature.

use crate::data::campaign::Climate;
use crate::data::{Season, Weather};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherState {
    pub current: Weather,
    pub until_tick: u64,
    pub temperature: f32,
    pub daily_offset: f32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current: Weather::Clear,
            until_tick: 0,
            temperature: 12.0,
            daily_offset: 0.0,
        }
    }
}

impl WeatherState {
    /// Outdoor temperature from season, time of day and the current weather.
    pub fn compute_temperature(&self, climate: &Climate, season: Season, daylight: f32) -> f32 {
        let base = climate.season_temperature[season.index()];
        let diurnal = (daylight - 0.5) * climate.temperature_swing;
        let weather = match self.current {
            Weather::Clear => 1.0,
            Weather::Overcast => -1.0,
            Weather::Rain => -3.0,
            Weather::Storm => -5.0,
            Weather::Snow => -9.0,
            Weather::Heatwave => 9.0,
            Weather::Ashfall => -2.0,
        };
        base + diurnal + weather + self.daily_offset
    }

    pub fn is_storm(&self) -> bool {
        self.current.is_severe()
    }
}
