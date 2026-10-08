//! The act tracker, contextual alerts, toasts and status flashes that sit
//! over the map's upper-left and lower-left corners.

use super::actions::{Overlay, Selection, UiAction};
use super::context::*;
use super::theme::*;
use crate::data::{fill_template, game_data, Need};
use crate::sim::campaign::{is_met, progress_text};
use crate::state::play::PlayState;
use macroquad::prelude::*;

const TRACKER_WIDTH: f32 = 430.0;
const TOAST_SECONDS: f32 = 9.0;

struct Alert {
    text: String,
    color: Color,
    action: UiAction,
}

fn count_where(
    play: &PlayState,
    test: impl Fn(&crate::agents::Agent) -> bool,
) -> (usize, Option<u32>) {
    let matching: Vec<u32> = play
        .sim
        .present()
        .filter(|a| test(a))
        .map(|a| a.id)
        .collect();
    (matching.len(), matching.first().copied())
}

fn alert_text(key: &str, count: usize) -> String {
    fill_template(game_data().label(key), &[("count", &count.to_string())])
}

fn collect_alerts(play: &PlayState) -> Vec<Alert> {
    let mut alerts = Vec::new();
    let sim = &play.sim;
    let mut person_alert =
        |key: &str, color: Color, test: &dyn Fn(&crate::agents::Agent) -> bool| {
            let (count, first) = count_where(play, test);
            if let (true, Some(id)) = (count > 0, first) {
                alerts.push(Alert {
                    text: alert_text(key, count),
                    color,
                    action: UiAction::Select(Selection::Agent(id)),
                });
            }
        };
    person_alert("alert_starving", BAD, &|a| a.needs.get(Need::Food) < 20.0);
    person_alert("alert_freezing", BAD, &|a| a.needs.get(Need::Warmth) < 20.0);
    person_alert("alert_breaking", ROMANCE, &|a| a.mental_break.is_some());
    person_alert("alert_injured", WARN, &|a| {
        a.health.hp < 50.0 || a.health.ill
    });
    let beds: u32 = sim.world.built().map(|s| s.def().beds).sum();
    let present = sim.present().count() as u32;
    if beds < present {
        alerts.push(Alert {
            text: alert_text("alert_beds", (present - beds) as usize),
            color: WARN,
            action: UiAction::ToggleBuild,
        });
    }
    if let Some(creature) = sim.world.creatures.first() {
        alerts.push(Alert {
            text: alert_text("alert_creatures", sim.world.creatures.len()),
            color: BAD,
            action: UiAction::JumpTo(creature.tile),
        });
    }
    if sim.colony.power.demand > sim.colony.power.generated + 0.1 && sim.calendar.is_daylight() {
        alerts.push(Alert {
            text: game_data().label("alert_power").to_owned(),
            color: WARN,
            action: UiAction::ToggleBuild,
        });
    }
    let research = &sim.colony.research;
    if research.focus.is_none()
        && research.active.is_none()
        && !sim.colony.tree.available().is_empty()
    {
        alerts.push(Alert {
            text: game_data().label("alert_research").to_owned(),
            color: CYAN,
            action: UiAction::Open(Overlay::Research),
        });
    }
    if let Some(call) = &sim.colony.expeditions.call {
        let max = game_data().balance.expeditions.max_party;
        let text = fill_template(
            game_data().label("alert_call"),
            &[
                ("count", &call.volunteers.len().to_string()),
                ("max", &max.to_string()),
            ],
        );
        alerts.push(Alert {
            text,
            color: CYAN,
            action: UiAction::Open(Overlay::Expeditions),
        });
    }
    let council = sim.world.count_built("council_hall") > 0;
    if sim.colony.campaign.act >= 4 && council && sim.colony.campaign.vote.is_none() {
        alerts.push(Alert {
            text: game_data().label("alert_vote").to_owned(),
            color: ACCENT,
            action: UiAction::Open(Overlay::Council),
        });
    }
    alerts
}

