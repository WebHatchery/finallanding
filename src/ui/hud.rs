//! The top bar (time, resources, speed, menu) and the bottom toolbar.

use super::actions::{Overlay, UiAction};
use super::context::*;
use super::theme::*;
use crate::data::{game_data, Resource};
use crate::state::play::PlayState;
use macroquad::prelude::*;

const SHOWN_RESOURCES: [Resource; 9] = [
    Resource::Food,
    Resource::Meals,
    Resource::Salvage,
    Resource::Fibre,
    Resource::Stone,
    Resource::Metal,
    Resource::Components,
    Resource::Medicine,
    Resource::Relics,
];

fn date_line(play: &PlayState) -> String {
    let calendar = play.sim.calendar;
    let data = game_data();
    let year = data
        .label("year_short")
        .replace("{year}", &calendar.year().to_string());
    format!(
        "{} {} · {} {} · {:02}:{:02}",
        data.label("day"),
        calendar.day(),
        data.label(calendar.season().key()),
        year,
        calendar.hour() as u32,
        calendar.minute_of_hour(),
    )
}

fn draw_resources(ui: &mut Ui, play: &PlayState, x: f32) -> f32 {
    let mut cursor = x;
    let stock = &play.sim.colony.stock;
    for resource in SHOWN_RESOURCES {
        let amount = stock.get(resource);
        if resource == Resource::Relics && amount < 0.5 && play.sim.colony.campaign.act < 2 {
            continue;
        }
        let value = format!("{amount:.0}");
        let width = 30.0 + text_width(&value, TEXT_BODY);
        let rect = Rect::new(cursor, 8.0, width, 40.0);
        draw_rectangle(cursor + 2.0, 21.0, 12.0, 12.0, resource_color(resource));
        label(&value, cursor + 20.0, 16.0, TEXT_BODY, TEXT);
        if ui.hovered(rect) {
            let name = game_data().label(resource.key());
            label(name, cursor, 60.0, TEXT_SMALL, TEXT_DIM);
        }
        cursor += width + 14.0;
    }
    let strip = Rect::new(x, 4.0, cursor - x, 48.0);
    if ui.hit(strip) {
        ui.act(UiAction::Open(Overlay::Colony));
    }
    let power = &play.sim.colony.power;
    let power_text = format!("{:.0}/{:.0}", power.generated, power.demand);
    let color = if power.generated + 0.1 >= power.demand {
        CYAN
    } else {
        BAD
    };
    draw_triangle(
        vec2(cursor + 7.0, 18.0),
        vec2(cursor + 1.0, 29.0),
        vec2(cursor + 8.0, 29.0),
        color,
    );
    draw_triangle(
        vec2(cursor + 5.0, 27.0),
        vec2(cursor + 12.0, 27.0),
        vec2(cursor + 4.0, 38.0),
        color,
    );
    label(&power_text, cursor + 18.0, 16.0, TEXT_BODY, color);
    cursor + 30.0 + text_width(&power_text, TEXT_BODY)
}

pub fn draw_top_bar(ui: &mut Ui, play: &PlayState) {
    let bar = Rect::new(0.0, 0.0, VIRTUAL_WIDTH, TOP_BAR_HEIGHT);
    ui.block(bar);
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w,
        bar.h,
        Color::new(0.06, 0.07, 0.08, 0.95),
    );
    draw_line(0.0, bar.h, bar.w, bar.h, 1.0, LINE);
    let data = game_data();
    label(
        &play.sim.colony.setup.colony_name.to_uppercase(),
        PAD,
        6.0,
        TEXT_SMALL,
        ACCENT,
    );
    label(&date_line(play), PAD, 26.0, TEXT_BODY, TEXT);
    let weather = &play.sim.world.weather;
    let weather_text = format!(
        "{} {:.0}°",
        data.label(weather.current.key()),
        weather.temperature
    );
    label(
        &weather_text,
        380.0,
        16.0,
        TEXT_BODY,
        if weather.current.is_severe() {
            WARN
        } else {
            TEXT_DIM
        },
    );
    let after = draw_resources(ui, play, 560.0);
    let people = format!(
        "{} {}",
        play.sim.present().count(),
        data.label("people_short")
    );
    let mood = play.sim.average_mood();
    label(&people, after + 10.0, 16.0, TEXT_BODY, TEXT);
    let mood_text = format!("{} {:.0}", data.label("mood"), mood);
    label(
        &mood_text,
        after + 30.0 + text_width(&people, TEXT_BODY),
        16.0,
        TEXT_BODY,
        level_color(mood),
    );
    draw_speed_controls(ui, play);
}

