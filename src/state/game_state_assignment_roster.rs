//! game state assignment roster domain.

pub use crate::data::assign_roster::{assign_roster_page_count, assign_visible_colonist_indices};
use crate::data::colonist::JobPreference;

pub fn next_assign_role_filter(current: Option<JobPreference>) -> Option<JobPreference> {
    match current {
        None => Some(JobPreference::Explorer),
        Some(JobPreference::Explorer) => Some(JobPreference::Builder),
        Some(JobPreference::Builder) => Some(JobPreference::Cook),
        Some(JobPreference::Cook) => Some(JobPreference::Hauler),
        Some(JobPreference::Hauler) | Some(JobPreference::None) => None,
    }
}
