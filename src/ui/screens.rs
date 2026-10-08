//! Screens outside the colony view: title, new colony setup and results.

use super::actions::UiAction;
use super::art::Art;
use super::context::*;
use super::theme::*;
use crate::colony::campaign::Outcome;
use crate::data::{fill_template, game_data};
use crate::state::play::PlayState;
use crate::state::{SetupState, TitleState};
use macroquad::prelude::*;

fn backdrop(art: &Art, darkness: f32) {
    art.draw_backdrop(Rect::new(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT), WHITE);
    draw_rectangle(
        0.0,
        0.0,
        VIRTUAL_WIDTH,
        VIRTUAL_HEIGHT,
        Color::new(0.02, 0.03, 0.04, darkness),
    );
}

pub fn draw_title(ui: &mut Ui, art: &Art, title: &TitleState) {
    let data = game_data();
    backdrop(art, 0.35);
    draw_rectangle(
        0.0,
        0.0,
        860.0,
        VIRTUAL_HEIGHT,
        Color::new(0.03, 0.04, 0.05, 0.8),
    );
    label(data.label("game_title"), 110.0, 220.0, 84.0, TEXT);
    label(data.label("game_tagline"), 114.0, 320.0, TEXT_LARGE, ACCENT);
    paragraph(
        data.label("game_pitch"),
        114.0,
        370.0,
        560.0,
        TEXT_BODY,
        TEXT_DIM,
    );
    let mut y = 540.0;
    if title.has_save {
        ui.action_button(
            Rect::new(110.0, y, 420.0, 64.0),
            data.label("continue"),
            Tone::Primary,
            true,
            UiAction::ContinueSave,
        );
        y += 80.0;
    }
    let tone = if title.has_save {
        Tone::Normal
    } else {
        Tone::Primary
    };
    ui.action_button(
        Rect::new(110.0, y, 420.0, 64.0),
        data.label("new_colony"),
        tone,
        true,
        UiAction::OpenNewColony,
    );
    y += 80.0;
    if cfg!(not(target_arch = "wasm32")) {
        ui.action_button(
            Rect::new(110.0, y, 420.0, 56.0),
            data.label("quit"),
            Tone::Quiet,
            true,
            UiAction::Quit,
        );
    }
    if let Some(message) = &title.message {
        paragraph(message, 114.0, 920.0, 560.0, TEXT_SMALL, WARN);
    }
    label(
        data.label("version"),
        114.0,
        VIRTUAL_HEIGHT - 50.0,
        TEXT_SMALL,
        TEXT_FAINT,
    );
}

/// What a site has plenty of, what it lacks, and how wild its weather is.
fn site_traits(site: &crate::data::campaign::SiteDef, x: f32, y: f32, width: f32) {
    let data = game_data();
    let mut ranked: Vec<_> = site
        .node_density
        .iter()
        .filter(|(k, _)| **k != crate::data::NodeKind::Ruin)
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(a.1));
    let name =
        |kind: &crate::data::NodeKind| data.label(&format!("node_{}", kind.key())).to_owned();
    let plenty: Vec<String> = ranked.iter().take(2).map(|(k, _)| name(k)).collect();
    let scarce = ranked.last().map(|(k, _)| name(k)).unwrap_or_default();
    let storms = site.climate.storm_chance.iter().sum::<f32>() / 4.0;
    let ruins = site
        .node_density
        .get(&crate::data::NodeKind::Ruin)
        .copied()
        .unwrap_or(0.0);
    let lines = [
        (
            fill_template(data.label("site_plenty"), &[("list", &plenty.join(", "))]),
            GOOD,
        ),
        (
            fill_template(data.label("site_scarce"), &[("item", &scarce)]),
            WARN,
        ),
        (
            fill_template(
                data.label("site_storms"),
                &[("percent", &format!("{:.0}", storms * 100.0))],
            ),
            CYAN,
        ),
        (
            fill_template(
                data.label("site_ruins"),
                &[(
                    "amount",
                    data.label(if ruins >= 0.8 {
                        "many"
                    } else if ruins >= 0.55 {
                        "some"
                    } else {
                        "few"
                    }),
                )],
            ),
            Color::new(0.75, 0.6, 0.95, 1.0),
        ),
    ];
    let mut row = y;
    for (text, color) in lines {
        label_fit(&text, x, row, width, TEXT_SMALL + 1.0, color);
        row += 28.0;
    }
}