fn draw_speed_controls(ui: &mut Ui, play: &PlayState) {
    let labels = ["II", ">", ">>", ">>>"];
    let mut x = VIRTUAL_WIDTH - 470.0;
    for (index, text) in labels.iter().enumerate() {
        let rect = Rect::new(x, 6.0, 64.0, 44.0);
        let tone = if play.speed == index {
            Tone::Selected
        } else {
            Tone::Normal
        };
        ui.action_button(rect, text, tone, true, UiAction::SetSpeed(index));
        x += 70.0;
    }
    let data = game_data();
    let help = Rect::new(VIRTUAL_WIDTH - 180.0, 6.0, 74.0, 44.0);
    ui.action_button(
        help,
        data.label("help"),
        Tone::Quiet,
        true,
        UiAction::Open(Overlay::Help),
    );
    let menu = Rect::new(VIRTUAL_WIDTH - 98.0, 6.0, 86.0, 44.0);
    ui.action_button(
        menu,
        data.label("menu"),
        Tone::Quiet,
        true,
        UiAction::SaveAndExit,
    );
}

pub fn draw_toolbar(ui: &mut Ui, play: &PlayState) {
    let data = game_data();
    let y = VIRTUAL_HEIGHT - TOOLBAR_HEIGHT;
    let bar = Rect::new(0.0, y, VIRTUAL_WIDTH, TOOLBAR_HEIGHT);
    ui.block(bar);
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w,
        bar.h,
        Color::new(0.06, 0.07, 0.08, 0.95),
    );
    draw_line(0.0, y, bar.w, y, 1.0, LINE);
    let council_ready = play.sim.colony.campaign.act >= 4;
    let buttons: [(&str, Option<Overlay>, bool); 8] = [
        ("tool_build", None, true),
        ("tool_colony", Some(Overlay::Colony), true),
        ("tool_research", Some(Overlay::Research), true),
        ("tool_colonists", Some(Overlay::Colonists), true),
        ("tool_relations", Some(Overlay::Relations), true),
        ("tool_expeditions", Some(Overlay::Expeditions), true),
        ("tool_chronicle", Some(Overlay::Chronicle), true),
        ("tool_council", Some(Overlay::Council), council_ready),
    ];
    let width = 168.0;
    let gap = 8.0;
    let total = buttons.len() as f32 * (width + gap) - gap;
    let mut x = (VIRTUAL_WIDTH - total) * 0.5;
    for (key, overlay, enabled) in buttons {
        let rect = Rect::new(x, y + 12.0, width, 52.0);
        let active = match overlay {
            Some(o) => play.overlay == Some(o),
            None => play.build_open,
        };
        let tone = if active { Tone::Selected } else { Tone::Normal };
        let action = match overlay {
            Some(_) if active => UiAction::CloseOverlay,
            Some(o) => UiAction::Open(o),
            None => UiAction::ToggleBuild,
        };
        ui.action_button(rect, data.label(key), tone, enabled, action);
        x += width + gap;
    }
    let zoom_y = y + 12.0;
    ui.action_button(
        Rect::new(PAD, zoom_y, 52.0, 52.0),
        "-",
        Tone::Quiet,
        true,
        UiAction::ZoomBy(0.8),
    );
    ui.action_button(
        Rect::new(PAD + 58.0, zoom_y, 52.0, 52.0),
        "+",
        Tone::Quiet,
        true,
        UiAction::ZoomBy(1.25),
    );
    ui.action_button(
        Rect::new(PAD + 116.0, zoom_y, 110.0, 52.0),
        data.label("recenter"),
        Tone::Quiet,
        true,
        UiAction::Recenter,
    );
}
