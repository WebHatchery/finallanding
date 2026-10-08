//! The event director: draws from the event deck at a pace set by difficulty.

use super::event_effects::fire_event;
use super::Sim;
use crate::data::events::EventDef;
use crate::data::game_data;

fn is_eligible(sim: &Sim, event: &EventDef) -> bool {
    let day = sim.calendar.day();
    let colony = &sim.colony;
    if event.scripted || event.weight <= 0.0 || day < event.min_day {
        return false;
    }
    if event.max_day.is_some_and(|max| day > max) {
        return false;
    }
    if event.once && colony.events_fired.contains(&event.id) {
        return false;
    }
    if colony
        .event_cooldowns
        .get(&event.id)
        .is_some_and(|until| day < *until)
    {
        return false;
    }
    if !event.acts.is_empty() && !event.acts.contains(&colony.campaign.act) {
        return false;
    }
    if !event.seasons.is_empty() && !event.seasons.contains(&sim.calendar.season()) {
        return false;
    }
    if !event.sites.is_empty() && !event.sites.contains(&colony.setup.site) {
        return false;
    }
    sim.population() >= event.min_population
}

fn draw(sim: &mut Sim) -> Option<String> {
    let deck: Vec<(String, f32)> = game_data()
        .events
        .iter()
        .filter(|e| is_eligible(sim, e))
        .map(|e| (e.id.clone(), e.weight))
        .collect();
    let total: f32 = deck.iter().map(|(_, w)| w).sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll = sim.rng.next_f32() * total;
    for (id, weight) in deck {
        if roll < weight {
            return Some(id);
        }
        roll -= weight;
    }
    None
}

pub fn daily(sim: &mut Sim) {
    let day = sim.calendar.day();
    if day < sim.colony.next_event_day {
        return;
    }
    if let Some(id) = draw(sim) {
        fire_event(sim, &id);
    }
    let pressure = sim.colony.difficulty_value(|d| d.event_pressure).max(0.2);
    let gap = (sim.rng.range_f32(2.5, 6.5) / pressure).round().max(1.0) as u32;
    sim.colony.next_event_day = day + gap;
}
