//! Expeditions overlay: known sites, the volunteer call, parties in the
//! field and past reports.

use crate::data::{fill_template, game_data};
use crate::sim::commands::Command;
use crate::sim::expeditions::available_sites;
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use crate::world::Calendar;
use macroquad::prelude::*;

fn danger_color(danger: f32) -> Color {
    if danger < 0.3 {
        GOOD
    } else if danger < 0.55 {
        WARN
    } else {
        BAD
    }
}

fn sites(ui: &mut Ui, play: &PlayState, rect: Rect) {
    let data = game_data();
    let has_gate = play.sim.world.built().any(|s| s.def().expedition);
    let busy = play.sim.colony.expeditions.call.is_some();
    heading(data.label("sites"), rect.x, rect.y);
    if !has_gate {
        paragraph(
            data.label("needs_gate"),
            rect.x,
            rect.y + 30.0,
            rect.w,
            TEXT_BODY,
            WARN,
        );
        return;
    }
    let mut y = rect.y + 30.0;
    for site in available_sites(&play.sim) {
        let card = Rect::new(rect.x, y, rect.w, 118.0);
        panel_solid(card);
        label(&site.name, card.x + 16.0, card.y + 10.0, TEXT_LARGE, TEXT);
        let freshness = play.sim.colony.expeditions.reward_factor(&site.id);
        let danger = fill_template(
            data.label("danger"),
            &[("percent", &format!("{:.0}", site.danger * 100.0))],
        );
        label(
            &danger,
            card.x + 16.0,
            card.y + 44.0,
            TEXT_SMALL,
            danger_color(site.danger),
        );
        let days = fill_template(
            data.label("days"),
            &[("days", &format!("{:.1}", site.days))],
        );
        label(&days, card.x + 150.0, card.y + 44.0, TEXT_SMALL, TEXT_DIM);
        let yield_text = fill_template(
            data.label("yield"),
            &[("percent", &format!("{:.0}", freshness * 100.0))],
        );
        label(
            &yield_text,
            card.x + 260.0,
            card.y + 44.0,
            TEXT_SMALL,
            if freshness > 0.6 { TEXT_DIM } else { WARN },
        );
        let mut rewards: Vec<String> = site
            .rewards
            .iter()
            .map(|(r, _)| data.label(r.key()).to_owned())
            .collect();
        if site.relics > 0.0 {
            rewards.push(data.label("relics").to_owned());
        }
        if site.recruits {
            rewards.push(data.label("survivors").to_owned());
        }
        if site.reveals_tech {
            rewards.push(data.label("knowledge").to_owned());
        }
        label_fit(
            &rewards.join(" · "),
            card.x + 16.0,
            card.y + 70.0,
            card.w - 260.0,
            TEXT_SMALL,
            CYAN,
        );
        label_fit(
            &site.description,
            card.x + 16.0,
            card.y + 92.0,
            card.w - 260.0,
            TEXT_SMALL - 1.0,
            TEXT_FAINT,
        );
        let button = Rect::new(card.x + card.w - 230.0, card.y + 34.0, 214.0, 50.0);
        ui.action_button(
            button,
            data.label("call_volunteers"),
            Tone::Primary,
            !busy,
            UiAction::Command(Command::OpenExpedition(site.id.clone())),
        );
        y += 128.0;
        if y > rect.y + rect.h - 120.0 {
            break;
        }
    }
}

fn status(ui: &mut Ui, play: &PlayState, rect: Rect) {
    let data = game_data();
    let expeditions = &play.sim.colony.expeditions;
    let mut y = rect.y;
    heading(data.label("in_progress"), rect.x, y);
    y += 30.0;
    if let Some(call) = &expeditions.call {
        let site = data
            .expedition_site(&call.site)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        label(
            &fill_template(data.label("calling_for"), &[("site", &site)]),
            rect.x,
            y,
            TEXT_BODY,
            ACCENT,
        );
        y += 30.0;
        let names: Vec<String> = call
            .volunteers
            .iter()
            .filter_map(|id| play.sim.agent(*id))
            .map(|a| a.given_name.clone())
            .collect();
        let text = if names.is_empty() {
            data.label("no_volunteers_yet").to_owned()
        } else {
            names.join(", ")
        };
        y += paragraph(&text, rect.x, y, rect.w, TEXT_SMALL + 1.0, TEXT_DIM) + 6.0;
        paragraph(
            data.label("volunteer_help"),
            rect.x,
            y,
            rect.w,
            TEXT_SMALL,
            TEXT_FAINT,
        );
        y += 48.0;
        let cancel = Rect::new(rect.x, y, 200.0, 46.0);
        ui.action_button(
            cancel,
            data.label("cancel_call"),
            Tone::Danger,
            true,
            UiAction::Command(Command::CancelExpedition),
        );
        y += 60.0;
    }
    for active in &expeditions.active {
        let site = data
            .expedition_site(&active.site)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let back = (active.return_tick / Calendar::ticks_per_day()) + 1;
        let names: Vec<String> = active
            .members
            .iter()
            .filter_map(|id| play.sim.agent(*id))
            .map(|a| a.given_name.clone())
            .collect();
        let text = fill_template(
            data.label("party_away"),
            &[
                ("site", &site),
                ("names", &names.join(", ")),
                ("day", &back.to_string()),
            ],
        );
        y += paragraph(&text, rect.x, y, rect.w, TEXT_BODY, CYAN) + 10.0;
    }
    if expeditions.call.is_none() && expeditions.active.is_empty() {
        label(data.label("no_parties"), rect.x, y, TEXT_BODY, TEXT_DIM);
        y += 30.0;
    }
    y += 20.0;
    heading(data.label("reports"), rect.x, y);
    y += 30.0;
    for report in expeditions.reports.iter().rev() {
        if y > rect.y + rect.h - 40.0 {
            break;
        }
        label(
            &format!("{} {}", data.label("day"), report.day),
            rect.x,
            y,
            TEXT_SMALL,
            TEXT_FAINT,
        );
        y += paragraph(
            &report.summary,
            rect.x + 80.0,
            y,
            rect.w - 80.0,
            TEXT_SMALL,
            TEXT_DIM,
        ) + 8.0;
    }
}

pub fn draw(ui: &mut Ui, play: &PlayState, content: Rect) {
    let split = content.w * 0.56;
    sites(
        ui,
        play,
        Rect::new(content.x, content.y, split - 30.0, content.h),
    );
    status(
        ui,
        play,
        Rect::new(content.x + split, content.y, content.w - split, content.h),
    );
}
