//! game state lifecycle domain.

use super::*;
use crate::state::persistence::save_game;
use crate::ui::introduction::{draw_arrival_overlay, ARRIVAL_STAGE_COUNT};
use crate::ui::introduction_continue_rect;

impl State for GameplayState {
    fn update(&mut self) -> StateTransition {
        self.layout.refresh();
        let mut input = InputState::capture();
        let pointer = Pointer::read(|position| position);
        if pointer.released {
            input.mouse_pos = pointer.position;
            input.left_pressed = true;
            input.left_released = true;
            input.left_click = true;
        }

        if let Some(stage) = self.arrival_stage {
            if input.left_released
                && introduction_continue_rect(screen_width(), screen_height())
                    .contains(input.mouse_pos)
            {
                if stage + 1 >= ARRIVAL_STAGE_COUNT {
                    self.complete_arrival();
                } else {
                    self.arrival_stage = Some(stage + 1);
                }
            }
            return StateTransition::None;
        }

        if self.menu_requested {
            self.menu_requested = false;
            return self.menu_transition();
        }

        // Debug toggle
        self.debug_overlay.record_frame(get_frame_time());
        if is_key_pressed(KeyCode::F3) {
            self.debug_overlay.toggle();
        }
        let keyboard_captured = self.update_social_history_search_input();

        if let Some(transition) = self.scenario_restart_transition(&input) {
            return transition;
        }

        // Time speed and priority controls (keyboard)
        if !keyboard_captured && input.space_pressed {
            self.data.time.speed = if self.data.time.speed == TimeSpeed::Paused {
                TimeSpeed::Normal
            } else {
                TimeSpeed::Paused
            };
        }
        if !keyboard_captured && is_key_pressed(KeyCode::Key1) {
            self.set_priority(ColonyPriority::Recovery);
        }
        if !keyboard_captured && is_key_pressed(KeyCode::Key2) {
            self.set_priority(ColonyPriority::Stockpile);
        }
        if !keyboard_captured && is_key_pressed(KeyCode::Key3) {
            self.set_priority(ColonyPriority::Survey);
        }

        let ui_consumed_input = self.update_pointer_ui_input(&input);
        if self.menu_requested {
            self.menu_requested = false;
            return self.menu_transition();
        }
        if !ui_consumed_input {
            self.update_colonist_selection(&input);
        }

        if self.data.scenario.is_finished() {
            return StateTransition::None;
        }

        let elapsed_ticks = self.advance_time();
        if elapsed_ticks > 0 {
            MissionSystem::process_completed_missions(&mut self.data);
            MissionSystem::recover_injured_colonists(&mut self.data);
            self.process_time_events();
            crate::game::colonist_ai::update_colonists(&mut self.data, elapsed_ticks);
            ScenarioSystem::evaluate(&mut self.data);
        } else {
            crate::game::colonist_ai::update_colonists(&mut self.data, 0);
        }

        // Update hovered cell based on mouse position (account for UI offset)
        let mouse = input.mouse_pos;
        let game_area = self.world_area();
        let grid_pos = self.iso_view().screen_to_grid(mouse);
        if game_area.contains(mouse) && self.data.grid.is_in_bounds(grid_pos.x, grid_pos.y) {
            self.hovered_cell = Some(grid_pos);
        } else {
            self.hovered_cell = None;
        }

        // Building system updates (keyboard)
        if !keyboard_captured {
            self.update_building_selection();
        }
        self.update_building_placement(&input);
        self.refresh_render_caches();
        self.maybe_autosave();

        StateTransition::None
    }

