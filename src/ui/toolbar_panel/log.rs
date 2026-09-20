//! log domain.

use super::*;
use macroquad_toolkit::ui::{draw_ui_text, format_clock, measure_ui_text};

pub struct LogContext<'a> {
    pub context: Rect,
    pub logs: &'a [ColonyLogEntry],
    pub social_history: &'a [SocialHistoryEntry],
    pub social_history_page: usize,
    pub social_history_filter: LogFilter,
    pub social_history_query: &'a str,
    pub social_history_search_active: bool,
    pub selected_social_history_day: Option<u32>,
    pub summary: &'a ColonyPressureSummary,
    pub timeline_rows: &'a [SocialTimelineRow],
    pub page_count: usize,
}

pub fn draw_log_context(view: LogContext<'_>) {
    let LogContext {
        context,
        logs,
        social_history,
        social_history_page,
        social_history_filter,
        social_history_query,
        social_history_search_active,
        selected_social_history_day,
        summary,
        timeline_rows,
        page_count,
    } = view;
    let text = &crate::data::config::game_config().text;
    let mut hovered_history = None;
    draw_log_search_control(context, social_history_query, social_history_search_active);
    if social_history_search_active {
        draw_touch_keyboard(context);
    }
    let social_brief = social_brief_lines(summary);
    draw_ui_text(
        &social_brief.header,
        context.x + 18.0,
        context.y + 51.0,
        style::TINY_SIZE,
        social_brief.color,
    );
    draw_ui_text(
        &style::fit_text(&social_brief.detail, context.w - 36.0, style::TINY_SIZE),
        context.x + 18.0,
        context.y + 68.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );

    let current_page = social_history_page.min(page_count.saturating_sub(1));
    let timeline = timeline_rows;
    if !social_history.is_empty() {
        draw_ui_text(
            "SOCIAL TIMELINE",
            context.x + 18.0,
            context.y + 78.0,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
        draw_log_filter_controls(context, social_history_filter);
        if page_count > 1 {
            draw_log_page_controls(context, current_page, page_count);
        }

        if timeline.is_empty() {
            draw_ui_text(
                text.label("log_no_matching"),
                context.x + 18.0,
                context.y + 112.0,
                style::TINY_SIZE,
                style::TEXT_MUTED,
            );
            return;
        }

        if let Some(entry) =
            selected_social_history_entry(social_history, selected_social_history_day)
        {
            draw_social_report_drilldown(context, entry);
            return;
        }

        for (index, row) in timeline.iter().enumerate() {
            let rect = log_timeline_row_rect(context, index);
            if style::button_hovered(rect) {
                hovered_history = Some(row);
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.1, 0.14, 0.15, 0.7),
                );
            }
            if selected_social_history_day == Some(row.day) {
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.18, 0.22, 0.2, 0.82),
                );
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, style::ACCENT_GOLD);
            }
            draw_rectangle(rect.x, rect.y, 3.0, rect.h, row.color);
            draw_ui_text(
                &format!("D{}", row.day),
                rect.x + 9.0,
                rect.y + 24.0,
                style::SMALL_SIZE,
                row.color,
            );
            draw_ui_text(
                &style::fit_text(&row.title, rect.w - 182.0, style::SMALL_SIZE),
                rect.x + 39.0,
                rect.y + 24.0,
                style::SMALL_SIZE,
                style::TEXT_BODY,
            );
            draw_ui_text(
                &row.metrics,
                rect.x + rect.w - 104.0,
                rect.y + 24.0,
                style::TINY_SIZE,
                style::TEXT_MUTED,
            );
        }
        let _ = hovered_history;
        return;
    }

    let mut hovered_log = None;
    for (index, log) in logs.iter().rev().take(2).enumerate() {
        let y = context.y + 91.0 + index as f32 * 20.0;
        let row = Rect::new(context.x + 12.0, y - 14.0, context.w - 24.0, 18.0);
        if style::button_hovered(row) {
            hovered_log = Some(log);
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                Color::new(0.1, 0.14, 0.15, 0.65),
            );
        }
        draw_ui_text(
            category_prefix(log.category),
            context.x + 18.0,
            y,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
        draw_ui_text(
            &style::fit_text(
                &format!("{} {}", format_clock(log.hour, log.minute), log.title),
                context.w - 64.0,
                style::TINY_SIZE,
            ),
            context.x + 52.0,
            y,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    if let Some(log) = hovered_log {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &log.title, &log.detail);
    }
}

