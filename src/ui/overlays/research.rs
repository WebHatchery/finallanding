//! Research overlay: the technologies this colony knows, laid out from what
//! it has found, the native species that inspired them, and the selected
//! technology's details. Unknown and absent technologies are not drawn, so
//! each run's tree grows into its own shape.

mod detail;

use crate::colony::tech_tree::TechStatus;
use crate::data::finds::FindDef;
use crate::data::{fill_template, game_data, Branch};
use crate::state::play::PlayState;
use crate::ui::actions::UiAction;
use crate::ui::context::*;
use crate::ui::theme::*;
use macroquad::prelude::*;
use std::collections::HashMap;

const SIDEBAR: f32 = 210.0;
const DETAIL: f32 = 470.0;
const FINDS_HEIGHT: f32 = 128.0;
const CARD_HEIGHT: f32 = 30.0;
const CARD_GAP: f32 = 5.0;
const MAX_CARD_WIDTH: f32 = 236.0;
const CHIP_WIDTH: f32 = 262.0;
const CHIP_HEIGHT: f32 = 36.0;

fn status_color(status: TechStatus) -> Color {
    match status {
        TechStatus::Researched => GOOD,
        TechStatus::Available => TEXT,
        TechStatus::Waiting => TEXT_DIM,
        TechStatus::Hidden | TechStatus::Absent => TEXT_FAINT,
        TechStatus::Locked => Color::new(0.4, 0.3, 0.3, 1.0),
    }
}

fn find_color(find: &FindDef) -> Color {
    Color::from_rgba(find.color[0], find.color[1], find.color[2], 255)
}

fn is_shown(status: TechStatus) -> bool {
    !matches!(status, TechStatus::Hidden | TechStatus::Absent)
}

/// The area the technology cards share, above the finds strip.
fn grid_area(content: Rect) -> Rect {
    Rect::new(
        content.x + SIDEBAR,
        content.y,
        content.w - SIDEBAR - DETAIL - 20.0,
        content.h - FINDS_HEIGHT,
    )
}

/// Known technologies of each branch in tier order, broken into columns: a
/// new tier starts a new column, and a column holds as many cards as a
/// branch row fits.
fn branch_columns(play: &PlayState, branch: Branch, per_column: usize) -> Vec<Vec<String>> {
    let tree = &play.sim.colony.tree;
    let mut columns: Vec<Vec<String>> = Vec::new();
    for tier in 1..=5u8 {
        let known: Vec<String> = game_data()
            .techs
            .iter()
            .filter(|t| t.branch == branch && t.tier == tier && is_shown(tree.status(&t.id)))
            .map(|t| t.id.clone())
            .collect();
        for chunk in known.chunks(per_column.max(1)) {
            columns.push(chunk.to_vec());
        }
    }
    columns
}

/// Card rectangles for every shown technology.
fn layout(play: &PlayState, content: Rect) -> HashMap<String, Rect> {
    let grid = grid_area(content);
    let row_h = grid.h / Branch::ALL.len() as f32;
    let per_column = ((row_h - 10.0) / (CARD_HEIGHT + CARD_GAP)).floor() as usize;
    let rows: Vec<Vec<Vec<String>>> = Branch::ALL
        .iter()
        .map(|b| branch_columns(play, *b, per_column))
        .collect();
    let widest = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let column_w = (grid.w / widest as f32).min(MAX_CARD_WIDTH);
    let mut rects = HashMap::new();
    for (row, columns) in rows.iter().enumerate() {
        for (column, ids) in columns.iter().enumerate() {
            for (index, id) in ids.iter().enumerate() {
                let x = grid.x + column as f32 * column_w + 6.0;
                let y = grid.y + row as f32 * row_h + 6.0 + index as f32 * (CARD_HEIGHT + CARD_GAP);
                rects.insert(id.clone(), Rect::new(x, y, column_w - 12.0, CARD_HEIGHT));
            }
        }
    }
    rects
}

