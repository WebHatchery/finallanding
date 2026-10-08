//! Weather, temperature, crops, power and wear on buildings.

use super::chronicle_text::line;
use super::Sim;
use crate::colony::chronicle::Category;
use crate::data::{game_data, Resource, Weather};
use crate::world::{Calendar, CropStage};

fn site_climate(sim: &Sim) -> &'static crate::data::campaign::Climate {
    let data = game_data();
    &data
        .site(&sim.colony.setup.site)
        .or_else(|| data.campaign.sites.first())
        .expect("campaign data has at least one landing site")
        .climate
}

/// Roll the next spell of weather when the current one ends.
fn roll_weather(sim: &mut Sim) {
    let climate = site_climate(sim);
    let season = sim.calendar.season().index();
    let roll = sim.rng.next_f32();
    let cold = climate.season_temperature[season] < 2.0;
    let weather = if roll < climate.storm_chance[season] {
        Weather::Storm
    } else if roll < climate.storm_chance[season] + climate.precipitation_chance[season] {
        if climate.ashfall && sim.rng.chance(0.3) {
            Weather::Ashfall
        } else if climate.snows && cold {
            Weather::Snow
        } else {
            Weather::Rain
        }
    } else if sim.rng.chance(0.35) {
        Weather::Overcast
    } else {
        Weather::Clear
    };
    let days = sim.rng.range_f32(0.3, 1.4);
    set_weather(sim, weather, days);
}

pub fn set_weather(sim: &mut Sim, weather: Weather, days: f32) {
    let previous = sim.world.weather.current;
    sim.world.weather.current = weather;
    sim.world.weather.until_tick = sim.calendar.tick + Calendar::ticks_for_days(days);
    if weather.is_severe() && !previous.is_severe() {
        let label = game_data().label(weather.key()).to_owned();
        let text = line("weather_severe", sim.calendar.tick, &[("weather", &label)]);
        sim.record(Category::Danger, 1, text, Vec::new());
    }
}

fn update_power(sim: &mut Sim) {
    let daylight = sim.calendar.daylight_level();
    let mut generated = 0.0;
    let mut capacity = 0.0;
    let mut fuel_needed = 0.0;
    for structure in sim.world.built() {
        let def = structure.def();
        if def.power <= 0.0 || structure.condition < 15.0 {
            continue;
        }
        capacity += def.power;
        if def.solar {
            generated += def.power * daylight;
        } else if def.fuel_per_day > 0.0 {
            fuel_needed += def.fuel_per_day / 24.0;
            if sim.colony.stock.get(Resource::Fibre) >= def.fuel_per_day / 24.0 {
                generated += def.power;
            }
        } else {
            generated += def.power;
        }
    }
    sim.colony.stock.take(Resource::Fibre, fuel_needed);
    let mut demand = 0.0;
    let mut remaining = generated;
    for structure in sim.world.structures.iter_mut().filter(|s| s.is_built()) {
        let use_power = -structure.def().power;
        if use_power <= 0.0 {
            continue;
        }
        demand += use_power;
        structure.powered = remaining >= use_power;
        if structure.powered {
            remaining -= use_power;
        }
    }
    sim.colony.power.generated = generated;
    sim.colony.power.capacity = capacity;
    sim.colony.power.demand = demand;
}

pub fn hourly(sim: &mut Sim) {
    if sim.calendar.tick >= sim.world.weather.until_tick {
        roll_weather(sim);
    }
    let climate = site_climate(sim);
    let season = sim.calendar.season();
    let daylight = sim.calendar.daylight_level();
    sim.world.weather.temperature = sim
        .world
        .weather
        .compute_temperature(climate, season, daylight);
    update_power(sim);
    if sim.world.weather.is_storm() {
        let damage = game_data().balance.work.storm_damage / 24.0;
        for structure in sim
            .world
            .structures
            .iter_mut()
            .filter(|s| s.is_built() && !s.def().indoor)
        {
            structure.condition = (structure.condition - damage).max(0.0);
        }
    }
}

fn kill_crops_in_frost(sim: &mut Sim) {
    let frost = game_data().balance.farming.frost_kill_temperature;
    if sim.world.weather.temperature >= frost {
        return;
    }
    let mut lost = 0;
    for structure in sim.world.structures.iter_mut() {
        let indoor = structure.def().farm.as_ref().is_some_and(|f| f.indoor);
        if let Some(crop) = structure.crop.as_mut() {
            if !indoor && crop.stage == CropStage::Growing && crop.growth > 0.05 {
                crop.stage = CropStage::Fallow;
                crop.growth = 0.0;
                lost += 1;
            }
        }
    }
    if lost > 0 {
        let text = line(
            "frost_kill",
            sim.calendar.tick,
            &[("count", &lost.to_string())],
        );
        sim.record(Category::Danger, 1, text, Vec::new());
    }
}

pub fn per_tick(sim: &mut Sim) {
    let season = sim.calendar.season().index();
    let season_growth = game_data().balance.farming.season_growth[season];
    let ticks_per_day = Calendar::ticks_per_day() as f32;
    for structure in sim.world.structures.iter_mut().filter(|s| s.is_built()) {
        let Some(spec) = structure.def().farm.as_ref() else {
            continue;
        };
        let operational = structure.is_operational();
        let grow_days = spec.grow_days * structure.crop_traits().grow_scale;
        let Some(crop) = structure.crop.as_mut() else {
            continue;
        };
        if crop.stage != CropStage::Growing {
            continue;
        }
        let rate = if spec.indoor {
            if operational {
                1.0
            } else {
                0.0
            }
        } else {
            season_growth
        };
        crop.growth += rate / (grow_days * ticks_per_day);
        if crop.growth >= 1.0 {
            crop.stage = CropStage::Ripe;
        }
    }
    if sim.calendar.tick.is_multiple_of(60) {
        kill_crops_in_frost(sim);
    }
}

/// Daily wear, spoilage, and regrowth of living landscape features.
pub fn daily(sim: &mut Sim) {
    let work = &game_data().balance.work;
    let decay = work.decay_per_day;
    for (resource, rate) in [
        (Resource::Food, work.food_spoilage_per_day),
        (Resource::Meals, work.meal_spoilage_per_day),
    ] {
        let spoiled = sim.colony.stock.get(resource) * rate;
        sim.colony.stock.take(resource, spoiled);
    }
    for structure in sim.world.structures.iter_mut().filter(|s| s.is_built()) {
        structure.condition = (structure.condition - decay).max(0.0);
    }
    let regrow = game_data().balance.map.node_regrow_per_day;
    for node in sim.world.nodes.iter_mut().filter(|n| n.regrows()) {
        node.amount = (node.amount + node.max_amount * regrow).min(node.max_amount);
    }
}