pub fn draw_setup(ui: &mut Ui, art: &Art, setup: &SetupState) {
    let data = game_data();
    backdrop(art, 0.72);
    label(data.label("new_colony"), 110.0, 70.0, TEXT_TITLE, TEXT);
    label(data.label("choose_site"), 114.0, 150.0, TEXT_LARGE, ACCENT);
    let width = 540.0;
    for (index, site) in data.campaign.sites.iter().enumerate() {
        let rect = Rect::new(110.0 + index as f32 * (width + 30.0), 200.0, width, 330.0);
        let selected = setup.site == site.id;
        let hovered = ui.hovered(rect);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected {
                Color::new(0.2, 0.16, 0.1, 0.95)
            } else if hovered {
                PANEL_HOVER
            } else {
                PANEL
            },
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected { 3.0 } else { 1.0 },
            if selected { ACCENT } else { LINE },
        );
        label(&site.name, rect.x + 24.0, rect.y + 22.0, TEXT_HEADING, TEXT);
        let temps = &site.climate.season_temperature;
        let climate = fill_template(
            data.label("climate_line"),
            &[
                ("summer", &format!("{:.0}", temps[1])),
                ("winter", &format!("{:.0}", temps[3])),
            ],
        );
        label(&climate, rect.x + 24.0, rect.y + 70.0, TEXT_BODY, CYAN);
        let height = paragraph(
            &site.description,
            rect.x + 24.0,
            rect.y + 110.0,
            rect.w - 48.0,
            TEXT_BODY,
            TEXT_DIM,
        );
        site_traits(site, rect.x + 24.0, rect.y + 130.0 + height, rect.w - 48.0);
        if ui.hit(rect) {
            ui.act(UiAction::SetupSite(site.id.clone()));
        }
    }
    label(
        data.label("choose_difficulty"),
        114.0,
        580.0,
        TEXT_LARGE,
        ACCENT,
    );
    let mut y = 630.0;
    for difficulty in &data.campaign.difficulties {
        let rect = Rect::new(110.0, y, 260.0, 56.0);
        let tone = if setup.difficulty == difficulty.id {
            Tone::Selected
        } else {
            Tone::Normal
        };
        ui.action_button(
            rect,
            &difficulty.name,
            tone,
            true,
            UiAction::SetupDifficulty(difficulty.id.clone()),
        );
        label_fit(
            &difficulty.description,
            400.0,
            y + 16.0,
            900.0,
            TEXT_BODY,
            TEXT_DIM,
        );
        y += 68.0;
    }
    label(
        &fill_template(
            data.label("seed_line"),
            &[("seed", &setup.seed.to_string())],
        ),
        114.0,
        860.0,
        TEXT_BODY,
        TEXT_DIM,
    );
    ui.action_button(
        Rect::new(400.0, 848.0, 200.0, 50.0),
        data.label("reroll"),
        Tone::Quiet,
        true,
        UiAction::RerollSeed,
    );
    paragraph(
        data.label("setup_note"),
        114.0,
        920.0,
        1100.0,
        TEXT_SMALL,
        TEXT_FAINT,
    );
    ui.action_button(
        Rect::new(VIRTUAL_WIDTH - 430.0, VIRTUAL_HEIGHT - 130.0, 320.0, 72.0),
        data.label("land"),
        Tone::Primary,
        true,
        UiAction::StartColony,
    );
    ui.action_button(
        Rect::new(VIRTUAL_WIDTH - 650.0, VIRTUAL_HEIGHT - 122.0, 200.0, 56.0),
        data.label("back"),
        Tone::Quiet,
        true,
        UiAction::BackToTitle,
    );
}

