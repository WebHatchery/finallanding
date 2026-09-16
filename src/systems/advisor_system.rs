//! advisor system domain.

use crate::data::building::BuildingType;
use crate::data::game_state::GameState;
use crate::data::priority::ColonyPriority;
use crate::data::resources::ColonyCondition;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_system::TimeSystem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvisorSeverity {
    Stable,
    Action,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvisorLine {
    pub title: String,
    pub detail: String,
    pub severity: AdvisorSeverity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvisorPlan {
    pub headline: String,
    pub lines: Vec<AdvisorLine>,
}

pub struct AdvisorSystem;

impl AdvisorSystem {
    pub fn plan(state: &GameState) -> AdvisorPlan {
        let mut lines = Vec::new();
        let (day, _, _) = TimeSystem::get_time_of_day(state.tick);

        Self::add_incident_guidance(state, &mut lines);
        Self::add_pressure_warnings(state, &mut lines);
        Self::add_building_guidance(state, &mut lines);
        Self::add_progress_guidance(state, &mut lines);

        if lines.is_empty() {
            lines.push(AdvisorLine {
                title: "Hold the landing site".to_string(),
                detail: format!(
                    "Day {} of {}; keep supplies above daily need and relationships stable.",
                    day, state.scenario.target_day
                ),
                severity: AdvisorSeverity::Stable,
            });
        }

        AdvisorPlan {
            headline: Self::headline(state, day),
            lines,
        }
    }

    fn headline(state: &GameState, day: u32) -> String {
        format!(
            "Advisor | Day {}/{} | {}",
            day,
            state.scenario.target_day,
            state.priority.active.label()
        )
    }

    fn add_pressure_warnings(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        let summary = SummarySystem::colony_pressure_summary(state);

        if state.resources.condition == ColonyCondition::Critical
            || state.resources.condition == ColonyCondition::Collapsed
        {
            lines.push(AdvisorLine {
                title: "Stabilize the colony".to_string(),
                detail: "Prioritize food, recovery space, and safer mission timing.".to_string(),
                severity: AdvisorSeverity::Warning,
            });
        } else if state.resources.supplies < daily_need * 2 {
            lines.push(AdvisorLine {
                title: "Raise the supply buffer".to_string(),
                detail: format!(
                    "{} supplies against {} daily need is a thin reserve.",
                    state.resources.supplies, daily_need
                ),
                severity: AdvisorSeverity::Warning,
            });
        }

        if summary.average_mood < 35.0 {
            lines.push(AdvisorLine {
                title: "Give people recovery time".to_string(),
                detail: "Low mood increases refusals and can push the colony critical.".to_string(),
                severity: AdvisorSeverity::Warning,
            });
        } else if summary.strained_pairs > 1 {
            lines.push(AdvisorLine {
                title: "Ease social strain".to_string(),
                detail: format!(
                    "{} tense pairs are adding pressure to daily work.",
                    summary.strained_pairs
                ),
                severity: AdvisorSeverity::Action,
            });
        }
    }

    fn add_incident_guidance(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        if let Some(incident_type) = state.incidents.active_incident(state.tick) {
            lines.push(AdvisorLine {
                title: incident_type.advisor_title().to_string(),
                detail: incident_type.advisor_detail().to_string(),
                severity: AdvisorSeverity::Warning,
            });
        }
    }

    fn add_building_guidance(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        let habitat_capacity = ResourceSystem::habitat_capacity(state);
        if habitat_capacity < state.colonists.len() as u32 {
            lines.push(AdvisorLine {
                title: "Shelter every survivor".to_string(),
                detail: format!(
                    "{} sleeper slots for {} colonists.",
                    habitat_capacity,
                    state.colonists.len()
                ),
                severity: AdvisorSeverity::Action,
            });
        }

        for (building_type, title, detail) in [
            (
                BuildingType::MessHall,
                "Open a meal point",
                "Cook labor creates meals that reduce the daily supply draw.",
            ),
            (
                BuildingType::Workshop,
                "Recover repair stock",
                "Builder labor turns wreckage into usable salvage.",
            ),
            (
                BuildingType::Storage,
                "Secure storage",
                "Storage raises the supply cap before survey finds overflow.",
            ),
            (
                BuildingType::ExplorationGate,
                "Mark a survey route",
                "Survey missions bring tech items and emergency resources.",
            ),
        ] {
            if Self::building_count(state, building_type) == 0 {
                lines.push(AdvisorLine {
                    title: title.to_string(),
                    detail: detail.to_string(),
                    severity: AdvisorSeverity::Action,
                });
            }
        }
    }

    fn add_progress_guidance(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        if state.technology.unlocked_count() < state.scenario.required_tech_unlocks {
            let detail = if state.missions.active_count() > 0 {
                format!(
                    "Survey team away; tech progress {}/{}.",
                    state.technology.unlocked_count(),
                    state.scenario.required_tech_unlocks
                )
            } else if Self::building_count(state, BuildingType::ExplorationGate) > 0 {
                format!(
                    "Launch scans until tech reaches {}/{}.",
                    state.technology.unlocked_count(),
                    state.scenario.required_tech_unlocks
                )
            } else {
                "Build an Exploration Gate before Day 7 tech falls behind.".to_string()
            };

            lines.push(AdvisorLine {
                title: "Push toward field tech".to_string(),
                detail,
                severity: AdvisorSeverity::Action,
            });
        }

        if state.priority.active != ColonyPriority::Survey
            && state.technology.unlocked_count() < state.scenario.required_tech_unlocks
        {
            lines.push(AdvisorLine {
                title: "Use Survey priority".to_string(),
                detail: "Survey boosts exploration output and research-item returns.".to_string(),
                severity: AdvisorSeverity::Action,
            });
        }
    }

    fn building_count(state: &GameState, building_type: BuildingType) -> usize {
        state
            .building_system
            .buildings()
            .iter()
            .filter(|building| building.building_type == building_type)
            .count()
    }
}