fn draw_objectives(ui: &mut Ui, play: &PlayState, y: f32) -> f32 {
    let data = game_data();
    let campaign = &play.sim.colony.campaign;
    let Some(act) = data.act(campaign.act) else {
        return y;
    };
    let header = format!(
        "{} {} · {}",
        data.label("act"),
        roman(act.number),
        act.name.to_uppercase()
    );
    let header_rect = Rect::new(PAD, y, TRACKER_WIDTH, 40.0);
    let mut height = 44.0;
    if play.tracker_open {
        height += act.objectives.len() as f32 * 30.0 + 8.0;
    }
    panel(Rect::new(PAD, y, TRACKER_WIDTH, height));
    ui.block(Rect::new(PAD, y, TRACKER_WIDTH, height));
    label(&header, PAD + 14.0, y + 10.0, TEXT_BODY, ACCENT);
    label_right(
        if play.tracker_open { "-" } else { "+" },
        PAD + TRACKER_WIDTH - 14.0,
        y + 8.0,
        TEXT_LARGE,
        TEXT_DIM,
    );
    if ui.hit(header_rect) {
        ui.act(UiAction::ToggleTracker);
    }
    if !play.tracker_open {
        return y + height;
    }
    let mut row = y + 46.0;
    for objective in &act.objectives {
        let done =
            campaign.is_complete(act.number, &objective.id) || is_met(&play.sim, &objective.kind);
        let mark_color = if done { GOOD } else { TEXT_FAINT };
        draw_rectangle_lines(PAD + 14.0, row + 4.0, 16.0, 16.0, 1.5, mark_color);
        if done {
            draw_rectangle(PAD + 18.0, row + 8.0, 8.0, 8.0, GOOD);
        }
        let color = if done { TEXT_DIM } else { TEXT };
        label_fit(
            &objective.text,
            PAD + 40.0,
            row + 1.0,
            TRACKER_WIDTH - 140.0,
            TEXT_SMALL + 1.0,
            color,
        );
        if let Some(progress) = progress_text(&play.sim, &objective.kind) {
            label_right(
                &progress,
                PAD + TRACKER_WIDTH - 14.0,
                row + 1.0,
                TEXT_SMALL,
                TEXT_DIM,
            );
        }
        row += 30.0;
    }
    y + height
}

pub fn roman(number: u8) -> &'static str {
    ["", "I", "II", "III", "IV", "V"]
        .get(number as usize)
        .copied()
        .unwrap_or("")
}

pub fn draw_tracker(ui: &mut Ui, play: &PlayState) {
    let mut y = draw_objectives(ui, play, TOP_BAR_HEIGHT + 12.0) + 10.0;
    for alert in collect_alerts(play).into_iter().take(6) {
        let rect = Rect::new(PAD, y, TRACKER_WIDTH, 38.0);
        let hovered = ui.hovered(rect);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if hovered { PANEL_HOVER } else { PANEL },
        );
        draw_rectangle(rect.x, rect.y, 4.0, rect.h, alert.color);
        label_fit(
            &alert.text,
            rect.x + 14.0,
            rect.y + 9.0,
            rect.w - 24.0,
            TEXT_SMALL + 1.0,
            TEXT,
        );
        if ui.hit(rect) {
            ui.act(alert.action);
        }
        y += 44.0;
    }
}

pub fn draw_toasts(ui: &mut Ui, play: &PlayState) {
    let mut y = VIRTUAL_HEIGHT - TOOLBAR_HEIGHT - 20.0;
    let width = 560.0;
    for toast in play.toasts.iter().rev().take(4) {
        let age = play.clock - toast.born;
        let alpha = (1.0 - (age - TOAST_SECONDS + 1.0).max(0.0)).clamp(0.0, 1.0);
        let height = paragraph_height(&toast.text, width - 32.0, TEXT_SMALL + 1.0) + 18.0;
        y -= height + 8.0;
        let rect = Rect::new(PAD, y, width, height);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.07, 0.08, 0.09, 0.9 * alpha),
        );
        let color = category_color(toast.category);
        draw_rectangle(
            rect.x,
            rect.y,
            4.0,
            rect.h,
            Color::new(color.r, color.g, color.b, alpha),
        );
        paragraph(
            &toast.text,
            rect.x + 16.0,
            rect.y + 8.0,
            width - 32.0,
            TEXT_SMALL + 1.0,
            Color::new(TEXT.r, TEXT.g, TEXT.b, alpha),
        );
        if ui.hit(rect) {
            ui.act(UiAction::Open(Overlay::Chronicle));
        }
    }
    if let Some((text, born)) = &play.status {
        if play.clock - born < 3.0 {
            let width = text_width(text, TEXT_BODY) + 40.0;
            let rect = Rect::new(
                (VIRTUAL_WIDTH - width) * 0.5,
                VIRTUAL_HEIGHT - TOOLBAR_HEIGHT - 70.0,
                width,
                44.0,
            );
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.08, 0.06, 0.05, 0.92),
            );
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, WARN);
            label_centered(text, rect.center().x, rect.y + 11.0, TEXT_BODY, TEXT);
        }
    }
}

pub fn toast_seconds() -> f32 {
    TOAST_SECONDS
}
