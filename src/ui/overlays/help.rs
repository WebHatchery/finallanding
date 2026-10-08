//! Help pages and the first-run tutorial card.

use crate::data::game_data;
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use macroquad::prelude::*;

pub fn draw(ui: &mut Ui, play: &PlayState, content: Rect) {
    let data = game_data();
    let pages = &data.text.help;
    let mut y = content.y;
    for (index, page) in pages.iter().enumerate() {
        let rect = Rect::new(content.x, y, 320.0, 52.0);
        let tone = if play.help_page == index {
            Tone::Selected
        } else {
            Tone::Quiet
        };
        ui.action_button(rect, &page.title, tone, true, UiAction::HelpPage(index));
        y += 60.0;
    }
    let replay = Rect::new(content.x, y + 20.0, 320.0, 52.0);
    ui.action_button(
        replay,
        data.label("replay_tutorial"),
        Tone::Normal,
        true,
        UiAction::TutorialNext,
    );
    if let Some(page) = pages.get(play.help_page) {
        let x = content.x + 380.0;
        label(&page.title, x, content.y, TEXT_HEADING, ACCENT);
        paragraph(
            &page.body,
            x,
            content.y + 56.0,
            (content.w - 400.0).min(1000.0),
            TEXT_LARGE,
            TEXT,
        );
    }
}

/// The modal tutorial card shown over the paused colony.
pub fn draw_tutorial(ui: &mut Ui, step: usize) {
    let data = game_data();
    let steps = &data.text.tutorial;
    let Some(current) = steps.get(step) else {
        return;
    };
    let rect = Rect::new((VIRTUAL_WIDTH - 760.0) * 0.5, 300.0, 760.0, 330.0);
    ui.block(Rect::new(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    draw_rectangle(
        0.0,
        0.0,
        VIRTUAL_WIDTH,
        VIRTUAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    panel_solid(rect);
    draw_rectangle(rect.x, rect.y, rect.w, 4.0, ACCENT);
    label(
        &format!("{} / {}", step + 1, steps.len()),
        rect.x + 30.0,
        rect.y + 24.0,
        TEXT_SMALL,
        TEXT_FAINT,
    );
    label(
        &current.title,
        rect.x + 30.0,
        rect.y + 50.0,
        TEXT_HEADING,
        ACCENT,
    );
    paragraph(
        &current.body,
        rect.x + 30.0,
        rect.y + 104.0,
        rect.w - 60.0,
        TEXT_LARGE,
        TEXT,
    );
    let last = step + 1 >= steps.len();
    let next = Rect::new(rect.x + rect.w - 230.0, rect.y + rect.h - 76.0, 200.0, 54.0);
    let label_key = if last { "begin" } else { "next" };
    ui.action_button(
        next,
        data.label(label_key),
        Tone::Primary,
        true,
        UiAction::TutorialNext,
    );
    let skip = Rect::new(rect.x + 30.0, rect.y + rect.h - 76.0, 180.0, 54.0);
    ui.action_button(
        skip,
        data.label("skip"),
        Tone::Quiet,
        true,
        UiAction::TutorialClose,
    );
}
