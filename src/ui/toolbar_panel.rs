//! toolbar panel domain.

use super::Layout;
use crate::data::building::BuildingType;
use crate::data::colonist::{Colonist, JobPreference};
use crate::data::event_log::{ColonyLogEntry, LogCategory, SocialHistoryEntry};
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::data::technology::{TechId, TechnologyState};
use crate::systems::assignment_system::AssignmentSystem;
use crate::systems::mission_system::MissionPlan;
use crate::systems::summary_system::ColonyPressureSummary;
use crate::ui::hit_zones::{
    assign_batch_rect, assign_filter_rect, assign_page_next_rect, assign_page_previous_rect,
    assign_role_filter_rect, assign_room_filter_rect, assign_sort_rect, log_filter_rect,
    log_keyboard_bounds, log_keyboard_key_rect, log_page_next_rect, log_page_previous_rect,
    log_search_clear_rect, log_search_export_rect, log_search_rect, log_timeline_row_rect,
    toolbar_buildings_for_mode, toolbar_context_item_rect, toolbar_context_rect,
    toolbar_list_item_rect, AssignBatchAction, AssignRosterFilter, AssignRosterSort, LogFilter,
    ToolbarMode,
};
use crate::ui::style;
use crate::ui::tooltip::draw_tooltip_near_mouse;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub const SOCIAL_TIMELINE_PAGE_SIZE: usize = 3;

mod assign;
pub mod assign_model;
mod log;
pub mod log_model;
mod shared;

use assign::*;
use assign_model::*;
use log::*;
use log_model::*;
use shared::*;

pub use log_model::{social_history_page_count, social_timeline_day_at, SocialTimelineRow};

pub struct ToolbarPanelData<'a> {
    pub mode: ToolbarMode,
    pub selected_building: Option<BuildingType>,
    pub resources: &'a ResourceState,
    pub active_priority: ColonyPriority,
    pub research: ToolbarResearchData<'a>,
    pub assign: ToolbarAssignData<'a>,
    pub log: ToolbarLogData<'a>,
}

pub struct ToolbarResearchData<'a> {
    pub mission_plans: &'a [MissionPlan],
    pub technology: &'a TechnologyState,
    pub active_mission_count: usize,
    pub required_unlocks: usize,
}

pub struct ToolbarAssignData<'a> {
    pub colonists: &'a [Colonist],
    pub selected_colonist_id: Option<u32>,
    pub roster_page: usize,
    pub roster_filter: AssignRosterFilter,
    pub roster_sort: AssignRosterSort,
    pub role_filter: Option<JobPreference>,
    pub building_filter: Option<u32>,
    pub room_filter_armed: bool,
    pub technology: &'a TechnologyState,
}

pub struct ToolbarLogData<'a> {
    pub logs: &'a [ColonyLogEntry],
    pub social_history: &'a [SocialHistoryEntry],
    pub page: usize,
    pub filter: LogFilter,
    pub query: &'a str,
    pub search_active: bool,
    pub selected_day: Option<u32>,
    pub colony_summary: &'a ColonyPressureSummary,
    pub timeline_rows: &'a [SocialTimelineRow],
    pub page_count: usize,
}

pub fn draw_toolbar_context_panel(layout: &Layout, panel: ToolbarPanelData<'_>) {
    let context = toolbar_context_rect(layout.bottom_toolbar());
    style::draw_panel(context);
    style::draw_section_title(
        panel.mode.label().to_uppercase().as_str(),
        context.x + 14.0,
        context.y + 27.0,
    );

    match panel.mode {
        ToolbarMode::Build | ToolbarMode::Rooms | ToolbarMode::Objects => draw_build_context(
            context,
            panel.mode,
            panel.selected_building,
            panel.resources,
        ),
        ToolbarMode::Colony => draw_colony_context(context, panel.active_priority),
        ToolbarMode::Research => draw_research_context(
            context,
            panel.research.mission_plans,
            panel.research.technology,
            panel.research.active_mission_count,
            panel.research.required_unlocks,
        ),
        ToolbarMode::Assign => draw_assign_context(AssignContext {
            context,
            colonists: panel.assign.colonists,
            selected_colonist_id: panel.assign.selected_colonist_id,
            assign_roster_page: panel.assign.roster_page,
            active_filter: panel.assign.roster_filter,
            active_sort: panel.assign.roster_sort,
            active_role_filter: panel.assign.role_filter,
            active_building_filter: panel.assign.building_filter,
            technology: panel.assign.technology,
            room_filter_armed: panel.assign.room_filter_armed,
        }),
        ToolbarMode::Log => draw_log_context(LogContext {
            context,
            logs: panel.log.logs,
            social_history: panel.log.social_history,
            social_history_page: panel.log.page,
            social_history_filter: panel.log.filter,
            social_history_query: panel.log.query,
            social_history_search_active: panel.log.search_active,
            selected_social_history_day: panel.log.selected_day,
            summary: panel.log.colony_summary,
            timeline_rows: panel.log.timeline_rows,
            page_count: panel.log.page_count,
        }),
    }
}