fn draw_sidebar(play: &PlayState, content: Rect) {
    let data = game_data();
    let grid = grid_area(content);
    let row_h = grid.h / Branch::ALL.len() as f32;
    let threshold = data.balance.research.eureka_threshold;
    let tree = &play.sim.colony.tree;
    for (row, branch) in Branch::ALL.iter().enumerate() {
        let y = content.y + row as f32 * row_h;
        if row > 0 {
            divider(content.x, y, content.w - DETAIL - 20.0);
        }
        let color = branch_color(*branch);
        draw_rectangle(content.x, y + 8.0, 4.0, row_h - 16.0, color);
        label(
            data.label(branch.key()),
            content.x + 14.0,
            y + 8.0,
            TEXT_BODY,
            color,
        );
        let researched = fill_template(
            data.label("researched_count"),
            &[("count", &tree.researched_in(*branch).to_string())],
        );
        let unknown = fill_template(
            data.label("unknown_in_branch"),
            &[("count", &tree.unknown_in(*branch).to_string())],
        );
        label(
            &researched,
            content.x + 14.0,
            y + 34.0,
            TEXT_SMALL,
            TEXT_DIM,
        );
        label(&unknown, content.x + 14.0, y + 56.0, TEXT_SMALL, TEXT_FAINT);
        let insight = play.sim.colony.research.insight[branch.index()];
        label(
            data.label("insight"),
            content.x + 14.0,
            y + 80.0,
            TEXT_SMALL,
            TEXT_FAINT,
        );
        meter(
            Rect::new(content.x + 90.0, y + 87.0, SIDEBAR - 110.0, 6.0),
            insight / threshold,
            color,
        );
    }
}

fn draw_links(play: &PlayState, rects: &HashMap<String, Rect>, focus: &str) {
    let tree = &play.sim.colony.tree;
    let (Some(node), Some(target)) = (tree.node(focus), rects.get(focus)) else {
        return;
    };
    for prereq in &node.prereqs {
        if let Some(from) = rects.get(prereq) {
            let color = if tree.is_researched(prereq) {
                GOOD
            } else {
                WARN
            };
            draw_line(
                from.x + from.w,
                from.y + from.h * 0.5,
                target.x,
                target.y + target.h * 0.5,
                2.0,
                color,
            );
        }
    }
    for other in tree
        .nodes
        .iter()
        .filter(|n| n.prereqs.iter().any(|p| p == focus))
    {
        if let Some(to) = rects.get(&other.id) {
            draw_line(
                target.x + target.w,
                target.y + target.h * 0.5,
                to.x,
                to.y + to.h * 0.5,
                1.5,
                CYAN,
            );
        }
    }
}

fn draw_card(ui: &mut Ui, play: &PlayState, id: &str, rect: Rect, selected: bool) {
    let Some(tech) = game_data().tech(id) else {
        return;
    };
    let tree = &play.sim.colony.tree;
    let status = tree.status(id);
    let research = &play.sim.colony.research;
    let fill = if selected {
        Color::new(0.22, 0.18, 0.12, 1.0)
    } else if ui.hovered(rect) {
        PANEL_HOVER
    } else {
        PANEL_RAISED
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle(rect.x, rect.y, 4.0, rect.h, branch_color(tech.branch));
    let is_focus = research.focus.as_deref() == Some(id);
    let border = if is_focus {
        ACCENT
    } else if status == TechStatus::Available {
        LINE_STRONG
    } else {
        LINE
    };
    let thickness = if is_focus { 2.5 } else { 1.0 };
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, border);
    // Inspired technologies carry the colour of a species that inspired them.
    let inspiration = tech
        .inspired_by
        .iter()
        .find(|f| play.sim.colony.finds.get(f).is_some())
        .and_then(|f| game_data().find(f));
    let text_x = if let Some(find) = inspiration {
        draw_circle(rect.x + 16.0, rect.y + rect.h * 0.5, 5.0, find_color(find));
        rect.x + 28.0
    } else {
        rect.x + 12.0
    };
    label_fit(
        &tech.name,
        text_x,
        rect.y + 6.0,
        rect.x + rect.w - text_x - 22.0,
        TEXT_SMALL,
        status_color(status),
    );
    if status == TechStatus::Researched {
        icon_check(rect.x + rect.w - 18.0, rect.y + 9.0, 11.0, GOOD);
    }
    if status == TechStatus::Locked {
        let mid = rect.y + rect.h * 0.5;
        draw_line(rect.x + 8.0, mid, rect.x + rect.w - 8.0, mid, 1.5, BAD);
    }
    let progress = research.progress_of(id);
    if progress > 0.0 && status != TechStatus::Researched {
        let scale = play.sim.colony.difficulty_value(|d| d.research_cost);
        let fraction = (progress / tree.cost(id, scale)).min(1.0);
        draw_rectangle(
            rect.x + 4.0,
            rect.y + rect.h - 3.0,
            (rect.w - 4.0) * fraction,
            3.0,
            ACCENT,
        );
    }
    if ui.hit(rect) {
        ui.act(UiAction::FocusTech(id.to_owned()));
    }
}

