//! Colony overlay: work priorities, policies, stores and power.

use crate::agents::goals::WorkType;
use crate::data::{game_data, Resource};
use crate::sim::commands::Command;
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use macroquad::prelude::*;

fn priorities(ui: &mut Ui, play: &PlayState, x: f32, mut y: f32) {
    let data = game_data();
    heading(data.label("work_priorities"), x, y);
    y += 26.0;
    paragraph(
        data.label("work_priorities_help"),
        x,
        y,
        560.0,
        TEXT_SMALL,
        TEXT_DIM,
    );
    y += 56.0;
    let levels = [
        "priority_off",
        "priority_low",
        "priority_normal",
        "priority_high",
    ];
    for work in WorkType::ALL {
        label(data.label(work.label_key()), x, y + 12.0, TEXT_BODY, TEXT);
        let current = play.sim.colony.priority(work);
        for (level, key) in levels.iter().enumerate() {
            let rect = Rect::new(x + 160.0 + level as f32 * 104.0, y, 98.0, 46.0);
            let tone = if current as usize == level {
                Tone::Selected
            } else {
                Tone::Quiet
            };
            let command = Command::SetPriority {
                work,
                level: level as u8,
            };
            ui.action_button(
                rect,
                data.label(key),
                tone,
                true,
                UiAction::Command(command),
            );
        }
        y += 54.0;
    }
}

fn policies(ui: &mut Ui, play: &PlayState, x: f32, mut y: f32, width: f32) {
    let data = game_data();
    heading(data.label("policies"), x, y);
    y += 30.0;
    for policy in &data.society.policies {
        let unlocked = play.sim.colony.is_tech_researched(&policy.tech);
        let current = play
            .sim
            .colony
            .policies
            .get(&policy.id)
            .cloned()
            .unwrap_or_default();
        label(
            &policy.name,
            x,
            y,
            TEXT_BODY,
            if unlocked { TEXT } else { TEXT_FAINT },
        );
        let option = policy.options.iter().find(|o| o.id == current);
        let note = if unlocked {
            option.map(|o| o.description.clone()).unwrap_or_default()
        } else {
            let tech = policy
                .tech
                .as_ref()
                .and_then(|t| data.tech(t))
                .map(|t| t.name.clone())
                .unwrap_or_default();
            crate::data::fill_template(data.label("requires"), &[("tech", &tech)])
        };
        label_fit(
            &note,
            x + 200.0,
            y + 2.0,
            width - 200.0,
            TEXT_SMALL,
            TEXT_DIM,
        );
        y += 28.0;
        for (index, option) in policy.options.iter().enumerate() {
            let rect = Rect::new(x + index as f32 * 170.0, y, 162.0, 46.0);
            let tone = if option.id == current {
                Tone::Selected
            } else {
                Tone::Quiet
            };
            let command = Command::SetPolicy {
                policy: policy.id.clone(),
                option: option.id.clone(),
            };
            ui.action_button(
                rect,
                &option.name,
                tone,
                unlocked,
                UiAction::Command(command),
            );
        }
        y += 62.0;
    }
}

fn stores(play: &PlayState, x: f32, mut y: f32, width: f32) {
    let data = game_data();
    let colony = &play.sim.colony;
    let capacity = play.sim.world.storage_capacity();
    heading(data.label("stores"), x, y);
    label_right(
        &format!("{:.0} / {:.0}", colony.stock.total(), capacity),
        x + width,
        y,
        TEXT_SMALL,
        TEXT_DIM,
    );
    y += 30.0;
    for resource in Resource::ALL {
        let amount = colony.stock.get(resource);
        draw_rectangle(x, y + 6.0, 12.0, 12.0, resource_color(resource));
        label(data.label(resource.key()), x + 22.0, y, TEXT_BODY, TEXT);
        label_right(&format!("{amount:.0}"), x + width, y, TEXT_BODY, TEXT);
        y += 32.0;
    }
    y += 16.0;
    heading(data.label("power"), x, y);
    y += 30.0;
    let power = &colony.power;
    for (key, value) in [
        ("power_now", power.generated),
        ("power_capacity", power.capacity),
        ("power_demand", power.demand),
    ] {
        label(data.label(key), x, y, TEXT_BODY, TEXT_DIM);
        label_right(&format!("{value:.0}"), x + width, y, TEXT_BODY, TEXT);
        y += 30.0;
    }
    y += 16.0;
    heading(data.label("colony_record"), x, y);
    y += 30.0;
    let stats = &colony.stats;
    for (key, value) in [
        ("stat_built", stats.structures_built as f32),
        ("stat_births", stats.births as f32),
        ("stat_arrivals", stats.arrivals as f32),
        ("stat_deaths", stats.deaths as f32),
        ("stat_departures", stats.departures as f32),
        ("stat_eurekas", stats.eurekas as f32),
        ("stat_relics", stats.relics_found),
    ] {
        label(data.label(key), x, y, TEXT_BODY, TEXT_DIM);
        label_right(&format!("{value:.0}"), x + width, y, TEXT_BODY, TEXT);
        y += 30.0;
    }
}

pub fn draw(ui: &mut Ui, play: &PlayState, content: Rect) {
    priorities(ui, play, content.x, content.y);
    policies(ui, play, content.x + 620.0, content.y, 700.0);
    stores(play, content.x + content.w - 420.0, content.y, 400.0);
}