fn draw_build_context(
    context: Rect,
    mode: ToolbarMode,
    selected_building: Option<BuildingType>,
    resources: &ResourceState,
) {
    let text = &crate::data::config::game_config().text;
    let mut hovered_building = None;
    for (index, building_type) in toolbar_buildings_for_mode(mode).iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let can_afford = resources.salvage >= building_type.salvage_cost();
        let selected = selected_building == Some(*building_type);
        let hovered = style::button_hovered(rect);
        if hovered {
            hovered_building = Some(*building_type);
        }
        style::draw_button(rect, selected, hovered);
        let (r, g, b) = building_type.color();
        draw_rectangle(
            rect.x + 9.0,
            rect.y + 9.0,
            14.0,
            14.0,
            Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0),
        );
        draw_ui_text(
            &style::fit_text(building_type.name(), rect.w - 40.0, style::TINY_SIZE),
            rect.x + 30.0,
            rect.y + 18.0,
            style::TINY_SIZE,
            if can_afford {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
        draw_ui_text(
            &text
                .label("toolbar_cost")
                .replace("{}", &building_type.salvage_cost().to_string()),
            rect.x + 30.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            if can_afford {
                style::TEXT_BODY
            } else {
                style::ALERT_RED
            },
        );
    }

    let helper = match mode {
        ToolbarMode::Rooms => text.label("toolbar_rooms_helper"),
        ToolbarMode::Objects => text.label("toolbar_objects_helper"),
        _ => text.label("toolbar_build_helper"),
    };
    draw_ui_text(
        helper,
        context.x + 18.0,
        context.y + 111.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    if let Some(building_type) = hovered_building {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            building_type.name(),
            &format!(
                "{} Cost: {} salvage.",
                building_type.placement_impact(),
                building_type.salvage_cost()
            ),
        );
    }
}

fn draw_colony_context(context: Rect, active_priority: ColonyPriority) {
    let mut hovered_priority = None;
    for (index, priority) in ColonyPriority::all().iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let hovered = style::button_hovered(rect);
        if hovered {
            hovered_priority = Some(*priority);
        }
        style::draw_button(rect, *priority == active_priority, hovered);
        draw_ui_text(
            priority.short_label(),
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_ui_text(
            &style::fit_text(priority.description(), rect.w - 20.0, style::TINY_SIZE),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    if let Some(priority) = hovered_priority {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            priority.label(),
            priority.description(),
        );
    }
}

fn draw_research_context(
    context: Rect,
    mission_plans: &[MissionPlan],
    technology: &TechnologyState,
    active_mission_count: usize,
    required_unlocks: usize,
) {
    let mut hovered_plan = None;
    for (index, plan) in mission_plans.iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let hovered = style::button_hovered(rect);
        if hovered {
            hovered_plan = Some(plan);
        }
        style::draw_button(rect, plan.recommended, hovered);
        draw_ui_text(
            plan.definition.short_name,
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_ui_text(
            &format!(
                "{}m | {}%",
                plan.definition.duration_minutes, plan.danger_percent
            ),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    for (slot, tech_id) in technology.visible_research_targets(2).iter().enumerate() {
        let rect = toolbar_context_item_rect(context, slot + 3);
        let hovered = style::button_hovered(rect);
        style::draw_button(rect, false, hovered);
        draw_ui_text(
            &style::fit_text(tech_id.name(), rect.w - 20.0, style::SMALL_SIZE),
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_ui_text(
            &style::fit_text(
                &technology.requirement_progress_text(*tech_id),
                rect.w - 20.0,
                style::TINY_SIZE,
            ),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
        if hovered {
            draw_tooltip_near_mouse(
                toolbar_tooltip_bounds(context),
                tech_id.name(),
                &format!(
                    "{} {}",
                    tech_id.effect_text(),
                    technology.requirement_progress_text(*tech_id)
                ),
            );
        }
    }

    let tech_label = technology
        .next_research_target()
        .map(|tech| tech.name())
        .unwrap_or_else(|| {
            crate::data::config::game_config()
                .text
                .label("toolbar_all_tech")
        });
    let research_status = crate::data::config::game_config()
        .text
        .label("toolbar_research_status")
        .replacen("{}", &active_mission_count.to_string(), 1)
        .replacen("{}", &technology.unlocked_count().to_string(), 1)
        .replacen("{}", &required_unlocks.to_string(), 1)
        .replacen("{}", &technology.unlocked_count().to_string(), 1)
        .replacen("{}", &TechId::all().len().to_string(), 1)
        .replacen("{}", tech_label, 1);
    draw_ui_text(
        &research_status,
        context.x + 18.0,
        context.y + 109.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    if let Some(plan) = hovered_plan {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            plan.definition.name,
            &format!(
                "{} {}",
                plan.definition.description, plan.definition.reward_profile
            ),
        );
    }
}
