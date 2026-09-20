//! log domain.

use super::PageAction;
use super::{touch_target, touch_target_vertical};
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, HitTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSearchAction {
    Focus,
    Clear,
    Export,
    Key(char),
    Backspace,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSectionAction {
    Social,
    Events,
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
        let key = match self {
            LogFilter::All => "log_filter_all",
            LogFilter::Tense => "log_filter_tense",
            LogFilter::Support => "log_filter_support",
        };
        crate::data::config::game_config().text.label(key)
    }
}

pub fn log_page_previous_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 96.0, context.y + 56.0, 36.0, 30.0)
}

pub fn log_page_next_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 40.0, context.y + 56.0, 36.0, 30.0)
}

pub fn log_page_action_at(context: Rect, x: f32, y: f32) -> Option<PageAction> {
    hit_test(
        [
            HitTarget::new(
                touch_target(log_page_previous_rect(context)),
                PageAction::Previous,
            ),
            HitTarget::new(touch_target(log_page_next_rect(context)), PageAction::Next),
        ],
        vec2(x, y),
    )
}

pub fn log_search_rect(context: Rect) -> Rect {
    Rect::new(context.x + 72.0, context.y + 12.0, 300.0, 32.0)
}

pub fn log_search_clear_rect(context: Rect) -> Rect {
    Rect::new(context.x + 380.0, context.y + 12.0, 64.0, 32.0)
}

pub fn log_search_export_rect(context: Rect) -> Rect {
    Rect::new(context.x + 452.0, context.y + 12.0, 64.0, 32.0)
}

pub fn log_section_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 18.0 + index as f32 * 68.0,
        context.y + 56.0,
        62.0,
        30.0,
    )
}

pub fn log_section_at(context: Rect, x: f32, y: f32) -> Option<LogSectionAction> {
    hit_test(
        [LogSectionAction::Social, LogSectionAction::Events]
            .into_iter()
            .enumerate()
            .map(|(index, section)| {
                HitTarget::new(
                    touch_target_vertical(log_section_rect(context, index)),
                    section,
                )
            }),
        vec2(x, y),
    )
}

pub fn log_search_action_at(context: Rect, x: f32, y: f32) -> Option<LogSearchAction> {
    hit_test(
        [
            HitTarget::new(
                touch_target_vertical(log_search_rect(context)),
                LogSearchAction::Focus,
            ),
            HitTarget::new(
                touch_target_vertical(log_search_clear_rect(context)),
                LogSearchAction::Clear,
            ),
            HitTarget::new(
                touch_target_vertical(log_search_export_rect(context)),
                LogSearchAction::Export,
            ),
        ],
        vec2(x, y),
    )
}

pub fn log_keyboard_bounds(context: Rect) -> Rect {
    Rect::new(context.x, context.y - 190.0, context.w, 190.0)
}

pub fn log_keyboard_key_rect(context: Rect, index: usize) -> Rect {
    let columns = 7.0;
    let gap = 4.0;
    let width = (context.w - 24.0 - gap * (columns - 1.0)) / columns;
    let row = index / 7;
    let col = index % 7;
    Rect::new(
        context.x + 12.0 + col as f32 * (width + gap),
        context.y - 184.0 + row as f32 * 45.0,
        width,
        41.0,
    )
}

pub fn log_keyboard_action_at(context: Rect, x: f32, y: f32) -> Option<LogSearchAction> {
    let keys: &[char] = &[
        'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
        'L', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', ' ',
    ];
    for (index, key) in keys.iter().enumerate() {
        if log_keyboard_key_rect(context, index).contains(vec2(x, y)) {
            return Some(LogSearchAction::Key(*key));
        }
    }
    let backspace = Rect::new(context.x + 12.0, context.y - 3.0, context.w * 0.48, 41.0);
    let done = Rect::new(
        context.x + context.w * 0.52,
        context.y - 3.0,
        context.w * 0.48 - 12.0,
        41.0,
    );
    if backspace.contains(vec2(x, y)) {
        Some(LogSearchAction::Backspace)
    } else if done.contains(vec2(x, y)) {
        Some(LogSearchAction::Done)
    } else {
        None
    }
}

pub fn log_filter_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 164.0 + index as f32 * 68.0,
        context.y + 122.0,
        62.0,
        30.0,
    )
}

pub fn log_filter_at(context: Rect, x: f32, y: f32) -> Option<LogFilter> {
    hit_test(
        LogFilter::all().iter().enumerate().map(|(index, filter)| {
            HitTarget::new(
                touch_target_vertical(log_filter_rect(context, index)),
                *filter,
            )
        }),
        vec2(x, y),
    )
}

pub fn log_timeline_row_rect(context: Rect, index: usize) -> Rect {
    let y = context.y + 160.0 + index as f32 * 36.0;
    Rect::new(context.x + 12.0, y, context.w - 24.0, 34.0)
}

pub fn log_event_row_rect(context: Rect, index: usize) -> Rect {
    let y = context.y + 102.0 + index as f32 * 42.0;
    Rect::new(context.x + 12.0, y, context.w - 24.0, 38.0)
}

pub fn log_event_row_at(context: Rect, row_count: usize, x: f32, y: f32) -> Option<usize> {
    hit_test(
        (0..row_count.min(4)).map(|index| {
            HitTarget::new(
                touch_target_vertical(log_event_row_rect(context, index)),
                index,
            )
        }),
        vec2(x, y),
    )
}

pub fn log_timeline_row_at(context: Rect, row_count: usize, x: f32, y: f32) -> Option<usize> {
    hit_test(
        (0..row_count.min(3)).map(|index| {
            HitTarget::new(
                touch_target_vertical(log_timeline_row_rect(context, index)),
                index,
            )
        }),
        vec2(x, y),
    )
}

pub fn log_report_close_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 112.0, context.y + 92.0, 96.0, 30.0)
}
