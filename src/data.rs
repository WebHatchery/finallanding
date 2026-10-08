//! Data definitions and the embedded JSON catalog. Data types know nothing of
//! the simulation or UI.

pub mod balance;
pub mod buildings;
pub mod campaign;
pub mod catalog;
pub mod events;
pub mod finds;
pub mod kinds;
pub mod people;
pub mod resources;
pub mod society;
pub mod techs;
pub mod validation;

pub use catalog::{fill_template, game_data, GameData};
pub use kinds::*;
pub use resources::{Resource, ResourceBag};
