//! log domain.

use super::PageAction;
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, HitTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSearchAction {
    Focus,
    Clear,
    Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogFilter {
    All,
    Tense,
    Support,
}

impl LogFilter {
    pub fn all() -> &'static [LogFilter] {
        &[LogFilter::All, LogFilter::Tense, LogFilter::Support]
    }

    pub fn label(self) -> &'static str {
        match self {
            LogFilter::All => "ALL",
            LogFilter::Tense => "TENSE",
            LogFilter::Support => "PLUS",
        }
    }
}

pub fn log_page_previous_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 96.0, context.y + 72.0, 28.0, 17.0)
}

pub fn log_page_next_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 34.0, context.y + 72.0, 28.0, 17.0)
}

pub fn log_page_action_at(context: Rect, x: f32, y: f32) -> Option<PageAction> {
    hit_test(
        [
            HitTarget::new(log_page_previous_rect(context), PageAction::Previous),
            HitTarget::new(log_page_next_rect(context), PageAction::Next),
        ],
        vec2(x, y),
    )
}

pub fn log_search_rect(context: Rect) -> Rect {
    Rect::new(context.x + 72.0, context.y + 13.0, 200.0, 17.0)
}

pub fn log_search_clear_rect(context: Rect) -> Rect {
    Rect::new(context.x + 278.0, context.y + 13.0, 42.0, 17.0)
}

pub fn log_search_export_rect(context: Rect) -> Rect {
    Rect::new(context.x + 326.0, context.y + 13.0, 46.0, 17.0)
}

pub fn log_search_action_at(context: Rect, x: f32, y: f32) -> Option<LogSearchAction> {
    hit_test(
        [
            HitTarget::new(log_search_rect(context), LogSearchAction::Focus),
            HitTarget::new(log_search_clear_rect(context), LogSearchAction::Clear),
            HitTarget::new(log_search_export_rect(context), LogSearchAction::Export),
        ],
        vec2(x, y),
    )
}

pub fn log_filter_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 120.0 + index as f32 * 50.0,
        context.y + 72.0,
        46.0,
        17.0,
    )
}

pub fn log_filter_at(context: Rect, x: f32, y: f32) -> Option<LogFilter> {
    hit_test(
        LogFilter::all()
            .iter()
            .enumerate()
            .map(|(index, filter)| HitTarget::new(log_filter_rect(context, index), *filter)),
        vec2(x, y),
    )
}

pub fn log_timeline_row_rect(context: Rect, index: usize) -> Rect {
    let y = context.y + 94.0 + index as f32 * 13.0;
    Rect::new(context.x + 12.0, y - 11.0, context.w - 24.0, 13.0)
}

pub fn log_timeline_row_at(context: Rect, row_count: usize, x: f32, y: f32) -> Option<usize> {
    hit_test(
        (0..row_count.min(3))
            .map(|index| HitTarget::new(log_timeline_row_rect(context, index), index)),
        vec2(x, y),
    )
}
