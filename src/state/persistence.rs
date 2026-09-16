//! Versioned, cross-platform save DTOs for the active colony.

use super::runtime_state::GameState;
use macroquad_toolkit::persistence::{
    load_from_slot, save_to_slot_with_version_and_backup, slot_exists,
};
use serde::{Deserialize, Serialize};

pub const SAVE_GAME_NAME: &str = "finallanding";
pub const SAVE_SLOT: &str = "autosave";
pub const SAVE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveDto {
    pub schema_version: u32,
    pub game: GameState,
}

impl SaveDto {
    pub fn new(game: &GameState) -> Self {
        Self {
            schema_version: SAVE_SCHEMA_VERSION,
            game: game.clone(),
        }
    }

    pub fn validate(self) -> Result<GameState, String> {
        if self.schema_version != SAVE_SCHEMA_VERSION {
            return Err(format!(
                "Save schema {} is not supported; expected {}.",
                self.schema_version, SAVE_SCHEMA_VERSION
            ));
        }
        Ok(self.game)
    }
}

pub fn save_game(game: &GameState) -> Result<(), String> {
    let dto = SaveDto::new(game);
    save_to_slot_with_version_and_backup(SAVE_GAME_NAME, SAVE_SLOT, &dto, "1")
}

pub fn load_game() -> Result<GameState, String> {
    load_from_slot::<SaveDto>(SAVE_GAME_NAME, SAVE_SLOT)?.validate()
}

pub fn has_saved_game() -> bool {
    slot_exists(SAVE_GAME_NAME, SAVE_SLOT)
}
