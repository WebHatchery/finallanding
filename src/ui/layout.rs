//! Main layout manager for UI regions

use macroquad::prelude::*;

/// Screen layout regions
pub struct Layout {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub top_bar_height: f32,
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_toolbar_height: f32,
    pub screen_margin: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self::responsive(screen_width(), screen_height())
    }
}

impl Layout {
    pub fn responsive(viewport_width: f32, viewport_height: f32) -> Self {
        let compact = viewport_width < 1_100.0;
        let phone = viewport_width < 760.0;
        Self {
            viewport_width,
            viewport_height,
            top_bar_height: if phone { 58.0 } else { 66.0 },
            left_panel_width: if phone {
                154.0
            } else if compact {
                224.0
            } else {
                300.0
            },
            right_panel_width: if phone {
                156.0
            } else if compact {
                226.0
            } else {
                302.0
            },
            bottom_toolbar_height: if phone { 104.0 } else { 86.0 },
            screen_margin: if phone { 7.0 } else { 12.0 },
        }
    }

    pub fn refresh(&mut self) {
        *self = Self::responsive(screen_width(), screen_height());
    }

    /// Get the game area rectangle (where grid is drawn)
    pub fn game_area(&self) -> Rect {
        self.game_area_with_context(false)
    }

    pub fn game_area_with_context(&self, context_open: bool) -> Rect {
        self.game_area_with_height(context_open, 126.0)
    }

    pub fn game_area_with_height(&self, context_open: bool, context_height: f32) -> Rect {
        let reserved_bottom = self.bottom_toolbar_height
            + if context_open {
                context_height + 22.0
            } else {
                0.0
            };
        Rect {
            x: self.screen_margin,
            y: self.top_bar_height + self.screen_margin,
            w: (self.viewport_width - self.screen_margin * 2.0).max(1.0),
            h: (self.viewport_height
                - self.top_bar_height
                - reserved_bottom
                - self.screen_margin * 2.0)
                .max(1.0),
        }
    }

    pub fn inspector_panel(&self) -> Rect {
        let width = if self.viewport_width < 760.0 {
            260.0
        } else {
            292.0
        };
        let height: f32 = if self.viewport_height < 560.0 {
            212.0
        } else {
            254.0
        };
        Rect::new(
            (self.viewport_width - width - self.screen_margin).max(self.screen_margin),
            self.top_bar_height + self.screen_margin + 58.0,
            width.min(self.viewport_width - self.screen_margin * 2.0),
            height.min(
                self.viewport_height - self.top_bar_height - self.bottom_toolbar_height - 72.0,
            ),
        )
    }

    /// Get the top bar rectangle
    pub fn top_bar(&self) -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: self.viewport_width,
            h: self.top_bar_height,
        }
    }

    pub fn left_panel(&self) -> Rect {
        Rect {
            x: self.screen_margin,
            y: self.top_bar_height + self.screen_margin,
            w: self.left_panel_width - self.screen_margin * 2.0,
            h: self.viewport_height
                - self.top_bar_height
                - self.bottom_toolbar_height
                - self.screen_margin * 2.0,
        }
    }

    pub fn right_panel(&self) -> Rect {
        Rect {
            x: self.viewport_width - self.right_panel_width + self.screen_margin,
            y: self.top_bar_height + self.screen_margin,
            w: self.right_panel_width - self.screen_margin * 2.0,
            // The toolbar occupies the center; the right rail can use the
            // full screen height without crossing it.
            h: self.viewport_height - self.top_bar_height - self.screen_margin * 2.0,
        }
    }

    pub fn bottom_toolbar(&self) -> Rect {
        let width = (self.viewport_width - self.screen_margin * 2.0).clamp(308.0, 760.0);
        Rect {
            x: (self.viewport_width - width) * 0.5,
            y: self.viewport_height - self.bottom_toolbar_height + 10.0,
            w: width,
            h: self.bottom_toolbar_height - 20.0,
        }
    }
}
