//! game state toolbar input domain.

use super::*;

impl GameplayState {
    pub fn update_pointer_ui_input(&mut self, input: &InputState) -> bool {
        let assign_room_filter_click = self.context_panel_open
            && self.toolbar_mode == ToolbarMode::Assign
            && (input.right_released || self.assign_room_filter_armed && input.left_released);
        let map_click = assign_room_filter_click && input.hovered_rect(self.world_area());

        let mouse_x = input.mouse_pos.x;
        let mouse_y = input.mouse_pos.y;

        if map_click {
            self.update_assign_building_filter_click();
            self.assign_room_filter_armed = false;
            return true;
        }

        if !input.left_released {
            return false;
        }

        if mouse_y <= self.layout.top_bar_height {
            self.update_top_bar_click(mouse_x, mouse_y);
            return true;
        }

        if let Some(action) = camera_action_at(self.world_area(), mouse_x, mouse_y) {
            self.update_camera_action(action);
            return true;
        }

        if self.update_toolbar_click(mouse_x, mouse_y) {
            return true;
        }

        false
    }

    fn update_camera_action(&mut self, action: CameraAction) {
        match action {
            CameraAction::ZoomOut => self.camera_zoom = (self.camera_zoom - 0.1).max(0.8),
            CameraAction::ZoomIn => self.camera_zoom = (self.camera_zoom + 0.1).min(1.25),
            CameraAction::Recenter => self.camera_zoom = 1.0,
        }
    }

    pub fn update_toolbar_click(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        let toolbar = self.layout.bottom_toolbar();
        if let Some(mode) = toolbar_mode_at(toolbar, mouse_x, mouse_y) {
            if self.toolbar_mode == mode {
                self.context_panel_open = !self.context_panel_open;
            } else {
                self.context_panel_open = true;
            }
            if mode != ToolbarMode::Assign || !self.context_panel_open {
                self.assign_room_filter_armed = false;
                self.assign_pair_armed = false;
            }
            self.toolbar_mode = mode;
            if !mode.uses_building_choices()
                || self
                    .selected_building
                    .is_some_and(|building| !toolbar_buildings_for_mode(mode).contains(&building))
            {
                self.selected_building = None;
            }
            return true;
        }

        let context = toolbar_context_rect_for_mode(toolbar, self.toolbar_mode);
        let in_touch_keyboard = self.toolbar_mode == ToolbarMode::Log
            && self.social_history_search_active
            && log_keyboard_bounds(context).contains(Vec2::new(mouse_x, mouse_y));
        if !self.context_panel_open
            || (!context.contains(Vec2::new(mouse_x, mouse_y)) && !in_touch_keyboard)
        {
            return false;
        }

        match self.toolbar_mode {
            ToolbarMode::Build | ToolbarMode::Rooms | ToolbarMode::Objects => {
                self.handle_build_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Colony => {
                self.handle_colony_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Research => {
                self.handle_research_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Assign => {
                self.handle_assign_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Log => {
                self.handle_log_toolbar_click(context, mouse_x, mouse_y);
            }
        }

        true
    }
}
