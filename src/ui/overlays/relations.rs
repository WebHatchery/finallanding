//! Relations overlay: the colony's web of friendships, rivalries and loves.

use crate::agents::Agent;
use crate::data::game_data;
use crate::state::play::PlayState;
use crate::ui::actions::{Selection, UiAction};
use crate::ui::art::Art;
use crate::ui::context::*;
use crate::ui::theme::*;
use crate::world::AgentId;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;

const SIDE: f32 = 520.0;

fn positions(agents: &[&Agent], center: Vec2, radius: f32) -> Vec<(AgentId, Vec2)> {
    let count = agents.len().max(1) as f32;
    agents
        .iter()
        .enumerate()
        .map(|(index, agent)| {
            let angle = index as f32 / count * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            (agent.id, center + vec2(angle.cos(), angle.sin()) * radius)
        })
        .collect()
}

fn edge_color(a: &Agent, b: &Agent) -> Option<(Color, f32)> {
    if a.partner == Some(b.id) {
        return Some((ROMANCE, 4.0));
    }
    let ab = a.beliefs.opinion_of(b.id);
    let ba = b.beliefs.opinion_of(a.id);
    let mean = (ab + ba) * 0.5;
    if mean.abs() < 25.0 {
        return None;
    }
    let strength = (mean.abs() / 100.0).clamp(0.0, 1.0);
    Some((opinion_color(mean), 1.0 + strength * 3.0))
}

fn draw_side(ui: &mut Ui, play: &PlayState, focus: Option<AgentId>, rect: Rect) {
    let data = game_data();
    panel_solid(rect);
    let x = rect.x + 16.0;
    let mut y = rect.y + 14.0;
    let Some(agent) = focus.and_then(|id| play.sim.agent(id)) else {
        paragraph(
            data.label("relations_help"),
            x,
            y,
            rect.w - 32.0,
            TEXT_BODY,
            TEXT_DIM,
        );
        return;
    };
    label_fit(&agent.name(), x, y, rect.w - 32.0, TEXT_HEADING, TEXT);
    y += 46.0;
    let mut relations: Vec<_> = agent
        .beliefs
        .relations
        .iter()
        .filter(|(_, r)| r.familiarity >= 5.0 || r.opinion.abs() >= 5.0)
        .filter_map(|(id, r)| play.sim.agent(*id).filter(|o| o.is_alive()).map(|o| (o, r)))
        .collect();
    relations.sort_by(|a, b| b.1.opinion.total_cmp(&a.1.opinion));
    if relations.is_empty() {
        label(data.label("no_bonds"), x, y, TEXT_BODY, TEXT_DIM);
    }
    for (other, relation) in relations.into_iter().take(22) {
        let back = other.beliefs.opinion_of(agent.id);
        label_fit(&other.given_name, x, y, 150.0, TEXT_BODY, TEXT);
        let bond = data.label(relation.bond().label_key());
        label_fit(
            bond,
            x + 160.0,
            y + 2.0,
            140.0,
            TEXT_SMALL,
            opinion_color(relation.opinion),
        );
        label_right(
            &format!("{:+.0} / {:+.0}", relation.opinion, back),
            rect.x + rect.w - 16.0,
            y + 2.0,
            TEXT_SMALL,
            TEXT_DIM,
        );
        y += 30.0;
    }
    let inspect = Rect::new(x, rect.y + rect.h - 62.0, rect.w - 32.0, 48.0);
    if ui.button(inspect, data.label("inspect"), Tone::Primary, true) {
        ui.act(UiAction::CloseOverlay);
        ui.act(UiAction::Select(Selection::Agent(agent.id)));
    }
}

pub fn draw(ui: &mut Ui, art: &Art, play: &PlayState, content: Rect) {
    let data = game_data();
    let agents: Vec<&Agent> = play.sim.agents.iter().filter(|a| a.is_alive()).collect();
    let graph = Rect::new(content.x, content.y, content.w - SIDE - 20.0, content.h);
    let center = graph.center();
    let radius = graph.h.min(graph.w) * 0.42;
    let spots = positions(&agents, center, radius);
    let focus = play.relation_focus;
    let spot = |id: AgentId| spots.iter().find(|(a, _)| *a == id).map(|(_, p)| *p);
    for (index, a) in agents.iter().enumerate() {
        for b in agents.iter().skip(index + 1) {
            let Some((color, width)) = edge_color(a, b) else {
                continue;
            };
            let involved = focus.is_none_or(|f| f == a.id || f == b.id);
            let alpha = if involved { 0.85 } else { 0.08 };
            if let (Some(pa), Some(pb)) = (spot(a.id), spot(b.id)) {
                draw_line(pa.x, pa.y, pb.x, pb.y, width, with_alpha(color, alpha));
            }
        }
    }
    for agent in &agents {
        let Some(p) = spot(agent.id) else {
            continue;
        };
        let size = if agent.is_child { 44.0 } else { 58.0 };
        let rect = Rect::new(p.x - size * 0.5, p.y - size * 0.5, size, size);
        art.draw_portrait(agent.portrait, agent.hue, rect);
        let selected = focus == Some(agent.id);
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected { 3.0 } else { 1.5 },
            if selected {
                SELECT
            } else {
                level_color(agent.mood)
            },
        );
        label_centered(
            &agent.given_name,
            p.x,
            p.y + size * 0.5 + 4.0,
            TEXT_SMALL,
            if selected { SELECT } else { TEXT },
        );
        let hit = Rect::new(rect.x - 8.0, rect.y - 8.0, rect.w + 16.0, rect.h + 30.0);
        if ui.hit(hit) {
            ui.act(UiAction::RelationFocus(if selected {
                None
            } else {
                Some(agent.id)
            }));
        }
    }
    let legend_y = content.y + content.h - 30.0;
    let mut x = content.x;
    for (key, color) in [
        ("legend_partner", ROMANCE),
        ("legend_friends", GOOD),
        ("legend_rivals", BAD),
    ] {
        draw_rectangle(x, legend_y + 9.0, 24.0, 4.0, color);
        label(data.label(key), x + 32.0, legend_y, TEXT_SMALL, TEXT_DIM);
        x += 180.0;
    }
    draw_side(
        ui,
        play,
        focus,
        Rect::new(content.x + content.w - SIDE, content.y, SIDE, content.h),
    );
}
