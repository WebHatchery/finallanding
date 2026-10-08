//! Application screens and their state: title, colony setup, play, results.

pub mod play;
pub mod save;

use crate::colony::RunSetup;
use play::PlayState;

pub struct TitleState {
    pub has_save: bool,
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SetupState {
    pub site: String,
    pub difficulty: String,
    pub seed: u64,
}

impl SetupState {
    pub fn to_run(&self) -> RunSetup {
        RunSetup {
            colony_name: "New Meridian".into(),
            site: self.site.clone(),
            difficulty: self.difficulty.clone(),
            seed: self.seed,
        }
    }
}

pub enum Screen {
    Title(TitleState),
    Setup(SetupState),
    Playing(Box<PlayState>),
}