fn draw_touch_keyboard(context: Rect) {
    let text = &crate::data::config::game_config().text;
    style::draw_deep_panel(log_keyboard_bounds(context));
    let keys: &[char] = &[
        'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
        'L', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', ' ',
    ];
    for (index, key) in keys.iter().enumerate() {
        let rect = log_keyboard_key_rect(context, index);
        style::draw_button(rect, false, style::button_hovered(rect));
        let label = if *key == ' ' {
            "_".to_string()
        } else {
            key.to_string()
        };
        draw_ui_text(
            &label,
            rect.x + rect.w * 0.5 - 4.0,
            rect.y + 27.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
    }
    let backspace = Rect::new(context.x + 12.0, context.y - 3.0, context.w * 0.48, 41.0);
    let done = Rect::new(
        context.x + context.w * 0.52,
        context.y - 3.0,
        context.w * 0.48 - 12.0,
        41.0,
    );
    style::draw_button(backspace, false, style::button_hovered(backspace));
    style::draw_button(done, false, style::button_hovered(done));
    draw_ui_text(
        text.label("log_backspace"),
        backspace.x + 10.0,
        backspace.y + 27.0,
        style::TINY_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        text.label("log_done"),
        done.x + done.w * 0.5 - 14.0,
        done.y + 27.0,
        style::TINY_SIZE,
        style::TEXT_PRIMARY,
    );
}

pub fn draw_log_search_control(context: Rect, query: &str, active: bool) {
    let text = &crate::data::config::game_config().text;
    let search = log_search_rect(context);
    let clear = log_search_clear_rect(context);
    let export = log_search_export_rect(context);
    style::draw_button(search, active, style::button_hovered(search));
    style::draw_button(
        clear,
        false,
        !query.is_empty() && style::button_hovered(clear),
    );
    style::draw_button(export, false, style::button_hovered(export));

    let mut label = if query.is_empty() {
        text.label("log_search_placeholder").to_string()
    } else {
        style::fit_text(query, search.w - 22.0, style::TINY_SIZE)
    };
    if active {
        label.push('|');
    }

    draw_ui_text(
        &label,
        search.x + 10.0,
        search.y + 21.0,
        style::SMALL_SIZE,
        if query.is_empty() {
            style::TEXT_MUTED
        } else {
            style::TEXT_PRIMARY
        },
    );
    draw_ui_text(
        text.label("log_clear"),
        clear.x + 13.0,
        clear.y + 21.0,
        style::SMALL_SIZE,
        if query.is_empty() {
            style::TEXT_MUTED
        } else {
            style::TEXT_PRIMARY
        },
    );
    draw_ui_text(
        text.label("log_export_button"),
        export.x + 13.0,
        export.y + 21.0,
        style::SMALL_SIZE,
        style::TEXT_PRIMARY,
    );
}

pub fn draw_social_report_drilldown(context: Rect, entry: &SocialHistoryEntry) {
    let rect = Rect::new(
        context.x + 12.0,
        context.y + 96.0,
        context.w - 24.0,
        context.h - 108.0,
    );
    style::draw_deep_panel(rect);
    draw_rectangle(rect.x, rect.y, 4.0, rect.h, social_history_color(entry));
    draw_ui_text(
        &crate::data::config::game_config().text.fill(
            "log_day",
            &[
                entry.day.to_string(),
                style::fit_text(&entry.title, rect.w - 85.0, style::TINY_SIZE),
            ],
        ),
        rect.x + 14.0,
        rect.y + 24.0,
        style::SMALL_SIZE,
        style::TEXT_PRIMARY,
    );
    let close = log_report_close_rect(context);
    style::draw_button(close, false, style::button_hovered(close));
    draw_ui_text(
        "CLOSE REPORT",
        close.x + 10.0,
        close.y + 20.0,
        style::TINY_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_wrapped_report_text(
        "STORY",
        &entry.detail,
        rect.x + 14.0,
        rect.y + 54.0,
        rect.w - 28.0,
        style::SMALL_SIZE,
        style::TEXT_BODY,
    );
    draw_wrapped_report_text(
        "RECOMMENDATION",
        &entry.recommendation,
        rect.x + 14.0,
        rect.y + 122.0,
        rect.w - 28.0,
        style::SMALL_SIZE,
        style::HEADING_BLUE,
    );
}

fn draw_wrapped_report_text(
    label: &str,
    text: &str,
    x: f32,
    start_y: f32,
    width: f32,
    size: f32,
    color: Color,
) {
    draw_ui_text(label, x, start_y, style::TINY_SIZE, style::TEXT_MUTED);
    let mut line = String::new();
    let mut y = start_y + 19.0;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && measure_ui_text(&candidate, None, size as u16, 1.0).width > width {
            draw_ui_text(&line, x, y, size, color);
            y += size + 5.0;
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_ui_text(&line, x, y, size, color);
    }
}

pub fn draw_log_filter_controls(context: Rect, active_filter: LogFilter) {
    for (index, filter) in LogFilter::all().iter().enumerate() {
        let rect = log_filter_rect(context, index);
        let active = *filter == active_filter;
        style::draw_button(rect, active, style::button_hovered(rect));
        draw_ui_text(
            filter.label(),
            rect.x + 6.0,
            rect.y + 12.0,
            style::TINY_SIZE,
            if active {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
    }
}

pub fn draw_log_page_controls(context: Rect, current_page: usize, page_count: usize) {
    let previous = log_page_previous_rect(context);
    let next = log_page_next_rect(context);
    let can_go_previous = current_page > 0;
    let can_go_next = current_page + 1 < page_count;

    style::draw_button(
        previous,
        false,
        can_go_previous && style::button_hovered(previous),
    );
    style::draw_button(next, false, can_go_next && style::button_hovered(next));
    draw_ui_text(
        "<",
        previous.x + 10.0,
        previous.y + 20.0,
        style::TINY_SIZE,
        if can_go_previous {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_ui_text(
        ">",
        next.x + 10.0,
        next.y + 20.0,
        style::TINY_SIZE,
        if can_go_next {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_ui_text(
        &format!("{}/{}", current_page + 1, page_count),
        context.x + context.w - 63.0,
        context.y + 78.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

pub fn category_prefix(category: LogCategory) -> &'static str {
    match category {
        LogCategory::Time => "TIME",
        LogCategory::Social => "SOC",
        LogCategory::Work => "WORK",
        LogCategory::Mood => "MOOD",
        LogCategory::Resource => "RES",
        LogCategory::Mission => "MIS",
        LogCategory::Technology => "TECH",
        LogCategory::Colony => "COL",
        LogCategory::System => "SYS",
    }
}
