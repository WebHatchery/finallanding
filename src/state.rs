//! state domain.

pub mod game_state;
pub mod menu_state;
pub mod runtime_state;

use crate::state::game_state::GameplayState;

pub enum StateTransition {
    None,
    ToGameplay(Box<GameplayState>),
}

pub trait State {
    fn update(&mut self) -> StateTransition;
    fn draw(&self);
}