fn fates(play: &PlayState, x: f32, mut y: f32, width: f32, bottom: f32) {
    let data = game_data();
    heading(data.label("fates"), x, y);
    y += 30.0;
    for agent in &play.sim.agents {
        if y > bottom {
            break;
        }
        let fate = match &agent.life {
            crate::agents::LifeState::Dead { day, cause } => fill_template(
                data.label("fate_dead"),
                &[("day", &day.to_string()), ("cause", data.label(cause))],
            ),
            crate::agents::LifeState::Departed { day } => {
                fill_template(data.label("fate_left"), &[("day", &day.to_string())])
            }
            _ if agent.ambition.fulfilled => data.label("fate_fulfilled").to_owned(),
            _ => data.label("fate_stayed").to_owned(),
        };
        label_fit(&agent.name(), x, y, 220.0, TEXT_BODY, TEXT);
        label_fit(
            &fate,
            x + 230.0,
            y + 2.0,
            width - 230.0,
            TEXT_SMALL,
            TEXT_DIM,
        );
        y += 30.0;
    }
}

pub fn draw_results(ui: &mut Ui, art: &Art, play: &PlayState) {
    let data = game_data();
    let Some(outcome) = &play.sim.colony.campaign.outcome else {
        return;
    };
    ui.block(Rect::new(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    backdrop(art, 0.8);
    let (title, body, color) = match outcome {
        Outcome::Victory { ending, day } => {
            let def = data.ending(ending);
            let name = def.map(|e| e.name.clone()).unwrap_or_default();
            let epilogue = def.map(|e| e.epilogue.clone()).unwrap_or_default();
            (
                fill_template(
                    data.label("victory_title"),
                    &[("ending", &name), ("day", &day.to_string())],
                ),
                epilogue,
                ACCENT,
            )
        }
        Outcome::Failure { reason, day } => (
            fill_template(data.label("failure_title"), &[("day", &day.to_string())]),
            data.label(reason).to_owned(),
            BAD,
        ),
    };
    label(&title, 110.0, 90.0, TEXT_TITLE, color);
    paragraph(&body, 114.0, 190.0, 900.0, TEXT_LARGE, TEXT);
    let stats = &play.sim.colony.stats;
    let summary = fill_template(
        data.label("results_summary"),
        &[
            ("population", &play.sim.population().to_string()),
            ("births", &stats.births.to_string()),
            ("deaths", &stats.deaths.to_string()),
            (
                "techs",
                &play.sim.colony.tree.researched_count().to_string(),
            ),
            (
                "expeditions",
                &play.sim.colony.expeditions.completed.to_string(),
            ),
        ],
    );
    let mut y = 470.0 + paragraph(&summary, 114.0, 470.0, 900.0, TEXT_BODY, TEXT_DIM) + 24.0;
    heading(data.label("highlights"), 114.0, y);
    y += 30.0;
    let notable: Vec<_> = play
        .sim
        .colony
        .chronicle
        .entries
        .iter()
        .filter(|e| e.importance >= 2 && !e.agents.is_empty())
        .collect();
    let stride = (notable.len() / 9).max(1);
    for entry in notable.iter().step_by(stride).take(9) {
        let text = format!("{} {}: {}", data.label("day"), entry.day, entry.text);
        y += paragraph(&text, 114.0, y, 900.0, TEXT_SMALL + 1.0, TEXT_DIM) + 4.0;
        if y > VIRTUAL_HEIGHT - 200.0 {
            break;
        }
    }
    fates(play, 1100.0, 190.0, 700.0, VIRTUAL_HEIGHT - 160.0);
    ui.action_button(
        Rect::new(110.0, VIRTUAL_HEIGHT - 150.0, 320.0, 64.0),
        data.label("new_colony"),
        Tone::Primary,
        true,
        UiAction::OpenNewColony,
    );
    ui.action_button(
        Rect::new(450.0, VIRTUAL_HEIGHT - 150.0, 260.0, 64.0),
        data.label("title_screen"),
        Tone::Normal,
        true,
        UiAction::BackToTitle,
    );
}
