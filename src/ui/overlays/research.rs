//! Research overlay: this run's technology tree, insight and the selected
//! technology's details.

use crate::colony::tech_tree::TechStatus;
use crate::data::techs::{TechDef, TechEffect};
use crate::data::{fill_template, game_data, Branch};
use crate::sim::commands::Command;
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use macroquad::prelude::*;
use std::collections::HashMap;

const SIDEBAR: f32 = 210.0;
const DETAIL: f32 = 470.0;
const CARD_HEIGHT: f32 = 38.0;

fn status_color(status: TechStatus) -> Color {
    match status {
        TechStatus::Researched => GOOD,
        TechStatus::Available => TEXT,
        TechStatus::Waiting => TEXT_DIM,
        TechStatus::Hidden => TEXT_FAINT,
        TechStatus::Locked => Color::new(0.4, 0.3, 0.3, 1.0),
    }
}

/// Card rectangles for every technology, laid out by branch row and tier column.
fn layout(content: Rect) -> HashMap<String, Rect> {
    let data = game_data();
    let grid_x = content.x + SIDEBAR;
    let grid_w = content.w - SIDEBAR - DETAIL - 20.0;
    let column = grid_w / 5.0;
    let row_h = content.h / 6.0;
    let mut rects = HashMap::new();
    for (row, branch) in Branch::ALL.iter().enumerate() {
        for tier in 1..=5u8 {
            let techs: Vec<&TechDef> = data
                .techs
                .iter()
                .filter(|t| t.branch == *branch && t.tier == tier)
                .collect();
            let count = techs.len().max(1) as f32;
            let gap = ((row_h - 12.0) - count * CARD_HEIGHT) / count.max(1.0);
            for (index, tech) in techs.iter().enumerate() {
                let x = grid_x + (tier - 1) as f32 * column + 6.0;
                let y = content.y
                    + row as f32 * row_h
                    + 6.0
                    + index as f32 * (CARD_HEIGHT + gap.clamp(2.0, 10.0));
                rects.insert(tech.id.clone(), Rect::new(x, y, column - 18.0, CARD_HEIGHT));
            }
        }
    }
    rects
}

fn draw_sidebar(play: &PlayState, content: Rect) {
    let data = game_data();
    let row_h = content.h / 6.0;
    let threshold = data.balance.research.eureka_threshold;
    for (row, branch) in Branch::ALL.iter().enumerate() {
        let y = content.y + row as f32 * row_h;
        if row > 0 {
            divider(content.x, y, content.w - DETAIL - 20.0);
        }
        let color = branch_color(*branch);
        draw_rectangle(content.x, y + 10.0, 4.0, row_h - 20.0, color);
        label(
            data.label(branch.key()),
            content.x + 14.0,
            y + 12.0,
            TEXT_LARGE,
            color,
        );
        let researched = play.sim.colony.tree.researched_in(*branch);
        label(
            &fill_template(
                data.label("researched_count"),
                &[("count", &researched.to_string())],
            ),
            content.x + 14.0,
            y + 44.0,
            TEXT_SMALL,
            TEXT_DIM,
        );
        let insight = play.sim.colony.research.insight[branch.index()];
        label(
            data.label("insight"),
            content.x + 14.0,
            y + 70.0,
            TEXT_SMALL,
            TEXT_FAINT,
        );
        meter(
            Rect::new(content.x + 90.0, y + 77.0, SIDEBAR - 110.0, 6.0),
            insight / threshold,
            color,
        );
    }
}

fn draw_links(play: &PlayState, rects: &HashMap<String, Rect>, focus: &str) {
    let Some(node) = play.sim.colony.tree.node(focus) else {
        return;
    };
    let Some(target) = rects.get(focus) else {
        return;
    };
    for prereq in &node.prereqs {
        if let Some(from) = rects.get(prereq) {
            let color = if play.sim.colony.tree.is_researched(prereq) {
                GOOD
            } else {
                WARN
            };
            draw_line(
                from.x + from.w,
                from.y + from.h * 0.5,
                target.x,
                target.y + target.h * 0.5,
                2.0,
                color,
            );
        }
    }
    for other in play
        .sim
        .colony
        .tree
        .nodes
        .iter()
        .filter(|n| n.prereqs.iter().any(|p| p == focus))
    {
        if let Some(to) = rects.get(&other.id) {
            draw_line(
                target.x + target.w,
                target.y + target.h * 0.5,
                to.x,
                to.y + to.h * 0.5,
                1.5,
                CYAN,
            );
        }
    }
}

