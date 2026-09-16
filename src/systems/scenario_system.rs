//! scenario system domain.

use crate::data::event_log::LogCategory;
use crate::data::game_state::TimeSpeed;
use crate::data::resources::ColonyCondition;
use crate::data::scenario::ScenarioOutcome;
use crate::state::runtime_state::GameState;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::time_system::TimeSystem;

pub struct ScenarioSystem;

impl ScenarioSystem {
    pub fn evaluate(state: &mut GameState) {
        if state.scenario.is_finished() {
            return;
        }

        let (day, _, _) = TimeSystem::get_time_of_day(state.tick);

        if state.resources.condition == ColonyCondition::Collapsed {
            Self::finish(
                state,
                ScenarioOutcome::Failure,
                "Colony failed",
                "The settlement collapsed before a stable landing site could form.",
            );
            return;
        }

        if day >= state.scenario.target_day && Self::meets_victory_requirements(state) {
            Self::finish(
                state,
                ScenarioOutcome::Victory,
                "Stable landing secured",
                "The colony survived the first week with enough infrastructure, knowledge, and supplies to continue.",
            );
        } else if day >= state.scenario.target_day {
            Self::finish(
                state,
                ScenarioOutcome::Failure,
                "Landing site unstable",
                "The colony reached Day 7 without the supplies, condition, or field technology needed to hold the site.",
            );
        }
    }

    pub fn meets_victory_requirements(state: &GameState) -> bool {
        state.resources.condition == ColonyCondition::Stable
            && state.resources.supplies >= ResourceSystem::daily_supply_need(state).max(1)
            && state.technology.unlocked_count() >= state.scenario.required_tech_unlocks
    }

    pub fn objective_line(state: &GameState) -> String {
        let tech_count = state.technology.unlocked_count();
        let tech_required = state.scenario.required_tech_unlocks;
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        format!(
            "Survive to Day {} | Tech {}/{} | Supplies {}/{} | {}",
            state.scenario.target_day,
            tech_count,
            tech_required,
            state.resources.supplies,
            daily_need,
            state.resources.condition.label()
        )
    }

    pub fn estimated_real_minutes_to_target(state: &GameState, seconds_per_tick: f32) -> f32 {
        let target_tick =
            state.scenario.target_day.saturating_sub(1) as u64 * TimeSystem::ticks_per_day();
        target_tick.saturating_sub(state.tick) as f32 * seconds_per_tick / 60.0
    }

    fn finish(
        state: &mut GameState,
        outcome: ScenarioOutcome,
        title: &'static str,
        detail: &'static str,
    ) {
        state.scenario.outcome = outcome;
        state.scenario.outcome_tick = Some(state.tick);
        state.time.speed = TimeSpeed::Paused;
        state.push_log(LogCategory::Colony, title, detail);
    }
}
