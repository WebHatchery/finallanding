//! The survivor inspector: mind, needs, bonds and life story.

use super::actions::{InspectorTab, Selection, UiAction};
use super::art::Art;
use super::context::*;
use super::describe::{clock, decision_text, goal_text, plan_text, step_text};
use super::theme::*;
use crate::agents::Agent;
use crate::data::{game_data, Need, NodeKind, Resource, Skill};
use crate::sim::Sim;
use macroquad::prelude::*;

fn header(art: &Art, sim: &Sim, agent: &Agent, rect: Rect) -> f32 {
    let data = game_data();
    art.draw_portrait(
        agent.portrait,
        agent.hue,
        Rect::new(rect.x + 14.0, rect.y + 14.0, 96.0, 96.0),
    );
    draw_rectangle_lines(
        rect.x + 14.0,
        rect.y + 14.0,
        96.0,
        96.0,
        1.5,
        level_color(agent.mood),
    );
    let x = rect.x + 124.0;
    label_fit(
        &agent.name(),
        x,
        rect.y + 12.0,
        rect.w - 140.0,
        TEXT_LARGE,
        TEXT,
    );
    let role = if agent.is_child {
        data.label("child").to_owned()
    } else {
        agent.backstory.former_role.clone()
    };
    label_fit(
        &format!("{:.0} · {}", agent.age_years, role),
        x,
        rect.y + 44.0,
        rect.w - 140.0,
        TEXT_SMALL,
        TEXT_DIM,
    );
    label(data.label("mood"), x, rect.y + 70.0, TEXT_SMALL, TEXT_DIM);
    meter(
        Rect::new(x + 60.0, rect.y + 76.0, rect.w - 210.0, 8.0),
        agent.mood / 100.0,
        level_color(agent.mood),
    );
    label_right(
        &format!("{:.0}", agent.mood),
        rect.x + rect.w - 14.0,
        rect.y + 68.0,
        TEXT_BODY,
        level_color(agent.mood),
    );
    let activity = data.label(agent.activity.label_key());
    let status = match &agent.life {
        crate::agents::LifeState::Away { .. } => data.label("away").to_owned(),
        _ => activity.to_owned(),
    };
    label(&status, x, rect.y + 90.0, TEXT_SMALL, CYAN);
    let mut chip_x = rect.x + 14.0;
    for trait_id in &agent.traits {
        if let Some(def) = data.trait_def(trait_id) {
            chip_x += chip(chip_x, rect.y + 120.0, &def.name, ACCENT) + 6.0;
        }
    }
    if let Some(partner) = agent.partner.and_then(|p| sim.agent(p)) {
        chip(
            chip_x,
            rect.y + 120.0,
            &format!("    {}", partner.given_name),
            ROMANCE,
        );
        icon_heart(chip_x + 7.0, rect.y + 126.0, 13.0, ROMANCE);
    }
    rect.y + 156.0
}

fn tabs(ui: &mut Ui, tab: InspectorTab, rect: Rect, y: f32) -> f32 {
    let data = game_data();
    let entries = [
        (InspectorTab::Mind, "tab_mind"),
        (InspectorTab::Needs, "tab_needs"),
        (InspectorTab::Bonds, "tab_bonds"),
        (InspectorTab::Life, "tab_life"),
    ];
    let width = (rect.w - 28.0 - 18.0) / 4.0;
    for (index, (value, key)) in entries.iter().enumerate() {
        let r = Rect::new(rect.x + 14.0 + index as f32 * (width + 6.0), y, width, 44.0);
        let tone = if tab == *value {
            Tone::Selected
        } else {
            Tone::Quiet
        };
        ui.action_button(
            r,
            data.label(key),
            tone,
            true,
            UiAction::InspectorTab(*value),
        );
    }
    y + 56.0
}