    fn draw(&self) {
        let hovered_colonist_id = self.colonist_id_at_mouse();

        // Draw game area (grid, buildings, colonists)
        self.draw_grid_with_offset();
        self.draw_buildings();
        self.draw_ghost_preview();
        self.draw_colonists_with_offset(hovered_colonist_id);
        self.draw_hover_colonist_card(hovered_colonist_id);
        let advisor_plan = AdvisorSystem::plan(&self.data);
        draw_advisor_banner(
            &self.layout,
            &advisor_plan,
            &self.data.resources,
            self.data.colonists.len(),
            self.average_mood(),
        );
        draw_camera_controls(self.world_area(), self.camera_zoom);
        if !self.context_panel_open {
            draw_colonist_inspector(
                &self.layout,
                self.inspected_colonist(hovered_colonist_id),
                &self.data.colonists,
                self.data.tick,
                &self.art,
            );
        }

        // Draw UI components (on top)
        draw_top_bar(
            &self.layout,
            self.data.tick,
            self.data.time.speed,
            self.data.colonists.len(),
            self.average_mood(),
            &self.data.resources,
        );

        let mission_plans = MissionSystem::mission_plans(&self.data);
        let objectives = ObjectiveSystem::active_cards(&self.data);
        if self.context_panel_open {
            draw_toolbar_context_panel(
                &self.layout,
                ToolbarPanelData {
                    mode: self.toolbar_mode,
                    selected_building: self.selected_building,
                    resources: &self.data.resources,
                    active_priority: self.data.priority.active,
                    objectives: &objectives,
                    research: ToolbarResearchData {
                        mission_plans: &mission_plans,
                        technology: &self.data.technology,
                        active_mission_count: self.data.missions.active_count(),
                        required_unlocks: self.data.scenario.required_tech_unlocks,
                        selected_mission_type: self.selected_mission_type,
                        has_exploration_gate: self.data.building_system.buildings().iter().any(
                            |building| building.building_type == BuildingType::ExplorationGate,
                        ),
                        has_available_mission_crew: self
                            .data
                            .colonists
                            .iter()
                            .any(|colonist| colonist.can_start_mission(self.data.tick)),
                    },
                    assign: ToolbarAssignData {
                        colonists: &self.data.colonists,
                        selected_colonist_id: self.selected_colonist_id,
                        roster_page: self.assign_roster_page,
                        roster_filter: self.assign_roster_filter,
                        roster_sort: self.assign_roster_sort,
                        role_filter: self.assign_role_filter,
                        building_filter: self.assign_building_filter,
                        room_filter_armed: self.assign_room_filter_armed,
                        pair_action_armed: self.assign_pair_armed,
                        technology: &self.data.technology,
                    },
                    log: ToolbarLogData {
                        logs: &self.data.event_log,
                        social_history: &self.data.social_history,
                        page: self.social_history_page,
                        filter: self.social_history_filter,
                        query: &self.social_history_query,
                        search_active: self.social_history_search_active,
                        selected_day: self.selected_social_history_day,
                        colony_summary: &self.cached_colony_summary,
                        timeline_rows: &self.cached_log_rows,
                        page_count: self.cached_log_page_count,
                    },
                },
            );
        }
        draw_bottom_toolbar(&self.layout, self.toolbar_mode, self.selected_building);

        // Debug overlay
        if self.debug_overlay.visible {
            draw_debug_overlay(DebugOverlayContext {
                overlay: &self.debug_overlay,
                tick: self.data.tick,
                colonists: &self.data.colonists,
                hovered_cell: self.hovered_cell,
                building_count: self.data.building_system.building_count(),
                resources: &self.data.resources,
                storage_capacity: ResourceSystem::storage_capacity(&self.data),
                daily_supply_need: ResourceSystem::daily_supply_need(&self.data),
                objective: &ScenarioSystem::objective_line(&self.data),
                outcome: self.data.scenario.outcome,
                active_mission_count: self.data.missions.active_count(),
                technology: &self.data.technology,
                priority: self.data.priority.active,
            });
        }

        self.draw_scenario_overlay();
        if let Some(stage) = self.arrival_stage {
            draw_arrival_overlay(stage);
        }
    }
}

impl GameplayState {
    fn maybe_autosave(&mut self) {
        self.autosave_elapsed += get_frame_time();
        if self.autosave_elapsed < 2.0 {
            return;
        }
        self.autosave_elapsed = 0.0;

        match save_game(&self.data) {
            Ok(()) => self.save_error_reported = false,
            Err(error) if !self.save_error_reported => {
                self.data.push_log(
                    LogCategory::System,
                    "Autosave unavailable",
                    format!("The colony is still playable, but progress was not saved: {error}"),
                );
                self.save_error_reported = true;
            }
            Err(_) => {}
        }
    }

    fn menu_transition(&mut self) -> StateTransition {
        let status_message = match save_game(&self.data) {
            Ok(()) => None,
            Err(error) => {
                self.save_error_reported = true;
                Some(format!("Save unavailable: {error}"))
            }
        };
        StateTransition::ToMenu { status_message }
    }
}
