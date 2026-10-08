//! launch domain.

use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState, JobPreference};
use crate::data::event_log::LogCategory;
use crate::data::mission::{ActiveMission, MissionType};
use crate::state::runtime_state::GameState;
use crate::systems::mission_system::planning::MissionPlanning;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchMissionError {
    NoExplorationGate,
    NoAvailableColonist,
    MissionCooldown { remaining_ticks: u64 },
}

pub struct MissionLaunch;

impl MissionLaunch {
    pub fn launch_mission(
        state: &mut GameState,
        mission_type: MissionType,
    ) -> Result<u32, LaunchMissionError> {
        if !state
            .building_system
            .buildings()
            .iter()
            .any(|building| building.building_type == BuildingType::ExplorationGate)
        {
            return Err(LaunchMissionError::NoExplorationGate);
        }

        let cooldown_remaining = state.missions.cooldown_remaining(state.tick);
        if cooldown_remaining > 0 {
            return Err(LaunchMissionError::MissionCooldown {
                remaining_ticks: cooldown_remaining,
            });
        }

        let Some(colonist_index) = Self::find_available_mission_colonist(state) else {
            return Err(LaunchMissionError::NoAvailableColonist);
        };

        let colonist_id = state.colonists[colonist_index].id;
        let started_tick = state.tick;
        let definition = mission_type.definition();
        let mission_id = state.missions.next_id;
        state.missions.next_id += 1;
        let danger_percent = MissionPlanning::mission_danger_percent(state, mission_type);
        let completes_at_tick = started_tick + definition.duration_minutes;
        let cooldown_minutes = definition
            .cooldown_minutes
            .saturating_sub(state.technology.mission_cooldown_reduction());
        let priority = state.priority.active;

        let colonist_name = state.colonists[colonist_index].name.clone();
        state.colonists[colonist_index].state = ColonistState::OnMission { mission_id };
        state.colonists[colonist_index].current_activity =
            crate::data::schedule::ActivityType::Work;
        state.colonists[colonist_index].activity_location = ActivityLocation::None;
        state.colonists[colonist_index].active_mission_id = Some(mission_id);

        state.missions.active_missions.push(ActiveMission {
            id: mission_id,
            colonist_id,
            mission_type,
            started_tick,
            completes_at_tick,
            danger_percent,
            priority,
        });
        state.missions.next_launch_tick = state.tick + cooldown_minutes;

        state.push_log(
            LogCategory::Mission,
            format!("{} started {}", colonist_name, definition.name),
            format!(
                "{} Duration {}m, danger {}% after {} priority. Crew regroups for {}m.",
                definition.reward_profile,
                definition.duration_minutes,
                danger_percent,
                priority.label(),
                cooldown_minutes
            ),
        );

        Ok(mission_id)
    }

    fn find_available_mission_colonist(state: &GameState) -> Option<usize> {
        state
            .colonists
            .iter()
            .position(|colonist| {
                colonist.job_preference == JobPreference::Explorer
                    && colonist.can_start_mission(state.tick)
            })
            .or_else(|| {
                state
                    .colonists
                    .iter()
                    .position(|colonist| colonist.can_start_mission(state.tick))
            })
    }
}
