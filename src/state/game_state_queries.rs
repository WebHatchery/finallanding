//! game state queries domain.

use super::*;

impl GameplayState {
    pub fn iso_view(&self) -> IsoView {
        IsoView::for_area(
            self.layout.game_area(),
            self.data.grid.width as u32,
            self.data.grid.height as u32,
        )
    }
}