fn mind_tab(sim: &Sim, agent: &Agent, rect: Rect, mut y: f32) {
    let data = game_data();
    let x = rect.x + 14.0;
    let width = rect.w - 28.0;
    heading(data.label("now"), x, y);
    y += 24.0;
    match &agent.intention {
        Some(intention) => {
            label_fit(
                &goal_text(sim, &intention.goal),
                x,
                y,
                width,
                TEXT_LARGE,
                TEXT,
            );
            y += 32.0;
            label(data.label("drive"), x, y + 2.0, TEXT_SMALL, TEXT_DIM);
            meter(
                Rect::new(x + 70.0, y + 8.0, width - 70.0, 6.0),
                intention.utility / 1.5,
                ACCENT,
            );
            y += 24.0;
            if let Some(plan) = &intention.plan {
                label_fit(
                    &format!("{} {}", data.label("plan"), plan_text(&plan.id)),
                    x,
                    y,
                    width,
                    TEXT_BODY,
                    CYAN,
                );
                y += 28.0;
                for (index, step) in plan.steps.iter().enumerate().take(6) {
                    let color = if index < plan.cursor {
                        icon_check(x + 4.0, y + 4.0, 12.0, TEXT_FAINT);
                        TEXT_FAINT
                    } else if index == plan.cursor {
                        icon_arrow(x + 6.0, y + 4.0, 12.0, SELECT);
                        SELECT
                    } else {
                        draw_circle(x + 10.0, y + 10.0, 2.5, TEXT_DIM);
                        TEXT_DIM
                    };
                    label_fit(
                        &step_text(sim, step),
                        x + 24.0,
                        y,
                        width - 24.0,
                        TEXT_SMALL + 1.0,
                        color,
                    );
                    y += 24.0;
                }
            }
            if !intention.tried.is_empty() {
                let tried: Vec<String> = intention.tried.iter().map(|p| plan_text(p)).collect();
                y += paragraph(
                    &format!("{} {}", data.label("already_tried"), tried.join(", ")),
                    x,
                    y,
                    width,
                    TEXT_SMALL,
                    WARN,
                ) + 4.0;
            }
        }
        None => {
            label(data.label("deciding"), x, y, TEXT_BODY, TEXT_DIM);
            y += 30.0;
        }
    }
    if let Some((resource, amount)) = agent.carrying {
        label(
            &format!(
                "{} {amount:.1} {}",
                data.label("carrying"),
                data.label(resource.key())
            ),
            x,
            y,
            TEXT_SMALL,
            resource_color(resource),
        );
        y += 26.0;
    }
    y += 8.0;
    heading(data.label("reasoning"), x, y);
    y += 24.0;
    let bottom = rect.y + rect.h - 150.0;
    for decision in agent.decisions.entries.iter() {
        if y > bottom {
            break;
        }
        let (text, setback) = decision_text(sim, &decision.kind);
        label(&clock(decision.tick), x, y, TEXT_SMALL - 1.0, TEXT_FAINT);
        y += paragraph(
            &text,
            x + 52.0,
            y,
            width - 52.0,
            TEXT_SMALL,
            if setback { WARN } else { TEXT_DIM },
        ) + 2.0;
    }
    beliefs(sim, agent, rect, rect.y + rect.h - 140.0);
}

/// What the survivor believes, set against the truth where they differ.
fn beliefs(sim: &Sim, agent: &Agent, rect: Rect, mut y: f32) {
    let data = game_data();
    let x = rect.x + 14.0;
    divider(x, y - 6.0, rect.w - 28.0);
    heading(data.label("beliefs"), x, y);
    y += 24.0;
    let mut known = Vec::new();
    for kind in NodeKind::ALL {
        let count = agent
            .beliefs
            .nodes
            .values()
            .filter(|b| b.kind == kind && b.amount > 1.0)
            .count();
        if count > 0 {
            known.push(format!(
                "{} {}",
                count,
                data.label(&format!("node_{}", kind.key()))
            ));
        }
    }
    let places = if known.is_empty() {
        data.label("knows_nothing").to_owned()
    } else {
        known.join(", ")
    };
    y += paragraph(
        &format!("{} {}", data.label("knows_of"), places),
        x,
        y,
        rect.w - 28.0,
        TEXT_SMALL,
        TEXT_DIM,
    );
    let believed =
        agent.beliefs.stock.get(Resource::Food) + agent.beliefs.stock.get(Resource::Meals);
    let actual = sim.colony.stock.get(Resource::Food) + sim.colony.stock.get(Resource::Meals);
    let stores = crate::data::fill_template(
        data.label("believes_food"),
        &[
            ("believed", &format!("{believed:.0}")),
            ("actual", &format!("{actual:.0}")),
        ],
    );
    let color = if (believed - actual).abs() > actual * 0.25 + 5.0 {
        WARN
    } else {
        TEXT_DIM
    };
    y += paragraph(&stores, x, y, rect.w - 28.0, TEXT_SMALL, color);
    if !agent.beliefs.threats.is_empty() {
        let text = crate::data::fill_template(
            data.label("knows_threats"),
            &[("count", &agent.beliefs.threats.len().to_string())],
        );
        paragraph(&text, x, y, rect.w - 28.0, TEXT_SMALL, BAD);
    }
}

