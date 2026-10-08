//! Applying UI actions to the game: the single place state changes.

use super::Game;
use crate::colony::campaign::Outcome;
use crate::data::game_data;
use crate::sim::commands::{apply as apply_command, Command, CommandError};
use crate::sim::Sim;
use crate::state::play::{PlayState, SPEED_PAUSED};
use crate::state::save;
use crate::state::{Screen, SetupState, TitleState};
use crate::ui::actions::UiAction;
use crate::ui::camera::{tile_center, WorldCamera};
use crate::ui::theme::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use macroquad::prelude::vec2;

fn fresh_seed() -> u64 {
    (macroquad::miniquad::date::now() * 1000.0) as u64 % 1_000_000 + 1
}

fn to_title(game: &mut Game, message: Option<String>) {
    game.screen = Screen::Title(TitleState {
        has_save: save::has_save(),
        message,
    });
}

fn start_play(game: &mut Game, sim: Sim) {
    let tutorial = !game.preferences.tutorial_done;
    game.screen = Screen::Playing(Box::new(PlayState::new(sim, tutorial)));
}

fn menu_action(game: &mut Game, action: &UiAction) -> bool {
    match action {
        UiAction::OpenNewColony => {
            record_finished_run(game);
            game.screen = Screen::Setup(SetupState {
                site: game_data().campaign.sites[0].id.clone(),
                difficulty: "standard".into(),
                seed: fresh_seed(),
            });
        }
        UiAction::BackToTitle => {
            record_finished_run(game);
            to_title(game, None);
        }
        UiAction::ContinueSave => match save::load() {
            Ok(sim) => start_play(game, sim),
            Err(error) => to_title(
                game,
                Some(format!("{} {error}", game_data().label("load_failed"))),
            ),
        },
        UiAction::StartColony => {
            if let Screen::Setup(setup) = &game.screen {
                let sim = Sim::new(setup.to_run());
                if let Err(error) = save::save(&sim) {
                    eprintln!("initial save failed: {error}");
                }
                start_play(game, sim);
            }
        }
        UiAction::SetupSite(site) => {
            if let Screen::Setup(setup) = &mut game.screen {
                setup.site = site.clone();
            }
        }
        UiAction::SetupDifficulty(difficulty) => {
            if let Screen::Setup(setup) = &mut game.screen {
                setup.difficulty = difficulty.clone();
            }
        }
        UiAction::RerollSeed => {
            if let Screen::Setup(setup) = &mut game.screen {
                setup.seed = fresh_seed();
            }
        }
        UiAction::Quit => game.quit_requested = true,
        UiAction::SaveAndExit => {
            if let Screen::Playing(play) = &game.screen {
                if let Err(error) = save::save(&play.sim) {
                    eprintln!("save failed: {error}");
                }
            }
            to_title(game, None);
        }
        _ => return false,
    }
    true
}

/// Remember finished runs and clear their save so Continue starts fresh.
fn record_finished_run(game: &mut Game) {
    let Screen::Playing(play) = &game.screen else {
        return;
    };
    let Some(outcome) = &play.sim.colony.campaign.outcome else {
        return;
    };
    game.preferences.runs_completed += 1;
    if let Outcome::Victory { ending, .. } = outcome {
        if !game.preferences.endings_seen.contains(ending) {
            game.preferences.endings_seen.push(ending.clone());
        }
    }
    save::save_preferences(&game.preferences);
    save::delete();
}

fn command_feedback(play: &mut PlayState, error: CommandError) {
    let data = game_data();
    let text = match error {
        CommandError::Placement(issue) => data.label(issue.label_key()).to_owned(),
        CommandError::Locked => data.label("locked").to_owned(),
        CommandError::UnknownBuilding | CommandError::Rejected => {
            data.label("not_possible").to_owned()
        }
    };
    play.flash(text);
}

fn play_action(
    play: &mut PlayState,
    preferences: &mut crate::state::save::Preferences,
    action: UiAction,
) {
    match action {
        UiAction::SetSpeed(speed) => {
            play.speed = speed.min(3);
            if speed != SPEED_PAUSED {
                play.last_speed = speed;
            }
        }
        UiAction::TogglePause => {
            play.speed = if play.speed == SPEED_PAUSED {
                play.last_speed.max(1)
            } else {
                SPEED_PAUSED
            };
        }
        UiAction::Open(overlay) => {
            play.overlay = if play.overlay == Some(overlay) {
                None
            } else {
                Some(overlay)
            };
            play.scroll = 0.0;
            play.build_open = false;
            play.tool = None;
        }
        UiAction::CloseOverlay => play.overlay = None,
        UiAction::ToggleBuild => {
            play.build_open = !play.build_open;
            play.overlay = None;
            if !play.build_open {
                play.tool = None;
            }
        }
        UiAction::BuildCategory(category) => play.build_category = category,
        UiAction::ArmTool(building) => {
            play.tool = Some(building);
            play.selection = None;
        }
        UiAction::DisarmTool => play.tool = None,
        UiAction::PlaceAt(origin) => {
            if let Some(building) = play.tool.clone() {
                if let Err(error) =
                    apply_command(&mut play.sim, Command::PlaceBlueprint { building, origin })
                {
                    command_feedback(play, error);
                }
            }
        }
        UiAction::Select(selection) => {
            play.selection = Some(selection);
            play.overlay = None;
        }
        UiAction::Deselect => {
            play.selection = None;
            play.follow = false;
        }
        UiAction::InspectorTab(tab) => play.inspector_tab = tab,
        UiAction::ToggleFollow => play.follow = !play.follow,
        UiAction::Command(command) => {
            if let Err(error) = apply_command(&mut play.sim, command) {
                command_feedback(play, error);
            }
        }
        UiAction::ZoomBy(factor) => play
            .camera
            .zoom_at(factor, vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5)),
        UiAction::Recenter => play.camera = WorldCamera::looking_at(play.sim.world.landing_tile),
        UiAction::JumpTo(tile) => play.camera.target = tile_center(tile),
        UiAction::ChronicleFilter(filter) => {
            play.chronicle_filter = filter;
            play.scroll = 0.0;
        }
        UiAction::ChronicleDay(day) => {
            play.chronicle_day = day;
            play.scroll = 0.0;
        }
        UiAction::Scroll(amount) => play.scroll = (play.scroll + amount).max(0.0),
        UiAction::FocusTech(id) => play.tech_focus = Some(id),
        UiAction::RelationFocus(id) => play.relation_focus = id,
        UiAction::HelpPage(page) => play.help_page = page,
        UiAction::TutorialNext => {
            let steps = game_data().text.tutorial.len();
            let next = play.tutorial_step.map(|s| s + 1).unwrap_or(0);
            play.overlay = None;
            play.tutorial_step = (next < steps).then_some(next);
            if play.tutorial_step.is_none() {
                finish_tutorial(preferences);
            }
        }
        UiAction::TutorialClose => {
            play.tutorial_step = None;
            finish_tutorial(preferences);
        }
        UiAction::ToggleTracker => play.tracker_open = !play.tracker_open,
        _ => {}
    }
}

fn finish_tutorial(preferences: &mut crate::state::save::Preferences) {
    if !preferences.tutorial_done {
        preferences.tutorial_done = true;
        save::save_preferences(preferences);
    }
}

pub fn apply(game: &mut Game, action: UiAction) {
    if menu_action(game, &action) {
        return;
    }
    let Game {
        screen,
        preferences,
        ..
    } = game;
    if let Screen::Playing(play) = screen {
        play_action(play, preferences, action);
    }
}
