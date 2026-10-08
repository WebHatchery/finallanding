//! The interface, drawn into a fixed 1920×1080 virtual canvas. UI code reads
//! state and returns `UiAction`s; it never changes the simulation.

pub mod actions;
pub mod art;
pub mod build_panel;
pub mod camera;
pub mod context;
pub mod describe;
pub mod hud;
pub mod input;
pub mod inspector;
pub mod inspector_agent;
pub mod overlays;
pub mod play_screen;
pub mod screens;
pub mod theme;
pub mod tracker;
pub mod world_view;