fn needs_tab(agent: &Agent, rect: Rect, mut y: f32) {
    let data = game_data();
    let x = rect.x + 14.0;
    let width = rect.w - 28.0;
    for need in Need::ALL {
        let value = agent.needs.get(need);
        label(
            data.label(&format!("need_{}", need.key())),
            x,
            y,
            TEXT_BODY,
            TEXT,
        );
        meter(
            Rect::new(x + 140.0, y + 9.0, width - 190.0, 10.0),
            value / 100.0,
            level_color(value),
        );
        label_right(&format!("{value:.0}"), x + width, y, TEXT_BODY, TEXT_DIM);
        y += 34.0;
    }
    let hp = agent.health.hp;
    label(data.label("health"), x, y, TEXT_BODY, TEXT);
    meter(
        Rect::new(x + 140.0, y + 9.0, width - 190.0, 10.0),
        hp / 100.0,
        level_color(hp),
    );
    label_right(&format!("{hp:.0}"), x + width, y, TEXT_BODY, TEXT_DIM);
    y += 34.0;
    if agent.health.ill {
        let key = if agent.health.treated {
            "ill_treated"
        } else {
            "ill"
        };
        label(data.label(key), x, y, TEXT_SMALL, BAD);
        y += 26.0;
    }
    y += 10.0;
    heading(data.label("thoughts"), x, y);
    y += 26.0;
    let mut thoughts: Vec<_> = agent.mind.thoughts.iter().collect();
    thoughts.sort_by(|a, b| b.mood.abs().total_cmp(&a.mood.abs()));
    for thought in thoughts.into_iter().take(14) {
        if y > rect.y + rect.h - 30.0 {
            break;
        }
        let text = data
            .thought(&thought.id)
            .map(|t| t.text.as_str())
            .unwrap_or(game_data().label(&thought.id));
        let color = if thought.mood >= 0.0 { GOOD } else { BAD };
        label_fit(text, x, y, width - 60.0, TEXT_SMALL + 1.0, TEXT_DIM);
        label_right(
            &format!("{:+.0}", thought.mood),
            x + width,
            y,
            TEXT_SMALL + 1.0,
            color,
        );
        y += 26.0;
    }
}

fn bonds_tab(ui: &mut Ui, sim: &Sim, agent: &Agent, rect: Rect, mut y: f32) {
    let data = game_data();
    let x = rect.x + 14.0;
    let width = rect.w - 28.0;
    let mut relations: Vec<_> = agent
        .beliefs
        .relations
        .iter()
        .filter_map(|(id, r)| {
            sim.agent(*id)
                .filter(|a| a.is_alive())
                .map(|other| (other, r))
        })
        .collect();
    relations.sort_by(|a, b| b.1.opinion.total_cmp(&a.1.opinion));
    if relations.is_empty() {
        label(data.label("no_bonds"), x, y, TEXT_BODY, TEXT_DIM);
    }
    for (other, relation) in relations {
        if y > rect.y + rect.h - 40.0 {
            break;
        }
        let row = Rect::new(x, y, width, 34.0);
        if ui.hovered(row) {
            draw_rectangle(row.x, row.y, row.w, row.h, PANEL_HOVER);
        }
        label_fit(&other.given_name, x + 6.0, y + 6.0, 130.0, TEXT_BODY, TEXT);
        let bond = data.label(relation.bond().label_key());
        let partner = agent.partner == Some(other.id);
        let bond_text = if partner { data.label("partner") } else { bond };
        label_fit(
            bond_text,
            x + 140.0,
            y + 8.0,
            120.0,
            TEXT_SMALL,
            if partner {
                ROMANCE
            } else {
                opinion_color(relation.opinion)
            },
        );
        let bar = Rect::new(x + 268.0, y + 14.0, width - 320.0, 6.0);
        draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::new(1.0, 1.0, 1.0, 0.07));
        let mid = bar.x + bar.w * 0.5;
        let extent = bar.w * 0.5 * (relation.opinion / 100.0);
        let (start, length) = if extent >= 0.0 {
            (mid, extent)
        } else {
            (mid + extent, -extent)
        };
        draw_rectangle(start, bar.y, length, bar.h, opinion_color(relation.opinion));
        if relation.romance > 20.0 {
            icon_heart(x + width - 44.0, y + 10.0, 13.0, ROMANCE);
        }
        label_right(
            &format!("{:+.0}", relation.opinion),
            x + width,
            y + 6.0,
            TEXT_SMALL,
            TEXT_DIM,
        );
        if ui.hit(row) {
            ui.act(UiAction::Select(Selection::Agent(other.id)));
        }
        y += 36.0;
    }
}