fn draw_card(ui: &mut Ui, play: &PlayState, tech: &TechDef, rect: Rect, selected: bool) {
    let data = game_data();
    let tree = &play.sim.colony.tree;
    let status = tree.status(&tech.id);
    let research = &play.sim.colony.research;
    let hovered = ui.hovered(rect);
    let fill = if selected {
        Color::new(0.22, 0.18, 0.12, 1.0)
    } else if hovered {
        PANEL_HOVER
    } else {
        PANEL_RAISED
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    let color = branch_color(tech.branch);
    draw_rectangle(
        rect.x,
        rect.y,
        4.0,
        rect.h,
        if status == TechStatus::Hidden {
            TEXT_FAINT
        } else {
            color
        },
    );
    let is_focus = research.focus.as_deref() == Some(tech.id.as_str());
    let border = if is_focus {
        ACCENT
    } else if status == TechStatus::Available {
        LINE_STRONG
    } else {
        LINE
    };
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if is_focus { 2.5 } else { 1.0 },
        border,
    );
    let name = if status == TechStatus::Hidden {
        data.label("unknown_tech")
    } else {
        tech.name.as_str()
    };
    label_fit(
        name,
        rect.x + 12.0,
        rect.y + 8.0,
        rect.w - 36.0,
        TEXT_SMALL + 1.0,
        status_color(status),
    );
    if status == TechStatus::Researched {
        icon_check(rect.x + rect.w - 22.0, rect.y + 12.0, 12.0, GOOD);
    }
    if status == TechStatus::Locked {
        draw_line(
            rect.x + 8.0,
            rect.y + rect.h * 0.5,
            rect.x + rect.w - 8.0,
            rect.y + rect.h * 0.5,
            1.5,
            BAD,
        );
    }
    let progress = research.progress_of(&tech.id);
    if progress > 0.0 && status != TechStatus::Researched {
        let scale = play.sim.colony.difficulty_value(|d| d.research_cost);
        let cost = tree.cost(&tech.id, scale);
        draw_rectangle(
            rect.x + 4.0,
            rect.y + rect.h - 4.0,
            (rect.w - 4.0) * (progress / cost).min(1.0),
            4.0,
            ACCENT,
        );
    }
    if ui.hit(rect) {
        ui.act(UiAction::FocusTech(tech.id.clone()));
    }
}

fn effect_text(effect: &TechEffect) -> String {
    let data = game_data();
    let percent = |amount: f32| format!("{:+.0}%", amount * 100.0);
    match effect {
        TechEffect::WorkSpeed { skill, amount } => {
            format!("{} {}", data.label(skill.key()), percent(*amount))
        }
        TechEffect::NeedDecay { need, amount } => format!(
            "{} {}",
            data.label(&format!("need_{}", need.key())),
            percent(*amount)
        ),
        TechEffect::FarmYield { amount } => {
            format!("{} {}", data.label("effect_farm"), percent(*amount))
        }
        TechEffect::ResearchSpeed { amount } => {
            format!("{} {}", data.label("effect_research"), percent(*amount))
        }
        TechEffect::Carry { amount } => format!("{} +{amount:.0}", data.label("effect_carry")),
        TechEffect::Mood { amount } => format!("{} {amount:+.1}", data.label("effect_mood")),
        TechEffect::HealRate { amount } => {
            format!("{} {}", data.label("effect_heal"), percent(*amount))
        }
        TechEffect::ExpeditionSafety { amount } => {
            format!("{} {}", data.label("effect_expedition"), percent(*amount))
        }
        TechEffect::Defence { amount } => {
            format!("{} {}", data.label("effect_defence"), percent(*amount))
        }
        TechEffect::Sight { amount } => format!("{} +{amount:.0}", data.label("effect_sight")),
    }
}

fn unlock_lines(tech: &TechDef) -> Vec<String> {
    let data = game_data();
    let mut lines: Vec<String> = data
        .buildings
        .iter()
        .filter(|b| b.tech.as_deref() == Some(tech.id.as_str()))
        .map(|b| b.name.clone())
        .collect();
    lines.extend(
        data.recipes
            .iter()
            .filter(|r| r.tech.as_deref() == Some(tech.id.as_str()))
            .map(|r| r.name.clone()),
    );
    lines.extend(
        data.society
            .policies
            .iter()
            .filter(|p| p.tech.as_deref() == Some(tech.id.as_str()))
            .map(|p| p.name.clone()),
    );
    lines.extend(tech.effects.iter().map(effect_text));
    lines
}

