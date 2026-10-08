//! Chronicle overlay: the colony AI's daily reports and searchable history.

use crate::colony::chronicle::{Category, Entry};
use crate::data::game_data;
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use macroquad::prelude::*;

const REPORT_WIDTH: f32 = 560.0;

fn filters(ui: &mut Ui, play: &PlayState, x: f32, y: f32) {
    let data = game_data();
    let all = Rect::new(x, y, 110.0, 44.0);
    let tone = if play.chronicle_filter.is_none() {
        Tone::Selected
    } else {
        Tone::Quiet
    };
    ui.action_button(
        all,
        data.label("all"),
        tone,
        true,
        UiAction::ChronicleFilter(None),
    );
    let mut cursor = x + 118.0;
    for category in Category::ALL {
        let rect = Rect::new(cursor, y, 140.0, 44.0);
        let tone = if play.chronicle_filter == Some(category) {
            Tone::Selected
        } else {
            Tone::Quiet
        };
        ui.action_button(
            rect,
            data.label(category.label_key()),
            tone,
            true,
            UiAction::ChronicleFilter(Some(category)),
        );
        cursor += 148.0;
    }
}

fn reports(ui: &mut Ui, play: &PlayState, rect: Rect) {
    let data = game_data();
    panel_solid(rect);
    let reports = &play.sim.colony.chronicle.reports;
    let mut y = rect.y + 14.0;
    heading(data.label("daily_reports"), rect.x + 16.0, y);
    y += 30.0;
    for report in reports.iter().rev() {
        if y > rect.y + rect.h - 60.0 {
            break;
        }
        let selected = play.chronicle_day == Some(report.day);
        let title = format!("{} {}", data.label("day"), report.day);
        let header = Rect::new(rect.x + 8.0, y - 4.0, rect.w - 16.0, 30.0);
        if selected {
            draw_rectangle(
                header.x,
                header.y,
                header.w,
                header.h,
                Color::new(0.22, 0.18, 0.12, 1.0),
            );
        }
        label(&title, rect.x + 16.0, y, TEXT_BODY, ACCENT);
        label_right(
            &format!("{} {}", report.population, data.label("people_short")),
            rect.x + rect.w - 16.0,
            y,
            TEXT_SMALL,
            TEXT_DIM,
        );
        if ui.hit(header) {
            ui.act(UiAction::ChronicleDay(if selected {
                None
            } else {
                Some(report.day)
            }));
        }
        y += 30.0;
        for line in report.lines.iter().take(4) {
            y += paragraph(line, rect.x + 24.0, y, rect.w - 40.0, TEXT_SMALL, TEXT_DIM) + 2.0;
        }
        y += 10.0;
    }
}

fn entry_matches(play: &PlayState, entry: &Entry) -> bool {
    play.chronicle_filter.is_none_or(|c| c == entry.category)
        && play.chronicle_day.is_none_or(|d| d == entry.day)
}

fn entries(ui: &mut Ui, play: &PlayState, rect: Rect) {
    let data = game_data();
    let all: Vec<&Entry> = play
        .sim
        .colony
        .chronicle
        .entries
        .iter()
        .rev()
        .filter(|e| entry_matches(play, e))
        .collect();
    let offset = (play.scroll.max(0.0) as usize).min(all.len().saturating_sub(1));
    let mut y = rect.y;
    for entry in all.iter().skip(offset) {
        let height = paragraph_height(&entry.text, rect.w - 190.0, TEXT_BODY);
        if y + height > rect.y + rect.h - 60.0 {
            break;
        }
        let color = category_color(entry.category);
        draw_rectangle(rect.x, y + 4.0, 4.0, height - 6.0, color);
        label(
            &format!("{} {} · {:02}:00", data.label("day"), entry.day, entry.hour),
            rect.x + 14.0,
            y,
            TEXT_SMALL,
            TEXT_FAINT,
        );
        let text_color = if entry.importance >= 2 {
            TEXT
        } else {
            TEXT_DIM
        };
        paragraph(
            &entry.text,
            rect.x + 170.0,
            y,
            rect.w - 190.0,
            TEXT_BODY,
            text_color,
        );
        y += height + 10.0;
    }
    if all.is_empty() {
        label(
            data.label("nothing_recorded"),
            rect.x,
            y,
            TEXT_BODY,
            TEXT_DIM,
        );
    }
    let up = Rect::new(rect.x + rect.w - 120.0, rect.y + rect.h - 50.0, 56.0, 46.0);
    let down = Rect::new(rect.x + rect.w - 58.0, rect.y + rect.h - 50.0, 56.0, 46.0);
    ui.action_button(up, "^", Tone::Quiet, offset > 0, UiAction::Scroll(-6.0));
    ui.action_button(
        down,
        "v",
        Tone::Quiet,
        offset + 1 < all.len(),
        UiAction::Scroll(6.0),
    );
}

pub fn draw(ui: &mut Ui, play: &PlayState, content: Rect) {
    reports(
        ui,
        play,
        Rect::new(content.x, content.y, REPORT_WIDTH, content.h),
    );
    let right = Rect::new(
        content.x + REPORT_WIDTH + 24.0,
        content.y,
        content.w - REPORT_WIDTH - 24.0,
        content.h,
    );
    filters(ui, play, right.x, right.y);
    entries(
        ui,
        play,
        Rect::new(right.x, right.y + 60.0, right.w, right.h - 60.0),
    );
}
