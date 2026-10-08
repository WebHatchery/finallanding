//! The Final Landing: a colony simulation of autonomous survivors.
//!
//! The binary is a thin event loop; the simulation, data and UI live here so
//! integration tests can exercise the rules through the public API.

pub mod agents;
pub mod autoplay;
pub mod colony;
pub mod data;
pub mod game;
pub mod sim;
pub mod state;
pub mod ui;
pub mod world;
