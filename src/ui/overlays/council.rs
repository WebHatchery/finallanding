//! Council overlay: proposing the colony's future in the Divergence vote.

use crate::data::{fill_template, game_data};
use crate::sim::commands::Command;
use crate::sim::vote::projected_support;
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use macroquad::prelude::*;

fn draw_result(play: &PlayState, content: Rect) {
    let data = game_data();
    let Some(vote) = &play.sim.colony.campaign.vote else {
        return;
    };
    let ending = data
        .ending(&vote.ending)
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let text = fill_template(
        data.label("vote_result"),
        &[
            ("ending", &ending),
            ("yes", &vote.supporters().to_string()),
            ("total", &vote.ballots.len().to_string()),
        ],
    );
    label(&text, content.x, content.y, TEXT_HEADING, ACCENT);
    let mut y = content.y + 60.0;
    let mut x = content.x;
    for ballot in &vote.ballots {
        let Some(agent) = play.sim.agent(ballot.agent) else {
            continue;
        };
        let color = if ballot.support > 0.0 { GOOD } else { BAD };
        label(&agent.given_name, x, y, TEXT_BODY, color);
        y += 30.0;
        if y > content.y + content.h - 40.0 {
            y = content.y + 60.0;
            x += 220.0;
        }
    }
}

pub fn draw(ui: &mut Ui, play: &PlayState, content: Rect) {
    let data = game_data();
    if play.sim.colony.campaign.vote.is_some() {
        draw_result(play, content);
        return;
    }
    let ready = play.sim.colony.campaign.act >= 4 && play.sim.world.count_built("council_hall") > 0;
    let intro = if ready {
        data.label("council_intro")
    } else {
        data.label("council_locked")
    };
    paragraph(
        intro,
        content.x,
        content.y,
        content.w * 0.7,
        TEXT_BODY,
        if ready { TEXT_DIM } else { WARN },
    );
    let width = (content.w - 40.0) / 3.0;
    for (index, ending) in data.campaign.endings.iter().enumerate() {
        let card = Rect::new(
            content.x + index as f32 * (width + 20.0),
            content.y + 80.0,
            width,
            content.h - 80.0,
        );
        panel_solid(card);
        let x = card.x + 20.0;
        let mut y = card.y + 18.0;
        label(&ending.name.to_uppercase(), x, y, TEXT_HEADING, ACCENT);
        y += 50.0;
        y += paragraph(&ending.pitch, x, y, card.w - 40.0, TEXT_BODY, TEXT) + 20.0;
        let tech = data
            .tech(&ending.capstone_tech)
            .map(|t| t.name.clone())
            .unwrap_or_default();
        let building = data
            .building(&ending.capstone_building)
            .map(|b| b.name.clone())
            .unwrap_or_default();
        heading(data.label("capstone"), x, y);
        y += 28.0;
        label(&tech, x, y, TEXT_BODY, CYAN);
        y += 28.0;
        label(&building, x, y, TEXT_BODY, CYAN);
        y += 44.0;
        let support = projected_support(&play.sim, &ending.id);
        let yes = support.iter().filter(|(_, s)| *s > 0.0).count();
        heading(data.label("projected_support"), x, y);
        y += 30.0;
        let fraction = yes as f32 / support.len().max(1) as f32;
        meter(
            Rect::new(x, y, card.w - 40.0, 12.0),
            fraction,
            if fraction > 0.5 { GOOD } else { WARN },
        );
        y += 22.0;
        label(
            &fill_template(
                data.label("would_vote"),
                &[
                    ("yes", &yes.to_string()),
                    ("total", &support.len().to_string()),
                ],
            ),
            x,
            y,
            TEXT_SMALL,
            TEXT_DIM,
        );
        y += 40.0;
        let mut ranked = support.clone();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        let names = |items: Vec<&(u32, f32)>| -> String {
            items
                .iter()
                .filter_map(|(id, _)| play.sim.agent(*id))
                .map(|a| a.given_name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let champions = names(ranked.iter().filter(|(_, s)| *s > 0.0).take(4).collect());
        let critics = names(
            ranked
                .iter()
                .rev()
                .filter(|(_, s)| *s < 0.0)
                .take(4)
                .collect(),
        );
        if !champions.is_empty() {
            heading(data.label("champions"), x, y);
            y += paragraph(&champions, x, y + 26.0, card.w - 40.0, TEXT_BODY, GOOD) + 34.0;
        }
        if !critics.is_empty() {
            heading(data.label("critics"), x, y);
            paragraph(&critics, x, y + 26.0, card.w - 40.0, TEXT_BODY, BAD);
        }
        let button = Rect::new(x, card.y + card.h - 70.0, card.w - 40.0, 52.0);
        ui.action_button(
            button,
            data.label("propose"),
            Tone::Primary,
            ready,
            UiAction::Command(Command::ProposeFuture(ending.id.clone())),
        );
    }
}