fn draw_detail(ui: &mut Ui, play: &PlayState, id: &str, rect: Rect) {
    let data = game_data();
    let Some(tech) = data.tech(id) else {
        return;
    };
    panel_solid(rect);
    let tree = &play.sim.colony.tree;
    let status = tree.status(id);
    let x = rect.x + 16.0;
    let width = rect.w - 32.0;
    let mut y = rect.y + 14.0;
    if status == TechStatus::Hidden {
        label(data.label("unknown_tech"), x, y, TEXT_HEADING, TEXT_DIM);
        paragraph(
            data.label("hidden_help"),
            x,
            y + 46.0,
            width,
            TEXT_BODY,
            TEXT_DIM,
        );
        return;
    }
    label_fit(&tech.name, x, y, width, TEXT_HEADING, TEXT);
    y += 42.0;
    let tier = fill_template(data.label("tier"), &[("tier", &tech.tier.to_string())]);
    label(
        &format!("{} · {}", data.label(tech.branch.key()), tier),
        x,
        y,
        TEXT_BODY,
        branch_color(tech.branch),
    );
    y += 32.0;
    let scale = play.sim.colony.difficulty_value(|d| d.research_cost);
    let cost = tree.cost(id, scale);
    let progress = play.sim.colony.research.progress_of(id);
    label(
        &format!(
            "{} {:.0}/{:.0}",
            data.label("progress"),
            progress.min(cost),
            cost
        ),
        x,
        y,
        TEXT_BODY,
        TEXT_DIM,
    );
    meter(Rect::new(x, y + 28.0, width, 6.0), progress / cost, ACCENT);
    y += 48.0;
    y += paragraph(&tech.description, x, y, width, TEXT_BODY, TEXT_DIM) + 12.0;
    if let Some(node) = tree.node(id) {
        if !node.prereqs.is_empty() {
            heading(data.label("requires_heading"), x, y);
            y += 26.0;
            for prereq in &node.prereqs {
                let name = data.tech(prereq).map(|t| t.name.as_str()).unwrap_or(prereq);
                let shown = if tree.status(prereq) == TechStatus::Hidden {
                    data.label("unknown_tech")
                } else {
                    name
                };
                label_fit(
                    shown,
                    x + 10.0,
                    y,
                    width - 10.0,
                    TEXT_SMALL + 1.0,
                    status_color(tree.status(prereq)),
                );
                y += 26.0;
            }
        }
    }
    heading(data.label("unlocks"), x, y);
    y += 26.0;
    for line in unlock_lines(tech) {
        label_fit(&line, x + 10.0, y, width - 10.0, TEXT_SMALL + 1.0, CYAN);
        y += 26.0;
    }
    if let Some(fork) = &tech.fork {
        let others: Vec<&str> = data
            .techs
            .iter()
            .filter(|t| t.fork.as_deref() == Some(fork) && t.id != tech.id)
            .map(|t| t.name.as_str())
            .collect();
        let text = fill_template(
            data.label("fork_warning"),
            &[("others", &others.join(", "))],
        );
        paragraph(&text, x, y + 6.0, width, TEXT_SMALL, WARN);
    }
    let button = Rect::new(x, rect.y + rect.h - 62.0, width, 48.0);
    let is_focus = play.sim.colony.research.focus.as_deref() == Some(id);
    if is_focus {
        ui.action_button(
            button,
            data.label("clear_focus"),
            Tone::Normal,
            true,
            UiAction::Command(Command::SetResearchFocus(None)),
        );
    } else {
        let enabled = status == TechStatus::Available;
        ui.action_button(
            button,
            data.label("set_focus"),
            Tone::Primary,
            enabled,
            UiAction::Command(Command::SetResearchFocus(Some(id.to_owned()))),
        );
    }
}

pub fn draw(ui: &mut Ui, play: &PlayState, content: Rect) {
    let data = game_data();
    let rects = layout(content);
    draw_sidebar(play, content);
    let column = (content.w - SIDEBAR - DETAIL - 20.0) / 5.0;
    for tier in 1..=5 {
        let label_text = fill_template(data.label("tier"), &[("tier", &tier.to_string())]);
        label(
            &label_text,
            content.x + SIDEBAR + (tier - 1) as f32 * column + 6.0,
            content.y - 30.0,
            TEXT_SMALL,
            TEXT_FAINT,
        );
    }
    let selected = play
        .tech_focus
        .clone()
        .or_else(|| play.sim.colony.research.focus.clone());
    if let Some(focus) = &selected {
        draw_links(play, &rects, focus);
    }
    for tech in &data.techs {
        if let Some(rect) = rects.get(&tech.id) {
            draw_card(
                ui,
                play,
                tech,
                *rect,
                selected.as_deref() == Some(tech.id.as_str()),
            );
        }
    }
    let detail = Rect::new(content.x + content.w - DETAIL, content.y, DETAIL, content.h);
    match &selected {
        Some(id) => draw_detail(ui, play, id, detail),
        None => {
            panel_solid(detail);
            paragraph(
                data.label("research_help"),
                detail.x + 16.0,
                detail.y + 16.0,
                DETAIL - 32.0,
                TEXT_BODY,
                TEXT_DIM,
            );
        }
    }
}
