//! game state pointer bounds domain.

use super::*;

impl GameplayState {
    pub fn pointer_inside_playable_map(&self, input: &InputState) -> bool {
        input.hovered_rect(self.world_area()) && !self.pointer_over_blocking_ui(input)
    }

    fn pointer_over_blocking_ui(&self, input: &InputState) -> bool {
        let toolbar = self.layout.bottom_toolbar();
        input.hovered_rect(toolbar)
            || (self.context_panel_open
                && input.hovered_rect(toolbar_context_rect_for_mode(toolbar, self.toolbar_mode)))
            || input.hovered_rect(camera_control_rect(self.world_area()))
            || input.hovered_rect(advisor_banner_rect(&self.layout))
            || input.mouse_pos.y <= self.layout.top_bar_height
    }
}
