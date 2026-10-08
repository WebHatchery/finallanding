//! The right-hand inspector for whatever is selected.

use super::actions::{InspectorTab, Selection, UiAction};
use super::art::Art;
use super::context::*;
use super::inspector_agent;
use super::theme::*;
use crate::data::{fill_template, game_data};
use crate::sim::commands::Command;
use crate::sim::Sim;
use crate::world::{CropStage, Structure};
use macroquad::prelude::*;

pub fn inspector_rect() -> Rect {
    let top = TOP_BAR_HEIGHT + 12.0;
    Rect::new(
        VIRTUAL_WIDTH - INSPECTOR_WIDTH - 12.0,
        top,
        INSPECTOR_WIDTH,
        VIRTUAL_HEIGHT - TOOLBAR_HEIGHT - top - 12.0,
    )
}

fn structure_status(sim: &Structure) -> String {
    let data = game_data();
    if !sim.is_built() {
        return fill_template(
            data.label("under_construction"),
            &[(
                "percent",
                &format!("{:.0}", sim.construction_fraction() * 100.0),
            )],
        );
    }
    if sim.def().needs_power() && !sim.powered {
        return data.label("unpowered").to_owned();
    }
    data.label("operational").to_owned()
}

fn draw_materials(structure: &Structure, x: f32, mut y: f32, width: f32) -> f32 {
    let data = game_data();
    heading(data.label("materials"), x, y);
    y += 26.0;
    for (resource, needed) in structure.def().cost_bag().nonzero() {
        let have = structure.delivered.get(resource);
        draw_rectangle(x, y + 6.0, 12.0, 12.0, resource_color(resource));
        label(data.label(resource.key()), x + 20.0, y, TEXT_BODY, TEXT);
        meter(
            Rect::new(x + 170.0, y + 9.0, width - 260.0, 8.0),
            have / needed,
            resource_color(resource),
        );
        label_right(
            &format!("{have:.0}/{needed:.0}"),
            x + width,
            y,
            TEXT_BODY,
            TEXT_DIM,
        );
        y += 30.0;
    }
    y + 6.0
}

fn draw_structure(ui: &mut Ui, sim: &Sim, structure: &Structure, rect: Rect) {
    let data = game_data();
    let def = structure.def();
    let x = rect.x + 14.0;
    let width = rect.w - 28.0;
    let mut y = rect.y + 14.0;
    label_fit(&def.name, x, y, width, TEXT_HEADING, TEXT);
    y += 40.0;
    label(
        &structure_status(structure),
        x,
        y,
        TEXT_BODY,
        if structure.is_operational() || !structure.is_built() {
            CYAN
        } else {
            WARN
        },
    );
    y += 32.0;
    y += paragraph(&def.description, x, y, width, TEXT_SMALL + 1.0, TEXT_DIM) + 12.0;
    if !structure.is_built() {
        y = draw_materials(structure, x, y, width);
        label(data.label("construction"), x, y, TEXT_BODY, TEXT);
        meter(
            Rect::new(x + 170.0, y + 9.0, width - 170.0, 8.0),
            structure.construction_fraction(),
            ACCENT,
        );
        let cancel = Rect::new(x, rect.y + rect.h - 60.0, 220.0, 46.0);
        ui.action_button(
            cancel,
            data.label("cancel_blueprint"),
            Tone::Danger,
            true,
            UiAction::Command(Command::CancelBlueprint(structure.id)),
        );
    } else {
        label(data.label("condition"), x, y, TEXT_BODY, TEXT);
        meter(
            Rect::new(x + 170.0, y + 9.0, width - 170.0, 8.0),
            structure.condition / 100.0,
            level_color(structure.condition),
        );
        y += 34.0;
        y = draw_function(sim, structure, x, y, width);
        draw_residents(ui, sim, structure, x, y, width);
    }
    let close = Rect::new(rect.x + rect.w - 154.0, rect.y + rect.h - 60.0, 140.0, 46.0);
    ui.action_button(
        close,
        data.label("close"),
        Tone::Quiet,
        true,
        UiAction::Deselect,
    );
}