/// The species the colony has found, in the order it found them.
fn draw_finds(play: &PlayState, content: Rect) {
    let data = game_data();
    let grid = grid_area(content);
    let area = Rect::new(
        content.x,
        grid.y + grid.h,
        content.w - DETAIL - 20.0,
        FINDS_HEIGHT,
    );
    divider(area.x, area.y, area.w);
    heading(data.label("finds_heading"), area.x, area.y + 10.0);
    let finds = &play.sim.colony.finds;
    let unfound = play
        .sim
        .world
        .species
        .iter()
        .filter(|id| finds.get(id).is_none())
        .count();
    let note = if finds.found.is_empty() {
        data.label("finds_none").to_owned()
    } else {
        fill_template(
            data.label("finds_unfound"),
            &[("count", &unfound.to_string())],
        )
    };
    let note_x =
        area.x + text_width(&data.label("finds_heading").to_uppercase(), TEXT_SMALL) + 20.0;
    label_fit(
        &note,
        note_x,
        area.y + 10.0,
        area.w - (note_x - area.x),
        TEXT_SMALL,
        TEXT_DIM,
    );
    let per_line = ((area.w + 8.0) / (CHIP_WIDTH + 8.0)).floor().max(1.0) as usize;
    let capacity = per_line * 2;
    let shown = if finds.found.len() > capacity {
        capacity - 1
    } else {
        capacity
    };
    for (index, found) in finds.found.iter().take(shown).enumerate() {
        let Some(find) = data.find(&found.id) else {
            continue;
        };
        let x = area.x + (index % per_line) as f32 * (CHIP_WIDTH + 8.0);
        let y = area.y + 40.0 + (index / per_line) as f32 * (CHIP_HEIGHT + 6.0);
        draw_rectangle(x, y, CHIP_WIDTH, CHIP_HEIGHT, PANEL_RAISED);
        draw_rectangle_lines(x, y, CHIP_WIDTH, CHIP_HEIGHT, 1.0, LINE);
        draw_rectangle(x + 8.0, y + 10.0, 16.0, 16.0, find_color(find));
        label_fit(
            &find.name,
            x + 32.0,
            y + 8.0,
            CHIP_WIDTH - 90.0,
            TEXT_SMALL,
            TEXT,
        );
        let amount = format!("{:.0}", found.gathered);
        let amount_x = x + CHIP_WIDTH - 10.0 - text_width(&amount, TEXT_SMALL);
        label(&amount, amount_x, y + 8.0, TEXT_SMALL, TEXT_DIM);
    }
    if finds.found.len() > shown {
        let index = shown;
        let x = area.x + (index % per_line) as f32 * (CHIP_WIDTH + 8.0);
        let y = area.y + 40.0 + (index / per_line) as f32 * (CHIP_HEIGHT + 6.0);
        let more = fill_template(
            data.label("finds_more"),
            &[("count", &(finds.found.len() - shown).to_string())],
        );
        label(&more, x + 8.0, y + 8.0, TEXT_SMALL, TEXT_DIM);
    }
}

pub fn draw(ui: &mut Ui, play: &PlayState, content: Rect) {
    let data = game_data();
    let rects = layout(play, content);
    draw_sidebar(play, content);
    let selected = play
        .tech_focus
        .clone()
        .or_else(|| play.sim.colony.research.focus.clone())
        .filter(|id| rects.contains_key(id));
    if let Some(focus) = &selected {
        draw_links(play, &rects, focus);
    }
    for tech in &data.techs {
        if let Some(rect) = rects.get(&tech.id) {
            draw_card(
                ui,
                play,
                &tech.id,
                *rect,
                selected.as_deref() == Some(tech.id.as_str()),
            );
        }
    }
    draw_finds(play, content);
    let detail = Rect::new(content.x + content.w - DETAIL, content.y, DETAIL, content.h);
    match &selected {
        Some(id) => detail::draw_detail(ui, play, id, detail),
        None => {
            panel_solid(detail);
            paragraph(
                data.label("research_help"),
                detail.x + 16.0,
                detail.y + 16.0,
                DETAIL - 32.0,
                TEXT_BODY,
                TEXT_DIM,
            );
        }
    }
}
