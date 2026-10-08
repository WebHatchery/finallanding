//! The build drawer: building categories, cards with costs and requirements,
//! and the armed placement tool.

use super::actions::UiAction;
use super::context::*;
use super::theme::*;
use crate::data::buildings::BuildingDef;
use crate::data::{game_data, BuildingCategory};
use crate::sim::commands::is_unlocked;
use crate::state::play::PlayState;
use macroquad::prelude::*;

const DRAWER_HEIGHT: f32 = 214.0;
const CARD_WIDTH: f32 = 232.0;
const CARD_HEIGHT: f32 = 128.0;

pub fn drawer_rect() -> Rect {
    let width = 1560.0;
    Rect::new(
        (VIRTUAL_WIDTH - width) * 0.5,
        VIRTUAL_HEIGHT - TOOLBAR_HEIGHT - DRAWER_HEIGHT - 8.0,
        width,
        DRAWER_HEIGHT,
    )
}

fn summary(def: &BuildingDef) -> String {
    let data = game_data();
    let mut parts = Vec::new();
    if def.beds > 0 {
        parts.push(format!("{} {}", def.beds, data.label("beds")));
    }
    if def.storage > 0.0 {
        parts.push(format!("{:.0} {}", def.storage, data.label("storage")));
    }
    if def.power > 0.0 {
        parts.push(format!("+{:.0} {}", def.power, data.label("power")));
    } else if def.power < 0.0 {
        parts.push(format!("{:.0} {}", def.power, data.label("power")));
    }
    if def.research > 0.0 {
        parts.push(format!("{} ×{:.1}", data.label("research"), def.research));
    }
    if def.care_beds > 0 {
        parts.push(format!("{} {}", def.care_beds, data.label("care_beds")));
    }
    if def.recreation > 0.0 {
        parts.push(data.label("recreation").to_owned());
    }
    if def.heat > 0.0 {
        parts.push(data.label("heat").to_owned());
    }
    if def.farm.is_some() {
        parts.push(data.label("grows_food").to_owned());
    }
    parts.join(" · ")
}

fn draw_card(ui: &mut Ui, play: &PlayState, def: &BuildingDef, rect: Rect) {
    let data = game_data();
    let unlocked = is_unlocked(&play.sim, &def.id);
    let armed = play.tool.as_deref() == Some(def.id.as_str());
    let hovered = ui.hovered(rect);
    let fill = if armed {
        Color::new(0.22, 0.17, 0.1, 1.0)
    } else if hovered && unlocked {
        PANEL_HOVER
    } else {
        PANEL_RAISED
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if armed { 2.5 } else { 1.0 },
        if armed { ACCENT } else { LINE },
    );
    let swatch = Color::from_rgba(def.color[0], def.color[1], def.color[2], 255);
    draw_rectangle(rect.x + 10.0, rect.y + 12.0, 14.0, 14.0, swatch);
    let name_color = if unlocked { TEXT } else { TEXT_FAINT };
    label_fit(
        &def.name,
        rect.x + 32.0,
        rect.y + 8.0,
        rect.w - 70.0,
        TEXT_BODY,
        name_color,
    );
    label_right(
        &format!("{}×{}", def.size[0], def.size[1]),
        rect.x + rect.w - 10.0,
        rect.y + 10.0,
        TEXT_SMALL,
        TEXT_FAINT,
    );
    if unlocked {
        let mut x = rect.x + 10.0;
        for (resource, amount) in def.cost_bag().nonzero() {
            let short = play.sim.colony.stock.get(resource) < amount;
            let text = format!("{amount:.0}");
            draw_rectangle(x, rect.y + 46.0, 10.0, 10.0, resource_color(resource));
            label(
                &text,
                x + 14.0,
                rect.y + 41.0,
                TEXT_SMALL,
                if short { BAD } else { TEXT_DIM },
            );
            x += 26.0 + text_width(&text, TEXT_SMALL);
        }
        label_fit(
            &summary(def),
            rect.x + 10.0,
            rect.y + 68.0,
            rect.w - 20.0,
            TEXT_SMALL,
            CYAN,
        );
        label_fit(
            &def.description,
            rect.x + 10.0,
            rect.y + 94.0,
            rect.w - 20.0,
            TEXT_SMALL - 1.0,
            TEXT_FAINT,
        );
    } else {
        let tech = def
            .tech
            .as_ref()
            .and_then(|t| data.tech(t))
            .map(|t| t.name.clone())
            .unwrap_or_default();
        let text = fill_requires(&tech);
        label_fit(
            &text,
            rect.x + 10.0,
            rect.y + 46.0,
            rect.w - 20.0,
            TEXT_SMALL,
            WARN,
        );
        label_fit(
            &def.description,
            rect.x + 10.0,
            rect.y + 74.0,
            rect.w - 20.0,
            TEXT_SMALL - 1.0,
            TEXT_FAINT,
        );
    }
    if unlocked && ui.hit(rect) {
        ui.act(if armed {
            UiAction::DisarmTool
        } else {
            UiAction::ArmTool(def.id.clone())
        });
    }
}

fn fill_requires(tech: &str) -> String {
    crate::data::fill_template(game_data().label("requires"), &[("tech", tech)])
}

pub fn draw(ui: &mut Ui, play: &PlayState) {
    let data = game_data();
    let rect = drawer_rect();
    ui.block(rect);
    panel_solid(rect);
    let mut x = rect.x + 12.0;
    for category in BuildingCategory::ALL {
        let has_any = data
            .buildings
            .iter()
            .any(|b| b.category == category && b.is_player_buildable());
        if !has_any {
            continue;
        }
        let tab = Rect::new(x, rect.y + 10.0, 132.0, 44.0);
        let tone = if play.build_category == category {
            Tone::Selected
        } else {
            Tone::Quiet
        };
        let key = format!("cat_{}", category.key());
        ui.action_button(
            tab,
            data.label(&key),
            tone,
            true,
            UiAction::BuildCategory(category),
        );
        x += 138.0;
    }
    let cancel = Rect::new(rect.x + rect.w - 130.0, rect.y + 10.0, 118.0, 44.0);
    ui.action_button(
        cancel,
        data.label("close"),
        Tone::Quiet,
        true,
        UiAction::ToggleBuild,
    );
    let mut card_x = rect.x + 12.0;
    let buildings = data
        .buildings
        .iter()
        .filter(|b| b.category == play.build_category && b.is_player_buildable());
    for def in buildings {
        if card_x + CARD_WIDTH > rect.x + rect.w {
            break;
        }
        draw_card(
            ui,
            play,
            def,
            Rect::new(card_x, rect.y + 70.0, CARD_WIDTH, CARD_HEIGHT),
        );
        card_x += CARD_WIDTH + 10.0;
    }
}
