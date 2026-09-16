//! objective system domain.

use crate::data::building::BuildingType;
use crate::data::resources::ColonyCondition;
use crate::state::runtime_state::GameState;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::scenario_system::ScenarioSystem;
use crate::systems::time_system::TimeSystem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveStatus {
    Complete,
    Active,
    AtRisk,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveCard {
    pub title: String,
    pub detail: String,
    pub progress: f32,
    pub status: ObjectiveStatus,
}

pub struct ObjectiveSystem;

impl ObjectiveSystem {
    pub fn active_cards(state: &GameState) -> Vec<ObjectiveCard> {
        let mut cards = vec![
            Self::landing_card(state),
            Self::shelter_card(state),
            Self::food_card(state),
            Self::core_rooms_card(state),
            Self::technology_card(state),
        ];

        cards.sort_by_key(|card| match card.status {
            ObjectiveStatus::AtRisk => 0,
            ObjectiveStatus::Active => 1,
            ObjectiveStatus::Complete => 2,
        });
        cards.truncate(4);
        cards
    }

    fn landing_card(state: &GameState) -> ObjectiveCard {
        let (day, _, _) = TimeSystem::get_time_of_day(state.tick);
        let target_day = state.scenario.target_day.max(1);
        let progress = day as f32 / target_day as f32;
        let status = if ScenarioSystem::meets_victory_requirements(state) {
            ObjectiveStatus::Complete
        } else if state.resources.condition == ColonyCondition::Critical
            || state.resources.condition == ColonyCondition::Collapsed
        {
            ObjectiveStatus::AtRisk
        } else {
            ObjectiveStatus::Active
        };

        ObjectiveCard {
            title: "Secure stable landing".to_string(),
            detail: format!(
                "Day {} of {} | {}",
                day,
                target_day,
                state.resources.condition.label()
            ),
            progress,
            status,
        }
    }

    fn shelter_card(state: &GameState) -> ObjectiveCard {
        let needed = state.colonists.len().max(1) as u32;
        let capacity = ResourceSystem::habitat_capacity(state);
        let progress = capacity as f32 / needed as f32;
        let complete = capacity >= state.colonists.len() as u32;

        ObjectiveCard {
            title: "Shelter every survivor".to_string(),
            detail: format!("{} beds for {} colonists", capacity, state.colonists.len()),
            progress,
            status: if complete {
                ObjectiveStatus::Complete
            } else {
                ObjectiveStatus::AtRisk
            },
        }
    }

    fn food_card(state: &GameState) -> ObjectiveCard {
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        let target_buffer = (daily_need * 2).max(1);
        let progress = state.resources.supplies as f32 / target_buffer as f32;
        let status = if state.resources.supplies >= target_buffer {
            ObjectiveStatus::Complete
        } else if state.resources.supplies < daily_need {
            ObjectiveStatus::AtRisk
        } else {
            ObjectiveStatus::Active
        };

        ObjectiveCard {
            title: "Hold a food buffer".to_string(),
            detail: format!(
                "{} food vs {} daily need",
                state.resources.supplies, daily_need
            ),
            progress,
            status,
        }
    }

    pub fn core_rooms_card(state: &GameState) -> ObjectiveCard {
        let placed = BuildingType::all()
            .iter()
            .filter(|building_type| Self::has_building(state, **building_type))
            .count();
        let total = BuildingType::all().len();

        ObjectiveCard {
            title: "Establish core rooms".to_string(),
            detail: format!("{} of {} room types placed", placed, total),
            progress: placed as f32 / total as f32,
            status: if placed == total {
                ObjectiveStatus::Complete
            } else {
                ObjectiveStatus::Active
            },
        }
    }

    pub fn technology_card(state: &GameState) -> ObjectiveCard {
        let required = state.scenario.required_tech_unlocks.max(1);
        let unlocked = state.technology.unlocked_count();
        let has_gate = Self::has_building(state, BuildingType::ExplorationGate);

        ObjectiveCard {
            title: "Recover field technology".to_string(),
            detail: format!("{} of {} tech unlocked", unlocked, required),
            progress: unlocked as f32 / required as f32,
            status: if unlocked >= required {
                ObjectiveStatus::Complete
            } else if !has_gate {
                ObjectiveStatus::AtRisk
            } else {
                ObjectiveStatus::Active
            },
        }
    }

    fn has_building(state: &GameState, building_type: BuildingType) -> bool {
        state
            .building_system
            .buildings()
            .iter()
            .any(|building| building.building_type == building_type)
    }
}
