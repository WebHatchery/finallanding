//! Saving and loading the colony through the toolkit's persistence layer.

use crate::sim::{jobs, Sim};
use macroquad_toolkit::persistence::{
    delete_json_key, json_key_exists, load_json_key, save_json_key,
};
use serde::{Deserialize, Serialize};

const GAME: &str = "finallanding";
const SLOT: &str = "colony";
const PREFERENCES: &str = "preferences";
pub const SAVE_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    sim: Sim,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Preferences {
    pub tutorial_done: bool,
    pub runs_completed: u32,
    pub endings_seen: Vec<String>,
}

pub fn has_save() -> bool {
    json_key_exists(GAME, SLOT)
}

pub fn save(sim: &Sim) -> Result<(), String> {
    let file = SaveFile {
        version: SAVE_VERSION,
        sim: sim.clone(),
    };
    save_json_key(GAME, SLOT, &file)
}

pub fn load() -> Result<Sim, String> {
    let file: SaveFile = load_json_key(GAME, SLOT)?;
    if file.version != SAVE_VERSION {
        return Err(format!("save version {} is not supported", file.version));
    }
    let mut sim = file.sim;
    sim.colony.refresh_derived();
    jobs::refresh(&mut sim);
    Ok(sim)
}

pub fn delete() {
    if let Err(error) = delete_json_key(GAME, SLOT) {
        eprintln!("could not delete save: {error}");
    }
}

pub fn load_preferences() -> Preferences {
    load_json_key(GAME, PREFERENCES).unwrap_or_default()
}

pub fn save_preferences(preferences: &Preferences) {
    if let Err(error) = save_json_key(GAME, PREFERENCES, preferences) {
        eprintln!("could not save preferences: {error}");
    }
}
