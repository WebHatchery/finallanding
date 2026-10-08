//! The selected technology: cost, progress, what inspired it, prerequisites,
//! unlocks, forks and the research-focus action.

use crate::colony::tech_tree::TechStatus;
use crate::data::techs::{TechDef, TechEffect};
use crate::data::{fill_template, game_data};
use crate::sim::commands::Command;
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use macroquad::prelude::*;

use super::{find_color, status_color};

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

/// The species behind an inspired technology that live here or were found,
/// with what the colony has gathered of each.
fn draw_inspiration(play: &PlayState, tech: &TechDef, x: f32, mut y: f32, width: f32) -> f32 {
    let data = game_data();
    let finds = &play.sim.colony.finds;
    heading(data.label("inspired_heading"), x, y);
    y += 26.0;
    let gathered = finds.gathered_any(&tech.inspired_by);
    let progress = fill_template(
        data.label("inspired_progress"),
        &[
            ("amount", &format!("{:.0}", gathered.min(tech.inspiration))),
            ("needed", &format!("{:.0}", tech.inspiration)),
        ],
    );
    label(&progress, x + 10.0, y, TEXT_SMALL + 1.0, TEXT_DIM);
    y += 26.0;
    for id in &tech.inspired_by {
        let Some(find) = finds.get(id).and_then(|f| data.find(&f.id)) else {
            continue;
        };
        draw_rectangle(x + 10.0, y + 4.0, 12.0, 12.0, find_color(find));
        label_fit(
            &format!("{} · {:.0}", find.name, finds.gathered(id)),
            x + 30.0,
            y,
            width - 30.0,
            TEXT_SMALL + 1.0,
            TEXT,
        );
        y += 26.0;
    }
    y + 6.0
}

pub fn draw_detail(ui: &mut Ui, play: &PlayState, id: &str, rect: Rect) {
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
    if tech.is_inspired() {
        y = draw_inspiration(play, tech, x, y, width);
    }
    if let Some(node) = tree.node(id) {
        let known: Vec<&String> = node
            .prereqs
            .iter()
            .filter(|p| !matches!(tree.status(p), TechStatus::Hidden | TechStatus::Absent))
            .collect();
        if !node.prereqs.is_empty() {
            heading(data.label("requires_heading"), x, y);
            y += 26.0;
            for prereq in &known {
                let name = data.tech(prereq).map_or(prereq.as_str(), |t| &t.name);
                label_fit(
                    name,
                    x + 10.0,
                    y,
                    width - 10.0,
                    TEXT_SMALL + 1.0,
                    status_color(tree.status(prereq)),
                );
                y += 26.0;
            }
            if known.len() < node.prereqs.len() {
                label(
                    data.label("requires_unknown"),
                    x + 10.0,
                    y,
                    TEXT_SMALL + 1.0,
                    TEXT_FAINT,
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
        ui.action_button(
            button,
            data.label("set_focus"),
            Tone::Primary,
            status == TechStatus::Available,
            UiAction::Command(Command::SetResearchFocus(Some(id.to_owned()))),
        );
    }
}
