//! isometric domain.

use crate::data::types::Position;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct IsoView {
    pub origin: Vec2,
    pub tile_w: f32,
    pub tile_h: f32,
}

impl IsoView {
    pub fn for_area(area: Rect, grid_width: u32, grid_height: u32) -> Self {
        Self::for_area_with_zoom(area, grid_width, grid_height, 1.0)
    }

    pub fn for_area_with_zoom(area: Rect, grid_width: u32, grid_height: u32, zoom: f32) -> Self {
        Self::for_area_with_zoom_and_offset(area, grid_width, grid_height, zoom, Vec2::ZERO)
    }

    pub fn for_area_with_zoom_and_offset(
        area: Rect,
        grid_width: u32,
        grid_height: u32,
        zoom: f32,
        offset: Vec2,
    ) -> Self {
        let map_span = (grid_width + grid_height) as f32;
        // Fit the whole diamond inside the playable region at compact sizes so
        // every cell remains reachable by touch. Desktop layouts still get
        // generous tiles, while phone layouts trade tile size for coverage.
        let width_limited = area.w * 1.86 / map_span;
        let height_limited = area.h * 3.5 / map_span;
        let tile_w = (width_limited.min(height_limited) * zoom.clamp(0.8, 1.25)).clamp(8.0, 52.0);
        let tile_h = tile_w * 0.5;
        let map_h = (grid_width + grid_height) as f32 * tile_h * 0.5;
        Self {
            origin: vec2(
                area.x + area.w * 0.5,
                area.y + (area.h - map_h) * 0.28 + 18.0,
            ) + offset,
            tile_w,
            tile_h,
        }
    }

    pub fn map_bounds(self, grid_width: u32, grid_height: u32) -> Rect {
        let horizontal_extent = grid_width.max(grid_height) as f32 * self.tile_w * 0.5;
        let vertical_extent = (grid_width + grid_height) as f32 * self.tile_h * 0.5;
        Rect::new(
            self.origin.x - horizontal_extent,
            self.origin.y,
            horizontal_extent * 2.0,
            vertical_extent,
        )
    }

    pub fn pan_limits(area: Rect, grid_width: u32, grid_height: u32, zoom: f32) -> (Vec2, Vec2) {
        let base = Self::for_area_with_zoom(area, grid_width, grid_height, zoom);
        let bounds = base.map_bounds(grid_width, grid_height);
        let margin = 18.0;
        (
            vec2(
                bounded_pan_axis(area.x, area.right(), bounds.x, bounds.right(), margin).0,
                bounded_pan_axis(area.y, area.bottom(), bounds.y, bounds.bottom(), margin).0,
            ),
            vec2(
                bounded_pan_axis(area.x, area.right(), bounds.x, bounds.right(), margin).1,
                bounded_pan_axis(area.y, area.bottom(), bounds.y, bounds.bottom(), margin).1,
            ),
        )
    }

    pub fn clamp_pan(
        area: Rect,
        grid_width: u32,
        grid_height: u32,
        zoom: f32,
        offset: Vec2,
    ) -> Vec2 {
        let (min, max) = Self::pan_limits(area, grid_width, grid_height, zoom);
        vec2(offset.x.clamp(min.x, max.x), offset.y.clamp(min.y, max.y))
    }

    pub fn grid_to_screen(self, position: Position) -> Vec2 {
        let x = position.x as f32;
        let y = position.y as f32;
        vec2(
            self.origin.x + (x - y) * self.tile_w * 0.5,
            self.origin.y + (x + y) * self.tile_h * 0.5,
        )
    }

    pub fn screen_to_grid(self, point: Vec2) -> Position {
        let dx = (point.x - self.origin.x) / (self.tile_w * 0.5);
        let dy = (point.y - self.origin.y) / (self.tile_h * 0.5);
        Position::new(
            ((dy + dx) * 0.5).floor() as i32,
            ((dy - dx) * 0.5).floor() as i32,
        )
    }
}

fn bounded_pan_axis(
    area_start: f32,
    area_end: f32,
    map_start: f32,
    map_end: f32,
    margin: f32,
) -> (f32, f32) {
    let min = area_end - margin - map_end;
    let max = area_start + margin - map_start;
    if min <= max {
        (min, max)
    } else {
        (0.0, 0.0)
    }
}

pub fn draw_iso_diamond(center: Vec2, tile_w: f32, tile_h: f32, color: Color) {
    let [top, right, bottom, left] = iso_diamond_points(center, tile_w, tile_h);
    draw_triangle(top, right, bottom, color);
    draw_triangle(top, bottom, left, color);
}

pub fn draw_iso_diamond_lines(
    center: Vec2,
    tile_w: f32,
    tile_h: f32,
    thickness: f32,
    color: Color,
) {
    let [top, right, bottom, left] = iso_diamond_points(center, tile_w, tile_h);
    draw_line(top.x, top.y, right.x, right.y, thickness, color);
    draw_line(right.x, right.y, bottom.x, bottom.y, thickness, color);
    draw_line(bottom.x, bottom.y, left.x, left.y, thickness, color);
    draw_line(left.x, left.y, top.x, top.y, thickness, color);
}

pub fn iso_diamond_points(center: Vec2, tile_w: f32, tile_h: f32) -> [Vec2; 4] {
    [
        vec2(center.x, center.y),
        vec2(center.x + tile_w * 0.5, center.y + tile_h * 0.5),
        vec2(center.x, center.y + tile_h),
        vec2(center.x - tile_w * 0.5, center.y + tile_h * 0.5),
    ]
}

pub fn draw_iso_prism(
    center: Vec2,
    width: f32,
    height: f32,
    wall_height: f32,
    roof_color: Color,
    front_color: Color,
    side_color: Color,
) {
    let [_top, right, bottom, left] = iso_diamond_points(center, width, height);
    let right_drop = right + vec2(0.0, wall_height);
    let bottom_drop = bottom + vec2(0.0, wall_height);
    let left_drop = left + vec2(0.0, wall_height);

    draw_quad(right, right_drop, bottom_drop, bottom, front_color);
    draw_quad(bottom, bottom_drop, left_drop, left, side_color);
    draw_iso_diamond(center, width, height, roof_color);
    draw_iso_diamond_lines(
        center,
        width,
        height,
        1.0,
        Color::new(0.84, 0.84, 0.76, 0.55),
    );
    draw_line(
        right.x,
        right.y,
        right_drop.x,
        right_drop.y,
        1.0,
        Color::new(0.03, 0.035, 0.035, 0.8),
    );
    draw_line(
        left.x,
        left.y,
        left_drop.x,
        left_drop.y,
        1.0,
        Color::new(0.03, 0.035, 0.035, 0.8),
    );
    draw_line(
        bottom_drop.x,
        bottom_drop.y,
        left_drop.x,
        left_drop.y,
        1.0,
        Color::new(0.03, 0.035, 0.035, 0.7),
    );
}

fn draw_quad(a: Vec2, b: Vec2, c: Vec2, d: Vec2, color: Color) {
    draw_triangle(a, b, c, color);
    draw_triangle(a, c, d, color);
}
