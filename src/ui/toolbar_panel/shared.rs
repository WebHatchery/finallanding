//! shared domain.

use crate::systems::assignment_system::AssignmentPressure;
use crate::systems::relationship_directive_system::PairDirective;
use crate::ui::style;
use macroquad::prelude::{Color, Rect};

pub fn toolbar_tooltip_bounds(context: Rect) -> Rect {
    Rect::new(0.0, 0.0, context.w, (context.y - 8.0).max(44.0))
}

pub fn assignment_pressure_color(pressure: AssignmentPressure) -> Color {
    match pressure {
        AssignmentPressure::Supported => style::BAR_GREEN,
        AssignmentPressure::Neutral => style::HEADING_BLUE,
        AssignmentPressure::Tense => style::ALERT_RED,
    }
}

pub fn directive_color(directive: PairDirective) -> Color {
    match directive {
        PairDirective::Pair => style::BAR_GREEN,
        PairDirective::Separate => style::ALERT_RED,
    }
}
