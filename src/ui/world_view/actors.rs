//! Drawing survivors and fauna in world space.

use crate::agents::{Activity, Agent};
use crate::data::CreatureKind;
use crate::ui::camera::{point_center, tile_center, tile_size};
use crate::ui::theme::*;
use crate::world::{Creature, CreatureMood, World};
use macroquad::prelude::*;
use macroquad_toolkit::colors::{darken, shift_hue, with_alpha};

/// Base jacket colours for the six portrait archetypes.
const JACKETS: [Color; 6] = [
    Color::new(0.62, 0.38, 0.22, 1.0),
    Color::new(0.78, 0.74, 0.66, 1.0),
    Color::new(0.36, 0.48, 0.3, 1.0),
    Color::new(0.5, 0.36, 0.3, 1.0),
    Color::new(0.38, 0.45, 0.36, 1.0),
    Color::new(0.34, 0.36, 0.42, 1.0),
];

pub fn agent_color(agent: &Agent) -> Color {
    shift_hue(
        JACKETS[agent.portrait as usize % JACKETS.len()],
        agent.hue * 2.0,
    )
}

fn bob(agent: &Agent, time: f32) -> f32 {
    match agent.activity {
        Activity::Walking | Activity::Hauling | Activity::Fleeing => {
            (time * 9.0 + agent.id as f32).sin().abs() * 2.5
        }
        Activity::Working(_) => (time * 5.0 + agent.id as f32).sin() * 1.2,
        _ => 0.0,
    }
}

fn draw_glyph(activity: Activity, at: Vec2, time: f32) {
    let bubble = |color: Color| {
        draw_circle(at.x, at.y, 8.0, Color::new(0.05, 0.06, 0.07, 0.8));
        draw_circle_lines(at.x, at.y, 8.0, 1.5, color);
    };
    match activity {
        Activity::Sleeping => {
            bubble(CYAN);
            let drift = (time * 1.5).sin() * 1.5;
            draw_line(
                at.x - 3.0,
                at.y - 3.0 + drift,
                at.x + 3.0,
                at.y - 3.0 + drift,
                1.5,
                CYAN,
            );
            draw_line(
                at.x + 3.0,
                at.y - 3.0 + drift,
                at.x - 3.0,
                at.y + 3.0 + drift,
                1.5,
                CYAN,
            );
            draw_line(
                at.x - 3.0,
                at.y + 3.0 + drift,
                at.x + 3.0,
                at.y + 3.0 + drift,
                1.5,
                CYAN,
            );
        }
        Activity::Talking => {
            bubble(TEXT);
            for i in 0..3 {
                let alpha = 0.4 + 0.6 * ((time * 4.0 - i as f32).sin() * 0.5 + 0.5);
                draw_circle(
                    at.x - 3.5 + i as f32 * 3.5,
                    at.y,
                    1.4,
                    with_alpha(TEXT, alpha),
                );
            }
        }
        Activity::Fleeing => {
            bubble(BAD);
            draw_line(at.x, at.y - 4.5, at.x, at.y + 1.0, 2.0, BAD);
            draw_circle(at.x, at.y + 3.5, 1.2, BAD);
        }
        Activity::Fighting => {
            bubble(BAD);
            draw_line(at.x - 4.0, at.y - 4.0, at.x + 4.0, at.y + 4.0, 2.0, BAD);
            draw_line(at.x + 4.0, at.y - 4.0, at.x - 4.0, at.y + 4.0, 2.0, BAD);
        }
        Activity::Eating => {
            bubble(WARN);
            draw_circle(at.x, at.y, 3.5, WARN);
        }
        Activity::Breaking => {
            bubble(ROMANCE);
            draw_circle_lines(at.x, at.y, 4.0, 1.5, ROMANCE);
        }
        Activity::Recovering => {
            bubble(GOOD);
            draw_line(at.x - 4.0, at.y, at.x + 4.0, at.y, 2.0, GOOD);
            draw_line(at.x, at.y - 4.0, at.x, at.y + 4.0, 2.0, GOOD);
        }
        _ => {}
    }
}

