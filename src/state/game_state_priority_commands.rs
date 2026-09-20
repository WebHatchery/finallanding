//! game state priority commands domain.

use super::*;

impl GameplayState {
    pub fn set_priority(&mut self, priority: ColonyPriority) {
        if self.data.priority.active == priority {
            return;
        }

        self.data.priority.active = priority;
        self.data.push_log(
            LogCategory::System,
            format!("Priority set: {}", priority.label()),
            priority.description(),
        );
    }

    // Handle building placement via mouse click

    pub fn handle_colony_toolbar_click(&mut self, context: Rect, mouse_x: f32, mouse_y: f32) {
        if let Some(priority) = toolbar_priority_at(context, mouse_x, mouse_y) {
            self.set_priority(priority);
        }
    }

    pub fn update_top_bar_click(&mut self, mouse_x: f32, mouse_y: f32) {
        if let Some(action) = top_bar_action_at(&self.layout, mouse_x, mouse_y) {
            match action {
                TopBarAction::Undo => self.undo_last_building(),
                TopBarAction::Cancel => {
                    self.selected_building = None;
                    self.assign_room_filter_armed = false;
                    self.assign_pair_armed = false;
                    self.social_history_search_active = false;
                }
                TopBarAction::Menu => self.menu_requested = true,
            }
            return;
        }

        if let Some(speed) = top_bar_speed_at_for(&self.layout, mouse_x, mouse_y) {
            self.data.time.speed = speed;
            return;
        }

        if let Some(priority) = top_bar_priority_at_for(&self.layout, mouse_x, mouse_y) {
            self.set_priority(priority);
        }
    }
}
