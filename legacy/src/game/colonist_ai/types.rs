//! types domain.

use crate::data::building::BuildingType;
use crate::data::colonist::ActivityLocation;
use crate::data::event_log::LogCategory;
use crate::data::types::Position;

pub type PendingLog = (LogCategory, String, String);
pub type SocialLocation = (u32, ActivityLocation);
pub type BuildingSnapshot = (u32, BuildingType, Position, (u32, u32));

#[derive(Clone, Copy, Debug)]
pub struct BuildingTarget {
    pub building_id: u32,
    pub building_type: BuildingType,
    pub entrance: Position,
}
