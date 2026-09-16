//! planning system domain.

use crate::data::building::BuildingType;
use crate::data::types::Position;
use crate::state::runtime_state::GameState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildingPlacementFeedback {
    pub building_type: BuildingType,
    pub footprint: (u32, u32),
    pub cost: i32,
    pub purpose: &'static str,
    pub helps: &'static str,
    pub impact: &'static str,
    pub invalid_reason: Option<String>,
}

impl BuildingPlacementFeedback {
    pub fn can_place(&self) -> bool {
        self.invalid_reason.is_none()
    }
}

pub struct PlanningSystem;

impl PlanningSystem {
    pub fn building_feedback(
        state: &GameState,
        building_type: BuildingType,
        position: Position,
    ) -> BuildingPlacementFeedback {
        let footprint = building_type.size();
        let invalid_reason = Self::invalid_reason(state, building_type, position);

        BuildingPlacementFeedback {
            building_type,
            footprint,
            cost: building_type.salvage_cost(),
            purpose: building_type.purpose(),
            helps: building_type.planning_role(),
            impact: building_type.placement_impact(),
            invalid_reason,
        }
    }

    pub fn invalid_reason(
        state: &GameState,
        building_type: BuildingType,
        position: Position,
    ) -> Option<String> {
        if state.resources.salvage < building_type.salvage_cost() {
            return Some(format!(
                "Need {} salvage; {} available.",
                building_type.salvage_cost(),
                state.resources.salvage
            ));
        }

        let (width, height) = building_type.size();
        if position.x < 0 || position.y < 0 {
            return Some("Footprint starts outside the map.".to_string());
        }

        if position.x as u32 + width > state.grid.width as u32
            || position.y as u32 + height > state.grid.height as u32
        {
            return Some("Footprint leaves the map.".to_string());
        }

        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                let check = Position::new(position.x + dx, position.y + dy);
                let Some(cell) = state.grid.get_cell(check.x, check.y) else {
                    return Some("Footprint leaves the map.".to_string());
                };

                if cell.building_id.is_some() {
                    return Some("Footprint overlaps another building.".to_string());
                }

                if !cell.is_walkable() {
                    return Some("Footprint overlaps blocked terrain.".to_string());
                }
            }
        }

        None
    }

    pub fn placement_log_detail(
        feedback: &BuildingPlacementFeedback,
        building_id: u32,
        remaining_salvage: i32,
    ) -> String {
        format!(
            "Added {} support: {} Building #{} cost {} salvage; {} remain.",
            feedback.helps, feedback.impact, building_id, feedback.cost, remaining_salvage
        )
    }
}