pub fn draw_agent(agent: &Agent, world: &World, time: f32, selected: bool) {
    let mut center = point_center(agent.position);
    if let Some(structure) = agent.inside.and_then(|id| world.structure(id)) {
        let fp = structure.footprint.center();
        let offset = ((agent.id % 5) as f32 - 2.0) * 6.0;
        center = point_center(fp) + vec2(offset, ((agent.id / 5) % 3) as f32 * 5.0 - 5.0);
    }
    let scale = if agent.is_child { 0.7 } else { 1.0 };
    let radius = tile_size() * 0.32 * scale;
    let lift = bob(agent, time);
    let body = agent_color(agent);
    let alpha = if agent.inside.is_some() { 0.75 } else { 1.0 };
    draw_ellipse(
        center.x,
        center.y + radius * 0.9,
        radius * 1.1,
        radius * 0.45,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    draw_circle_lines(
        center.x,
        center.y + radius * 0.9,
        radius * 0.9,
        2.0,
        with_alpha(level_color(agent.mood), 0.8 * alpha),
    );
    let top = center.y - lift;
    draw_circle(center.x, top, radius, with_alpha(body, alpha));
    draw_circle(
        center.x - radius * 0.3,
        top - radius * 0.25,
        radius * 0.4,
        with_alpha(macroquad_toolkit::colors::lighten(body, 0.12), alpha),
    );
    let skin = Color::new(0.85, 0.66, 0.52, alpha);
    draw_circle(
        center.x,
        top - radius * 1.05,
        radius * 0.55,
        darken(skin, (agent.portrait % 3) as f32 * 0.12),
    );
    if let Some((resource, _)) = agent.carrying {
        draw_rectangle(
            center.x + radius * 0.5,
            top - radius * 0.2,
            7.0,
            7.0,
            resource_color(resource),
        );
        draw_rectangle_lines(
            center.x + radius * 0.5,
            top - radius * 0.2,
            7.0,
            7.0,
            1.0,
            BLACK,
        );
    }
    if selected {
        let pulse = 0.6 + 0.4 * (time * 4.0).sin();
        draw_circle_lines(
            center.x,
            center.y,
            radius * 2.0,
            2.5,
            with_alpha(SELECT, pulse),
        );
    }
    draw_glyph(
        agent.activity,
        vec2(center.x + radius, top - radius * 2.2),
        time,
    );
}

/// The selected survivor's route and their feelings toward people nearby.
pub fn draw_agent_overlay(agent: &Agent, agents: &[Agent]) {
    let from = point_center(agent.position);
    let mut previous = from;
    for tile in agent.path.iter().take(60) {
        let next = tile_center(*tile);
        draw_line(
            previous.x,
            previous.y,
            next.x,
            next.y,
            2.0,
            with_alpha(SELECT, 0.45),
        );
        previous = next;
    }
    for other in agents
        .iter()
        .filter(|o| o.is_present() && o.id != agent.id && o.inside.is_none())
    {
        let Some(relation) = agent.beliefs.relation(other.id) else {
            continue;
        };
        let strength = (relation.opinion.abs() / 100.0).clamp(0.0, 1.0);
        if strength < 0.25 {
            continue;
        }
        let color = if agent.partner == Some(other.id) {
            ROMANCE
        } else {
            opinion_color(relation.opinion)
        };
        let to = point_center(other.position);
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            1.0 + strength * 2.5,
            with_alpha(color, 0.25 + strength * 0.4),
        );
    }
}

pub fn draw_creature(creature: &Creature, max_health: f32, time: f32) {
    let c = point_center(creature.position);
    let s = tile_size();
    let (size, color) = match creature.kind {
        CreatureKind::Skitter => (s * 0.22, Color::new(0.45, 0.32, 0.22, 1.0)),
        CreatureKind::Ridgeback => (s * 0.36, Color::new(0.38, 0.2, 0.18, 1.0)),
        CreatureKind::Driftmaw => (s * 0.6, Color::new(0.25, 0.14, 0.22, 1.0)),
    };
    let wobble = (time * 10.0 + creature.id as f32).sin() * 1.5;
    draw_ellipse(
        c.x,
        c.y + size * 0.8,
        size * 1.2,
        size * 0.45,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    for i in 0..6 {
        let a = i as f32 / 6.0 * std::f32::consts::TAU + wobble * 0.05;
        let tip = c + vec2(a.cos(), a.sin()) * size * 1.35;
        draw_line(c.x, c.y, tip.x, tip.y, 2.0, darken(color, 0.3));
    }
    draw_circle(c.x, c.y, size, color);
    let eye = if creature.mood == CreatureMood::Fleeing {
        WARN
    } else {
        BAD
    };
    draw_circle(c.x - size * 0.35, c.y - size * 0.25, size * 0.14, eye);
    draw_circle(c.x + size * 0.35, c.y - size * 0.25, size * 0.14, eye);
    if creature.health < max_health {
        let w = size * 2.2;
        draw_rectangle(
            c.x - w * 0.5,
            c.y - size * 1.8,
            w,
            4.0,
            Color::new(0.1, 0.1, 0.1, 0.8),
        );
        draw_rectangle(
            c.x - w * 0.5,
            c.y - size * 1.8,
            w * (creature.health / max_health).max(0.0),
            4.0,
            BAD,
        );
    }
}
