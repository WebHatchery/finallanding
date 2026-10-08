//! Full-screen panels that replace the map focus: colony, research, people,
//! relationships, expeditions, chronicle, council and help.

pub mod chronicle;
pub mod colony;
pub mod council;
pub mod expeditions;
pub mod help;
pub mod people;
pub mod relations;
pub mod research;

use super::actions::{Overlay, UiAction};
use super::art::Art;
use super::context::*;
use super::theme::*;
use crate::data::game_data;
use crate::state::play::PlayState;
use macroquad::prelude::*;

pub fn overlay_rect() -> Rect {
    let top = TOP_BAR_HEIGHT + 12.0;
    Rect::new(
        12.0,
        top,
        VIRTUAL_WIDTH - 24.0,
        VIRTUAL_HEIGHT - TOOLBAR_HEIGHT - top - 12.0,
    )
}

/// Draw the shared frame and return the content area.
fn frame(ui: &mut Ui, title: &str, subtitle: &str) -> Rect {
    let rect = overlay_rect();
    ui.block(rect);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.055, 0.065, 0.075, 0.98),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, LINE_STRONG);
    label(title, rect.x + 24.0, rect.y + 16.0, TEXT_HEADING, TEXT);
    label(
        subtitle,
        rect.x + 24.0 + text_width(title, TEXT_HEADING) + 20.0,
        rect.y + 24.0,
        TEXT_BODY,
        TEXT_DIM,
    );
    let close = Rect::new(rect.x + rect.w - 150.0, rect.y + 12.0, 136.0, 46.0);
    ui.action_button(
        close,
        game_data().label("close"),
        Tone::Quiet,
        true,
        UiAction::CloseOverlay,
    );
    divider(rect.x + 16.0, rect.y + 70.0, rect.w - 32.0);
    Rect::new(rect.x + 24.0, rect.y + 86.0, rect.w - 48.0, rect.h - 102.0)
}

pub fn draw(ui: &mut Ui, art: &Art, play: &PlayState, overlay: Overlay) {
    let data = game_data();
    match overlay {
        Overlay::Colony => {
            let content = frame(ui, data.label("tool_colony"), data.label("colony_subtitle"));
            colony::draw(ui, play, content);
        }
        Overlay::Research => {
            let content = frame(
                ui,
                data.label("tool_research"),
                data.label("research_subtitle"),
            );
            research::draw(ui, play, content);
        }
        Overlay::Colonists => {
            let content = frame(
                ui,
                data.label("tool_colonists"),
                data.label("colonists_subtitle"),
            );
            people::draw(ui, art, play, content);
        }
        Overlay::Relations => {
            let content = frame(
                ui,
                data.label("tool_relations"),
                data.label("relations_subtitle"),
            );
            relations::draw(ui, art, play, content);
        }
        Overlay::Expeditions => {
            let content = frame(
                ui,
                data.label("tool_expeditions"),
                data.label("expeditions_subtitle"),
            );
            expeditions::draw(ui, play, content);
        }
        Overlay::Chronicle => {
            let content = frame(
                ui,
                data.label("tool_chronicle"),
                data.label("chronicle_subtitle"),
            );
            chronicle::draw(ui, play, content);
        }
        Overlay::Council => {
            let content = frame(
                ui,
                data.label("tool_council"),
                data.label("council_subtitle"),
            );
            council::draw(ui, play, content);
        }
        Overlay::Help => {
            let content = frame(ui, data.label("help"), data.label("help_subtitle"));
            help::draw(ui, play, content);
        }
    }
}
