//! game state queries domain.

use super::*;

impl GameplayState {
    pub fn context_panel_height(&self) -> f32 {
        match self.toolbar_mode {
            ToolbarMode::Build | ToolbarMode::Rooms | ToolbarMode::Objects => 126.0,
            ToolbarMode::Colony => 218.0,
            ToolbarMode::Research => 196.0,
            ToolbarMode::Assign => 218.0,
            ToolbarMode::Log => 276.0,
        }
    }

    pub fn world_area(&self) -> Rect {
        self.layout
            .game_area_with_height(self.context_panel_open, self.context_panel_height())
    }

    pub fn iso_view(&self) -> IsoView {
        IsoView::for_area_with_zoom(
            self.world_area(),
            self.data.grid.width as u32,
            self.data.grid.height as u32,
            self.camera_zoom,
        )
    }
}