fn draw_function(sim: &Sim, structure: &Structure, x: f32, mut y: f32, width: f32) -> f32 {
    let data = game_data();
    let def = structure.def();
    if let Some(crop) = &structure.crop {
        let key = match crop.stage {
            CropStage::Fallow => "crop_fallow",
            CropStage::Growing => "crop_growing",
            CropStage::Ripe => "crop_ripe",
        };
        label(data.label(key), x, y, TEXT_BODY, TEXT);
        meter(
            Rect::new(x + 170.0, y + 9.0, width - 170.0, 8.0),
            crop.growth,
            GOOD,
        );
        y += 34.0;
    }
    if let Some(recipe) = structure.craft_recipe.as_ref().and_then(|r| data.recipe(r)) {
        label_fit(&recipe.name, x, y, 160.0, TEXT_BODY, TEXT);
        meter(
            Rect::new(x + 170.0, y + 9.0, width - 170.0, 8.0),
            structure.craft_progress / recipe.work,
            CYAN,
        );
        y += 34.0;
    } else if def.station.is_some() {
        label(data.label("station_idle"), x, y, TEXT_SMALL, TEXT_DIM);
        y += 28.0;
    }
    if def.research > 0.0 {
        let project = sim
            .colony
            .research
            .active
            .as_ref()
            .and_then(|id| data.tech(id))
            .map(|t| t.name.clone())
            .unwrap_or_else(|| data.label("no_project").to_owned());
        label_fit(
            &format!("{} {}", data.label("researching"), project),
            x,
            y,
            width,
            TEXT_BODY,
            CYAN,
        );
        y += 30.0;
    }
    if def.power != 0.0 {
        label(
            &format!("{} {:+.0}", data.label("power"), def.power),
            x,
            y,
            TEXT_BODY,
            if def.power > 0.0 { CYAN } else { TEXT_DIM },
        );
        y += 30.0;
    }
    y
}

fn draw_residents(ui: &mut Ui, sim: &Sim, structure: &Structure, x: f32, mut y: f32, width: f32) {
    let data = game_data();
    if structure.def().beds == 0 {
        return;
    }
    heading(
        &format!(
            "{} {}/{}",
            data.label("residents"),
            structure.residents.len(),
            structure.def().beds
        ),
        x,
        y,
    );
    y += 26.0;
    for resident in structure.residents.iter().filter_map(|id| sim.agent(*id)) {
        let row = Rect::new(x, y, width, 30.0);
        label(&resident.name(), x, y + 4.0, TEXT_BODY, TEXT);
        if ui.hit(row) {
            ui.act(UiAction::Select(Selection::Agent(resident.id)));
        }
        y += 32.0;
    }
}

fn draw_simple(ui: &mut Ui, title: &str, lines: &[String], rect: Rect) {
    let data = game_data();
    let x = rect.x + 14.0;
    let mut y = rect.y + 14.0;
    label_fit(title, x, y, rect.w - 28.0, TEXT_HEADING, TEXT);
    y += 44.0;
    for line in lines {
        y += paragraph(line, x, y, rect.w - 28.0, TEXT_BODY, TEXT_DIM) + 8.0;
    }
    let close = Rect::new(rect.x + rect.w - 154.0, rect.y + rect.h - 60.0, 140.0, 46.0);
    ui.action_button(
        close,
        data.label("close"),
        Tone::Quiet,
        true,
        UiAction::Deselect,
    );
}

pub fn draw(
    ui: &mut Ui,
    art: &Art,
    sim: &Sim,
    selection: Selection,
    tab: InspectorTab,
    following: bool,
) {
    let data = game_data();
    let full = inspector_rect();
    let rect = match selection {
        Selection::Agent(_) => full,
        _ => Rect::new(full.x, full.y, full.w, 560.0),
    };
    ui.block(rect);
    panel_solid(rect);
    match selection {
        Selection::Agent(id) => {
            if let Some(agent) = sim.agent(id) {
                inspector_agent::draw(ui, art, sim, agent, tab, following, rect);
            }
        }
        Selection::Structure(id) => {
            if let Some(structure) = sim.world.structure(id) {
                draw_structure(ui, sim, structure, rect);
            }
        }
        Selection::Node(id) => {
            if let Some(node) = sim.world.node(id) {
                let title = data.label(&format!("node_{}", node.kind.key())).to_owned();
                let knowers = sim
                    .agents
                    .iter()
                    .filter(|a| a.is_alive() && a.beliefs.nodes.contains_key(&id))
                    .count();
                let lines = vec![
                    fill_template(
                        data.label("node_amount"),
                        &[
                            ("amount", &format!("{:.0}", node.amount)),
                            ("resource", data.label(node.resource().key())),
                        ],
                    ),
                    data.label(if node.regrows() {
                        "node_regrows"
                    } else {
                        "node_spent"
                    })
                    .to_owned(),
                    fill_template(data.label("node_known"), &[("count", &knowers.to_string())]),
                ];
                draw_simple(ui, &title, &lines, rect);
            }
        }
        Selection::Creature(id) => {
            if let Some(creature) = sim.world.creatures.iter().find(|c| c.id == id) {
                let title = data
                    .label(&format!("creature_{}", creature.kind.key()))
                    .to_owned();
                let lines = vec![
                    data.label(&format!("creature_{}_text", creature.kind.key()))
                        .to_owned(),
                    format!("{} {:.0}", data.label("health"), creature.health.max(0.0)),
                ];
                draw_simple(ui, &title, &lines, rect);
            }
        }
    }
}
