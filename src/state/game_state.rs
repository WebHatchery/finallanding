//! game state domain.

use crate::data::building::{Building, BuildingType};
use crate::data::colonist::{ActivityLocation, Colonist, ColonistState, JobPreference};
use crate::data::event_log::{LogCategory, SocialHistoryEntry, SocialHistoryMetrics};
use crate::data::game_state::TimeSpeed;
use crate::data::mission::MissionType;
use crate::data::priority::ColonyPriority;
use crate::data::types::Position;
use crate::game::building_system::PlacementResult;
use crate::state::persistence::save_game;
use crate::state::runtime_state::GameState;
use crate::state::{State, StateTransition};
use crate::systems::advisor_system::AdvisorSystem;
use crate::systems::assignment_system::AssignmentSystem;
use crate::systems::incident_system::IncidentSystem;
use crate::systems::mission_system::MissionSystem;
use crate::systems::objective_system::ObjectiveSystem;
use crate::systems::planning_system::PlanningSystem;
use crate::systems::proximity_system::ProximitySystem;
use crate::systems::relationship_directive_system::{
    DirectiveChange, PairDirective, RelationshipDirectiveSystem,
};
use crate::systems::resource_system::ResourceSystem;
use crate::systems::scenario_system::ScenarioSystem;
use crate::systems::social_system::SocialSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_events::TimeEventCollector;
use crate::systems::time_system::TimeSystem;
use crate::systems::work_system::WorkSystem;
use crate::ui::{
    advisor_banner_rect, assign_batch_action_at, assign_filter_at, assign_page_action_at,
    assign_pair_action_rect, assign_role_action_rect, assign_role_filter_at,
    assign_room_filter_rect, assign_sort_at, camera_action_at, camera_control_rect,
    draw_advisor_banner, draw_bottom_toolbar, draw_camera_controls, draw_colonist_inspector,
    draw_debug_overlay, draw_toolbar_context_panel, draw_top_bar, log_filter_at,
    log_keyboard_action_at, log_keyboard_bounds, log_page_action_at, log_report_close_rect,
    log_search_action_at, log_section_at, log_timeline_row_at, research_action_rect,
    restart_button_rect, social_history_page_count, social_timeline_day_at,
    toolbar_building_at_for_mode, toolbar_buildings_for_mode, toolbar_colonist_index_at,
    toolbar_context_rect_for_mode, toolbar_mission_at, toolbar_mode_at, toolbar_priority_at,
    top_bar_action_at, top_bar_priority_at_for, top_bar_speed_at_for, AssignBatchAction,
    AssignRosterFilter, AssignRosterSort, CameraAction, DebugOverlayContext, IsoView, Layout,
    LogFilter, LogSearchAction, LogSectionAction, PageAction, PlaceholderArt, SocialTimelineRow,
    ToolbarAssignData, ToolbarLogData, ToolbarMode, ToolbarPanelData, ToolbarResearchData,
    TopBarAction,
};
use macroquad::prelude::*;
use macroquad_toolkit::debug::DebugOverlay;
use macroquad_toolkit::input::InputState;
use macroquad_toolkit::ui::Pointer;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ActionFeedback {
    pub category: LogCategory,
    pub title: String,
    pub detail: String,
    pub remaining_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LogCacheKey {
    history_len: usize,
    latest_day: Option<u32>,
    filter: LogFilter,
    query: String,
    page: usize,
    selected_day: Option<u32>,
}

pub struct GameplayState {
    pub data: GameState,
    /// Current step in the first-run arrival briefing; None means the colony is active.
    pub arrival_stage: Option<usize>,
    pub hovered_cell: Option<Position>,
    /// Currently selected building type for placement (None = not in build mode)
    pub selected_building: Option<BuildingType>,
    /// Fixed preview grid position used only by screenshot verification captures.
    pub capture_preview_position: Option<Position>,
    /// Selected colonist for relationship inspection.
    pub selected_colonist_id: Option<u32>,
    /// Time event collector for processing time-based events
    pub time_events: TimeEventCollector,
    /// Previous tick for event detection
    pub prev_tick: u64,
    /// Accumulates real time before advancing the simulation by game ticks
    pub time_accumulator: f32,
    /// UI layout configuration
    pub layout: Layout,
    /// Smoothed FPS/frame-time overlay with colony stat lines, toggled by F3.
    pub debug_overlay: DebugOverlay,
    /// Active bottom-toolbar mode.
    pub toolbar_mode: ToolbarMode,
    /// Whether the current mode's context tray is open over the lower world area.
    pub context_panel_open: bool,
    /// Bounded observation zoom; placement and picking use the same view.
    pub camera_zoom: f32,
    /// Bounded camera translation used by the map drag gesture.
    pub camera_offset: Vec2,
    /// Last pointer position while a map drag is in progress.
    pub camera_drag_last: Option<Vec2>,
    /// Whether the current map gesture moved far enough to consume selection.
    pub camera_drag_moved: bool,
    /// Current page in the Assign mode roster.
    pub assign_roster_page: usize,
    /// Active filter in the Assign mode roster.
    pub assign_roster_filter: AssignRosterFilter,
    /// Active sort in the Assign mode roster.
    pub assign_roster_sort: AssignRosterSort,
    /// Optional work-role filter in the Assign mode roster.
    pub assign_role_filter: Option<JobPreference>,
    /// Optional room/work-space instance filter in the Assign mode roster.
    pub assign_building_filter: Option<u32>,
    /// Whether the visible room filter control is waiting for a map tap.
    pub assign_room_filter_armed: bool,
    /// Whether the next roster tap should set or clear a relationship directive.
    pub assign_pair_armed: bool,
    /// Current page in the Log mode social archive.
    pub social_history_page: usize,
    /// Current page in the general event history.
    pub event_history_page: usize,
    /// Whether Log is showing general events instead of social reports.
    pub show_event_history: bool,
    /// Active filter in the Log mode social archive.
    pub social_history_filter: LogFilter,
    /// Search query for the Log mode social archive.
    pub social_history_query: String,
    /// Whether typed keys should edit the Log mode social archive search.
    pub social_history_search_active: bool,
    /// Selected daily social report for persistent Log drilldown.
    pub selected_social_history_day: Option<u32>,
    /// Production visual assets plus deterministic pose overlays for the colony.
    pub art: PlaceholderArt,
    /// Seconds since the last durable autosave.
    pub autosave_elapsed: f32,
    /// Prevents a storage failure from flooding the event log every frame.
    pub save_error_reported: bool,
    /// Set by a visible menu action so the lifecycle can save before leaving.
    pub menu_requested: bool,
    /// Whether the finished-scenario result is currently showing the Log view.
    pub result_review_open: bool,
    /// Whether the contextual help disclosure is open.
    pub help_open: bool,
    /// Brief acknowledgement for the latest player-visible event.
    pub action_feedback: Option<ActionFeedback>,
    /// Event-log length already surfaced in the transient acknowledgement.
    pub feedback_seen_log_len: usize,
    /// Mission card selected in the Research tray before an explicit launch.
    pub selected_mission_type: MissionType,
    /// Cached summary input fingerprint used to avoid repeated relationship scans during draw.
    pub cached_summary_key: u64,
    pub cached_colony_summary: crate::systems::summary_system::ColonyPressureSummary,
    /// Cached Log rows and pagination for the current filter/search/page selection.
    cached_log_key: Option<LogCacheKey>,
    pub cached_log_rows: Vec<SocialTimelineRow>,
    pub cached_log_page_count: usize,
}

impl Default for GameplayState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameplayState {
    pub fn new() -> Self {
        let mut data = GameState::new();
        data.tick = 420; // Start at 07:00 AM (Work time)
        Self::from_data(data, Some(0))
    }

    /// Create a fully populated state for deterministic screenshot and playthrough harnesses.
    pub fn new_for_capture() -> Self {
        let mut state = Self::new();
        state.complete_arrival();
        state.context_panel_open = true;
        state
    }

    /// Restore a saved colony without replaying the first-run arrival briefing.
    pub fn from_saved(data: GameState) -> Self {
        Self::from_data(data, None)
    }

    fn from_data(data: GameState, arrival_stage: Option<usize>) -> Self {
        let toolbar_mode = initial_toolbar_mode();
        let selected_building = initial_selected_building(toolbar_mode);
        let selected_colonist_id = initial_selected_colonist_id(&data, toolbar_mode);
        let capture_preview_position = initial_capture_preview_position();
        let selected_social_history_day = initial_selected_social_history_day(&data);
        let selected_mission_type = MissionSystem::recommended_mission_type(&data);
        let show_event_history = initial_log_view_is_events();
        let feedback_seen_log_len = data.event_log.len();

        let mut state = Self {
            prev_tick: data.tick,
            data,
            arrival_stage,
            hovered_cell: None,
            selected_building,
            capture_preview_position,
            selected_colonist_id,
            time_events: TimeEventCollector::new(),
            time_accumulator: 0.0,
            layout: Layout::default(),
            debug_overlay: DebugOverlay::new(),
            toolbar_mode,
            context_panel_open: false,
            camera_zoom: 1.0,
            camera_offset: Vec2::ZERO,
            camera_drag_last: None,
            camera_drag_moved: false,
            assign_roster_page: 0,
            assign_roster_filter: AssignRosterFilter::All,
            assign_roster_sort: AssignRosterSort::Roster,
            assign_role_filter: None,
            assign_building_filter: None,
            assign_room_filter_armed: false,
            assign_pair_armed: false,
            social_history_page: 0,
            event_history_page: 0,
            show_event_history,
            social_history_filter: LogFilter::All,
            social_history_query: String::new(),
            social_history_search_active: false,
            selected_social_history_day,
            art: PlaceholderArt::new(),
            autosave_elapsed: 0.0,
            save_error_reported: false,
            menu_requested: false,
            result_review_open: false,
            help_open: false,
            action_feedback: None,
            feedback_seen_log_len,
            selected_mission_type,
            cached_summary_key: 0,
            cached_colony_summary: crate::systems::summary_system::ColonyPressureSummary {
                average_mood: 0.0,
                average_relationship: 0.0,
                close_pairs: 0,
                strained_pairs: 0,
                connected_pairs: Vec::new(),
                tense_pairs: Vec::new(),
                strongest_pair: None,
                weakest_pair: None,
            },
            cached_log_key: None,
            cached_log_rows: Vec::new(),
            cached_log_page_count: 1,
        };
        state.refresh_render_caches();
        state
    }

    pub fn refresh_render_caches(&mut self) {
        let summary_key = self.summary_input_key();
        if summary_key != self.cached_summary_key {
            self.cached_colony_summary = SummarySystem::colony_pressure_summary(&self.data);
            self.cached_summary_key = summary_key;
        }

        let log_key = LogCacheKey {
            history_len: self.data.social_history.len(),
            latest_day: self.data.social_history.last().map(|entry| entry.day),
            filter: self.social_history_filter,
            query: self.social_history_query.clone(),
            page: self.social_history_page,
            selected_day: self.selected_social_history_day,
        };
        if self.cached_log_key.as_ref() != Some(&log_key) {
            self.cached_log_page_count = social_history_page_count(
                &self.data.social_history,
                self.social_history_filter,
                &self.social_history_query,
            );
            self.cached_log_rows = crate::ui::toolbar_panel::log_model::social_timeline_rows(
                &self.data.social_history,
                self.social_history_filter,
                &self.social_history_query,
                self.social_history_page,
            );
            self.cached_log_key = Some(log_key);
        }
    }

    fn summary_input_key(&self) -> u64 {
        let mut key = self.data.tick;
        for colonist in &self.data.colonists {
            key = key
                .wrapping_mul(31)
                .wrapping_add(colonist.id as u64)
                .wrapping_add(colonist.mood.to_bits() as u64);
            for value in colonist.relationships.values() {
                key = key.wrapping_mul(31).wrapping_add(*value as u64);
            }
        }
        let priority_key = match self.data.priority.active {
            ColonyPriority::Recovery => 1,
            ColonyPriority::Stockpile => 2,
            ColonyPriority::Survey => 3,
        };
        key.wrapping_add(self.data.building_system.building_count() as u64)
            .wrapping_add(self.data.resources.supplies as u64)
            .wrapping_add(self.data.resources.salvage as u64)
            .wrapping_add(priority_key)
    }

    /// Materialize the configured survivor roster and arrival-day capture fixtures once the
    /// player has read the briefing.
    pub fn complete_arrival(&mut self) {
        if self.arrival_stage.is_none() {
            return;
        }

        crate::game::colonist_spawner::spawn_initial_colonists(&mut self.data);
        self.data.push_log(
            LogCategory::System,
            "Crash survivors assembled",
            format!(
                "Starting stockpile: {} supplies, {} salvage. Objective: survive to Day {} and unlock {} technologies.",
                self.data.resources.supplies,
                self.data.resources.salvage,
                self.data.scenario.target_day,
                self.data.scenario.required_tech_unlocks
            ),
        );
        seed_assign_spaces_for_capture(&mut self.data);
        seed_activity_poses_for_capture(&mut self.data);
        seed_social_history_for_capture(&mut self.data);
        self.selected_colonist_id = initial_selected_colonist_id(&self.data, self.toolbar_mode);
        self.selected_social_history_day = initial_selected_social_history_day(&self.data);
        self.arrival_stage = None;
        self.autosave_elapsed = 0.0;
        self.save_error_reported = false;
        if let Err(error) = save_game(&self.data) {
            self.data.push_log(
                LogCategory::System,
                "Autosave unavailable",
                format!("The colony is still playable, but progress was not saved: {error}"),
            );
            self.save_error_reported = true;
        }
        self.refresh_render_caches();
    }
}

#[path = "game_state_assign_batch_commands.rs"]
mod game_state_assign_batch_commands;
#[path = "game_state_assignment_batch_rules.rs"]
pub mod game_state_assignment_batch_rules;
pub use game_state_assignment_batch_rules::*;
#[path = "game_state_assign_filter_commands.rs"]
mod game_state_assign_filter_commands;
#[path = "game_state_assign_roster_commands.rs"]
mod game_state_assign_roster_commands;
#[path = "game_state_assign_space_commands.rs"]
mod game_state_assign_space_commands;
#[path = "game_state_assignment_roster.rs"]
pub mod game_state_assignment_roster;
pub use game_state_assignment_roster::*;
#[path = "game_state_assignment_space_rules.rs"]
pub mod game_state_assignment_space_rules;
pub use game_state_assignment_space_rules::*;
#[path = "game_state_building_commands.rs"]
mod game_state_building_commands;
#[path = "game_state_keyboard_input.rs"]
mod game_state_keyboard_input;
#[path = "game_state_lifecycle.rs"]
mod game_state_lifecycle;
#[path = "game_state_log_commands.rs"]
mod game_state_log_commands;
#[path = "game_state_map_selection.rs"]
mod game_state_map_selection;
#[path = "game_state_mission_commands.rs"]
mod game_state_mission_commands;
#[path = "game_state_placement_results.rs"]
mod game_state_placement_results;
pub use game_state_placement_results::*;
#[path = "game_state_pointer_bounds.rs"]
mod game_state_pointer_bounds;
#[path = "game_state_priority_commands.rs"]
mod game_state_priority_commands;
#[path = "game_state_queries.rs"]
mod game_state_queries;
#[path = "game_state_relationship_commands.rs"]
mod game_state_relationship_commands;
#[path = "game_state_relationship_contact.rs"]
pub mod game_state_relationship_contact;
pub use game_state_relationship_contact::*;
#[path = "game_state_relationship_directive_logs.rs"]
mod game_state_relationship_directive_logs;
pub use game_state_relationship_directive_logs::*;
#[path = "game_state_setup.rs"]
pub mod game_state_setup;
pub use game_state_setup::*;
#[path = "game_state_simulation.rs"]
mod game_state_simulation;
#[path = "game_state_social_archive.rs"]
pub mod game_state_social_archive;
pub use game_state_social_archive::*;
#[path = "game_state_text.rs"]
mod game_state_text;
pub use game_state_text::*;
#[path = "game_state_toolbar_input.rs"]
mod game_state_toolbar_input;