fn life_tab(agent: &Agent, rect: Rect, mut y: f32) {
    let data = game_data();
    let x = rect.x + 14.0;
    let width = rect.w - 28.0;
    for skill in Skill::ALL {
        let level = agent.skills.level(skill);
        let passion = agent.skills.has_passion(skill);
        label(
            data.label(skill.key()),
            x,
            y,
            TEXT_BODY,
            if passion { ACCENT } else { TEXT },
        );
        if passion {
            icon_star(x + 132.0, y + 5.0, 14.0, ACCENT);
        }
        meter(
            Rect::new(x + 160.0, y + 9.0, width - 210.0, 10.0),
            level as f32 / 20.0,
            CYAN,
        );
        label_right(&level.to_string(), x + width, y, TEXT_BODY, TEXT_DIM);
        y += 32.0;
    }
    y += 10.0;
    if let Some(ambition) = agent.ambition_def() {
        heading(data.label("ambition"), x, y);
        y += 24.0;
        let state = if agent.ambition.fulfilled {
            data.label("fulfilled")
        } else {
            ""
        };
        label_fit(
            &format!("{} {}", ambition.name, state),
            x,
            y,
            width,
            TEXT_BODY,
            if agent.ambition.fulfilled { GOOD } else { TEXT },
        );
        y += 28.0;
        y += paragraph(&ambition.description, x, y, width, TEXT_SMALL, TEXT_DIM);
        meter(
            Rect::new(x, y + 4.0, width, 6.0),
            agent.ambition.progress / ambition.target.max(0.01),
            ACCENT,
        );
        y += 22.0;
    }
    heading(data.label("story"), x, y);
    y += 24.0;
    let story = crate::data::fill_template(
        data.label("backstory"),
        &[
            ("origin", &agent.backstory.origin),
            ("memory", &agent.backstory.memory),
        ],
    );
    y += paragraph(&story, x, y, width, TEXT_SMALL, TEXT_DIM) + 8.0;
    let stats = &agent.stats;
    let summary = crate::data::fill_template(
        data.label("life_stats"),
        &[
            ("talks", &stats.conversations.to_string()),
            ("built", &stats.structures_built.to_string()),
            ("trips", &stats.expeditions.to_string()),
            ("recovered", &stats.plans_recovered.to_string()),
            ("helped", &stats.helped_others.to_string()),
        ],
    );
    paragraph(&summary, x, y, width, TEXT_SMALL, TEXT_FAINT);
}

pub fn draw(
    ui: &mut Ui,
    art: &Art,
    sim: &Sim,
    agent: &Agent,
    tab: InspectorTab,
    following: bool,
    rect: Rect,
) {
    let data = game_data();
    let y = header(art, sim, agent, rect);
    let y = tabs(ui, tab, rect, y);
    let content = Rect::new(rect.x, rect.y, rect.w, rect.h - 64.0);
    match tab {
        InspectorTab::Mind => mind_tab(sim, agent, content, y),
        InspectorTab::Needs => needs_tab(agent, content, y),
        InspectorTab::Bonds => bonds_tab(ui, sim, agent, content, y),
        InspectorTab::Life => life_tab(agent, content, y),
    }
    let footer_y = rect.y + rect.h - 58.0;
    let follow_tone = if following {
        Tone::Selected
    } else {
        Tone::Normal
    };
    ui.action_button(
        Rect::new(rect.x + 14.0, footer_y, 200.0, 46.0),
        data.label("follow"),
        follow_tone,
        true,
        UiAction::ToggleFollow,
    );
    ui.action_button(
        Rect::new(rect.x + rect.w - 154.0, footer_y, 140.0, 46.0),
        data.label("close"),
        Tone::Quiet,
        true,
        UiAction::Deselect,
    );
}
