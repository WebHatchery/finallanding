//! advisor system domain.

use crate::data::building::BuildingType;
use crate::data::priority::ColonyPriority;
use crate::data::resources::ColonyCondition;
use crate::state::runtime_state::GameState;
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
        let text = &crate::data::config::game_config().text;

        Self::add_incident_guidance(state, &mut lines);
        Self::add_pressure_warnings(state, &mut lines);
        Self::add_building_guidance(state, &mut lines);
        Self::add_progress_guidance(state, &mut lines);

        if lines.is_empty() {
            lines.push(AdvisorLine {
                title: text.label("advisor_hold_title").to_string(),
                detail: text.fill(
                    "advisor_hold_detail",
                    &[day.to_string(), state.scenario.target_day.to_string()],
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
        crate::data::config::game_config().text.fill(
            "advisor_headline",
            &[
                day.to_string(),
                state.scenario.target_day.to_string(),
                state.priority.active.label().to_string(),
            ],
        )
    }

    fn add_pressure_warnings(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        let text = &crate::data::config::game_config().text;
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        let summary = SummarySystem::colony_pressure_summary(state);

        if state.resources.condition == ColonyCondition::Critical
            || state.resources.condition == ColonyCondition::Collapsed
        {
            lines.push(AdvisorLine {
                title: text.label("advisor_stabilize_title").to_string(),
                detail: text.label("advisor_stabilize_detail").to_string(),
                severity: AdvisorSeverity::Warning,
            });
        } else if state.resources.supplies < daily_need * 2 {
            lines.push(AdvisorLine {
                title: text.label("advisor_supply_title").to_string(),
                detail: text.fill(
                    "advisor_supply_detail",
                    &[state.resources.supplies.to_string(), daily_need.to_string()],
                ),
                severity: AdvisorSeverity::Warning,
            });
        }

        if summary.average_mood < 35.0 {
            lines.push(AdvisorLine {
                title: text.label("advisor_recovery_title").to_string(),
                detail: text.label("advisor_recovery_detail").to_string(),
                severity: AdvisorSeverity::Warning,
            });
        } else if summary.strained_pairs > 1 {
            lines.push(AdvisorLine {
                title: text.label("advisor_strain_title").to_string(),
                detail: text.fill(
                    "advisor_strain_detail",
                    &[summary.strained_pairs.to_string()],
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
        let text = &crate::data::config::game_config().text;
        let habitat_capacity = ResourceSystem::habitat_capacity(state);
        if habitat_capacity < state.colonists.len() as u32 {
            lines.push(AdvisorLine {
                title: text.label("advisor_shelter_title").to_string(),
                detail: text.fill(
                    "advisor_shelter_detail",
                    &[
                        habitat_capacity.to_string(),
                        state.colonists.len().to_string(),
                    ],
                ),
                severity: AdvisorSeverity::Action,
            });
        }

        for (building_type, title_key, detail_key) in [
            (
                BuildingType::MessHall,
                "advisor_meal_title",
                "advisor_meal_detail",
            ),
            (
                BuildingType::Workshop,
                "advisor_workshop_title",
                "advisor_workshop_detail",
            ),
            (
                BuildingType::Storage,
                "advisor_storage_title",
                "advisor_storage_detail",
            ),
            (
                BuildingType::ExplorationGate,
                "advisor_gate_title",
                "advisor_gate_detail",
            ),
        ] {
            if Self::building_count(state, building_type) == 0 {
                lines.push(AdvisorLine {
                    title: text.label(title_key).to_string(),
                    detail: text.label(detail_key).to_string(),
                    severity: AdvisorSeverity::Action,
                });
            }
        }
    }

    fn add_progress_guidance(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        let text = &crate::data::config::game_config().text;
        if state.technology.unlocked_count() < state.scenario.required_tech_unlocks {
            let detail = if state.missions.active_count() > 0 {
                text.fill(
                    "advisor_tech_away",
                    &[
                        state.technology.unlocked_count().to_string(),
                        state.scenario.required_tech_unlocks.to_string(),
                    ],
                )
            } else if Self::building_count(state, BuildingType::ExplorationGate) > 0 {
                text.fill(
                    "advisor_tech_launch",
                    &[
                        state.technology.unlocked_count().to_string(),
                        state.scenario.required_tech_unlocks.to_string(),
                    ],
                )
            } else {
                text.label("advisor_tech_build").to_string()
            };

            lines.push(AdvisorLine {
                title: text.label("advisor_tech_title").to_string(),
                detail,
                severity: AdvisorSeverity::Action,
            });
        }

        if state.priority.active != ColonyPriority::Survey
            && state.technology.unlocked_count() < state.scenario.required_tech_unlocks
        {
            lines.push(AdvisorLine {
                title: text.label("advisor_survey_title").to_string(),
                detail: text.label("advisor_survey_detail").to_string(),
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
