//! assign domain.

use super::PageAction;
use super::{touch_target, touch_target_vertical};
pub use crate::data::assign_roster::{AssignRosterFilter, AssignRosterSort};
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, rect_contains_point, HitTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignBatchAction {
    PageHome,
    PageWork,
    AllHome,
    AllWork,
}

impl AssignBatchAction {
    pub fn all() -> &'static [AssignBatchAction] {
        &[
            AssignBatchAction::PageHome,
            AssignBatchAction::PageWork,
            AssignBatchAction::AllHome,
            AssignBatchAction::AllWork,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            AssignBatchAction::PageHome => "P-H",
            AssignBatchAction::PageWork => "P-W",
            AssignBatchAction::AllHome => "ALL-H",
            AssignBatchAction::AllWork => "ALL-W",
        }
    }

    pub fn tooltip_title(self) -> &'static str {
        match self {
            AssignBatchAction::PageHome => "Copy page home",
            AssignBatchAction::PageWork => "Copy page work",
            AssignBatchAction::AllHome => "Copy colony home",
            AssignBatchAction::AllWork => "Copy colony work",
        }
    }

    pub fn tooltip_body(self) -> &'static str {
        match self {
            AssignBatchAction::PageHome => {
                "Copy this survivor's Habitat pin to compatible survivors on the visible roster page."
            }
            AssignBatchAction::PageWork => {
                "Copy this survivor's work pin to compatible survivors on the visible roster page."
            }
            AssignBatchAction::AllHome => {
                "Copy this survivor's Habitat pin to every compatible survivor in the colony."
            }
            AssignBatchAction::AllWork => {
                "Copy this survivor's work pin to every compatible survivor in the colony."
            }
        }
    }

    pub fn copies_home(self) -> bool {
        matches!(
            self,
            AssignBatchAction::PageHome | AssignBatchAction::AllHome
        )
    }

    pub fn targets_all(self) -> bool {
        matches!(
            self,
            AssignBatchAction::AllHome | AssignBatchAction::AllWork
        )
    }
}

pub fn assign_page_previous_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 96.0, context.y + 12.0, 36.0, 30.0)
}

pub fn assign_page_next_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 40.0, context.y + 12.0, 36.0, 30.0)
}

pub fn assign_page_action_at(context: Rect, x: f32, y: f32) -> Option<PageAction> {
    hit_test(
        [
            HitTarget::new(
                touch_target(assign_page_previous_rect(context)),
                PageAction::Previous,
            ),
            HitTarget::new(
                touch_target(assign_page_next_rect(context)),
                PageAction::Next,
            ),
        ],
        vec2(x, y),
    )
}

pub fn assign_filter_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 72.0 + index as f32 * 50.0,
        context.y + 12.0,
        46.0,
        30.0,
    )
}

pub fn assign_filter_at(context: Rect, x: f32, y: f32) -> Option<AssignRosterFilter> {
    let point = vec2(x, y);
    hit_test(
        AssignRosterFilter::all()
            .iter()
            .enumerate()
            .map(|(index, filter)| {
                HitTarget::new(
                    touch_target_vertical(assign_filter_rect(context, index)),
                    *filter,
                )
            }),
        point,
    )
}

pub fn assign_sort_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 252.0 + index as f32 * 50.0,
        context.y + 12.0,
        46.0,
        30.0,
    )
}

pub fn assign_sort_at(context: Rect, x: f32, y: f32) -> Option<AssignRosterSort> {
    hit_test(
        AssignRosterSort::all()
            .iter()
            .enumerate()
            .map(|(index, sort)| {
                HitTarget::new(
                    touch_target_vertical(assign_sort_rect(context, index)),
                    *sort,
                )
            }),
        vec2(x, y),
    )
}

pub fn assign_role_filter_rect(context: Rect) -> Rect {
    Rect::new(context.x + 380.0, context.y + 12.0, 56.0, 30.0)
}

pub fn assign_role_filter_at(context: Rect, x: f32, y: f32) -> bool {
    rect_contains_point(
        touch_target_vertical(assign_role_filter_rect(context)),
        vec2(x, y),
    )
}

pub fn assign_room_filter_rect(context: Rect) -> Rect {
    Rect::new(context.x + 18.0, context.y + 128.0, 118.0, 30.0)
}

pub fn assign_role_action_rect(context: Rect) -> Rect {
    Rect::new(context.x + 18.0, context.y + 92.0, 108.0, 30.0)
}

pub fn assign_pair_action_rect(context: Rect) -> Rect {
    Rect::new(context.x + 134.0, context.y + 92.0, 132.0, 30.0)
}

pub fn assign_batch_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + context.w - 238.0 + index as f32 * 58.0,
        context.y + 128.0,
        54.0,
        30.0,
    )
}

pub fn assign_batch_action_at(context: Rect, x: f32, y: f32) -> Option<AssignBatchAction> {
    hit_test(
        AssignBatchAction::all()
            .iter()
            .enumerate()
            .map(|(index, action)| {
                HitTarget::new(
                    touch_target_vertical(assign_batch_rect(context, index)),
                    *action,
                )
            }),
        vec2(x, y),
    )
}
