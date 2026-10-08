//! Colonists overlay: a roster to compare survivors and jump to one.

use crate::agents::{Agent, LifeState};
use crate::data::game_data;
use crate::state::play::PlayState;
use crate::ui::actions::{Selection, UiAction};
use crate::ui::art::Art;
use crate::ui::context::*;
use crate::ui::describe::goal_text;
use crate::ui::theme::*;
use macroquad::prelude::*;

const ROW: f32 = 64.0;

fn status_text(agent: &Agent) -> (String, Color) {
    let data = game_data();
    match &agent.life {
        LifeState::Present => (data.label(agent.activity.label_key()).to_owned(), TEXT_DIM),
        LifeState::Away { .. } => (data.label("away").to_owned(), CYAN),
        LifeState::Dead { day, .. } => (
            crate::data::fill_template(data.label("died_on"), &[("day", &day.to_string())]),
            BAD,
        ),
        LifeState::Departed { day } => (
            crate::data::fill_template(data.label("left_on"), &[("day", &day.to_string())]),
            WARN,
        ),
    }
}

fn columns(content: Rect) -> [f32; 7] {
    let x = content.x;
    [
        x + 80.0,
        x + 330.0,
        x + 430.0,
        x + 560.0,
        x + 900.0,
        x + 1260.0,
        x + 1560.0,
    ]
}

pub fn draw(ui: &mut Ui, art: &Art, play: &PlayState, content: Rect) {
    let data = game_data();
    let cols = columns(content);
    for (index, key) in [
        "col_name",
        "col_age",
        "col_mood",
        "col_goal",
        "col_traits",
        "col_skills",
        "col_ambition",
    ]
    .iter()
    .enumerate()
    {
        label(
            &data.label(key).to_uppercase(),
            cols[index],
            content.y,
            TEXT_SMALL,
            ACCENT,
        );
    }
    let mut agents: Vec<&Agent> = play.sim.agents.iter().collect();
    agents.sort_by_key(|a| (!a.is_alive(), a.is_child, a.joined_day, a.id));
    let visible_rows = ((content.h - 40.0) / ROW) as usize;
    let max_scroll = agents.len().saturating_sub(visible_rows);
    let offset = (play.scroll.max(0.0) as usize).min(max_scroll);
    let mut y = content.y + 32.0;
    for agent in agents.iter().skip(offset).take(visible_rows) {
        let row = Rect::new(content.x, y, content.w, ROW - 6.0);
        let alive = agent.is_alive();
        if ui.hovered(row) && alive {
            draw_rectangle(row.x, row.y, row.w, row.h, PANEL_HOVER);
        }
        let tint = if alive { 0.0 } else { 120.0 };
        art.draw_portrait(
            agent.portrait,
            agent.hue + tint,
            Rect::new(content.x + 8.0, y + 3.0, 52.0, 52.0),
        );
        let name_color = if alive { TEXT } else { TEXT_FAINT };
        label_fit(
            &agent.name(),
            cols[0],
            y + 6.0,
            240.0,
            TEXT_BODY,
            name_color,
        );
        let (status, status_color) = status_text(agent);
        label_fit(&status, cols[0], y + 32.0, 240.0, TEXT_SMALL, status_color);
        label(
            &format!("{:.0}", agent.age_years),
            cols[1],
            y + 16.0,
            TEXT_BODY,
            TEXT_DIM,
        );
        if alive {
            meter(
                Rect::new(cols[2], y + 26.0, 100.0, 8.0),
                agent.mood / 100.0,
                level_color(agent.mood),
            );
            let goal = agent
                .current_goal()
                .map(|g| goal_text(&play.sim, &g))
                .unwrap_or_default();
            label_fit(&goal, cols[3], y + 16.0, 320.0, TEXT_BODY, TEXT);
        }
        let traits: Vec<&str> = agent
            .traits
            .iter()
            .filter_map(|t| data.trait_def(t))
            .map(|t| t.name.as_str())
            .collect();
        label_fit(
            &traits.join(", "),
            cols[4],
            y + 16.0,
            340.0,
            TEXT_SMALL + 1.0,
            TEXT_DIM,
        );
        let mut skills: Vec<_> = crate::data::Skill::ALL
            .iter()
            .map(|s| (*s, agent.skills.level(*s)))
            .collect();
        skills.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        let top: Vec<String> = skills
            .iter()
            .take(3)
            .map(|(s, l)| format!("{} {}", data.label(s.key()), l))
            .collect();
        label_fit(
            &top.join(" · "),
            cols[5],
            y + 16.0,
            290.0,
            TEXT_SMALL + 1.0,
            CYAN,
        );
        let ambition = agent
            .ambition_def()
            .map(|a| a.name.clone())
            .unwrap_or_default();
        let color = if agent.ambition.fulfilled {
            GOOD
        } else {
            TEXT_DIM
        };
        label_fit(
            &ambition,
            cols[6],
            y + 16.0,
            content.x + content.w - cols[6],
            TEXT_SMALL + 1.0,
            color,
        );
        if alive && ui.hit(row) {
            ui.act(UiAction::CloseOverlay);
            ui.act(UiAction::Select(Selection::Agent(agent.id)));
        }
        y += ROW;
    }
    if max_scroll > 0 {
        let up = Rect::new(
            content.x + content.w - 120.0,
            content.y + content.h - 50.0,
            56.0,
            46.0,
        );
        let down = Rect::new(
            content.x + content.w - 58.0,
            content.y + content.h - 50.0,
            56.0,
            46.0,
        );
        ui.action_button(up, "^", Tone::Quiet, offset > 0, UiAction::Scroll(-4.0));
        ui.action_button(
            down,
            "v",
            Tone::Quiet,
            offset < max_scroll,
            UiAction::Scroll(4.0),
        );
    }
}
