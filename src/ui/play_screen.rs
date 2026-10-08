//! Composes the colony view: world, HUD, inspector, drawers and overlays.

use super::actions::Selection;
use super::art::Art;
use super::context::Ui;
use super::inspector::inspector_rect;
use super::theme::*;
use super::{build_panel, hud, inspector, overlays, screens, tracker, world_view};
use crate::state::play::PlayState;
use macroquad::prelude::*;
use macroquad_toolkit::ui::VirtualUi;

/// Regions that always belong to the UI in the current layout.
pub fn is_over_fixed_ui(play: &PlayState, point: Vec2) -> bool {
    if point.y < TOP_BAR_HEIGHT || point.y > VIRTUAL_HEIGHT - TOOLBAR_HEIGHT {
        return true;
    }
    if play.overlay.is_some() || play.tutorial_step.is_some() {
        return true;
    }
    if play.build_open && build_panel::drawer_rect().contains(point) {
        return true;
    }
    matches!(play.selection, Some(Selection::Agent(_))) && inspector_rect().contains(point)
}

pub fn draw(ui: &mut Ui, art: &Art, play: &mut PlayState, frame: &VirtualUi) {
    let pointer = ui.pointer.position;
    let world_pointer = (!is_over_fixed_ui(play, pointer)).then_some(pointer);
    world_view::draw(play, art, frame, world_pointer);
    let play: &PlayState = play;
    if let Some(overlay) = play.overlay {
        overlays::draw(ui, art, play, overlay);
    } else {
        tracker::draw_tracker(ui, play);
        tracker::draw_toasts(ui, play);
        if let Some(selection) = play.selection {
            inspector::draw(
                ui,
                art,
                &play.sim,
                selection,
                play.inspector_tab,
                play.follow,
            );
        }
        if play.build_open {
            build_panel::draw(ui, play);
        }
    }
    hud::draw_top_bar(ui, play);
    hud::draw_toolbar(ui, play);
    if let Some(step) = play.tutorial_step {
        overlays::help::draw_tutorial(ui, step);
    }
    if play.sim.colony.campaign.is_over() {
        screens::draw_results(ui, art, play);
    }
}
