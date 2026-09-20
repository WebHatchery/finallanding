//! hit zones domain.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAction {
    Previous,
    Next,
}

pub mod assign;
pub mod camera;
pub mod log;
pub mod menu;
pub mod toolbar;
pub mod top_bar;

pub use assign::*;
pub use camera::*;
pub use log::*;
pub use menu::*;
pub use toolbar::*;
pub use top_bar::*;
