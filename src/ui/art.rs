//! Painted art embedded in the binary: the crash-site backdrop, survivor
//! portraits and the building atlas.

use macroquad::prelude::*;
use macroquad_toolkit::colors::shift_hue;

const PORTRAIT_COLUMNS: f32 = 3.0;
const PORTRAIT_ROWS: f32 = 2.0;
const BUILDING_SPRITES: f32 = 5.0;

pub struct Art {
    pub backdrop: Texture2D,
    pub portraits: Texture2D,
    pub buildings: Texture2D,
}

fn texture(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    texture.set_filter(FilterMode::Linear);
    texture
}

impl Art {
    pub fn load() -> Self {
        Self {
            backdrop: texture(include_bytes!("../../assets/art/crash_site_backdrop.png")),
            portraits: texture(include_bytes!("../../assets/art/survivor_portraits.png")),
            buildings: texture(include_bytes!("../../assets/art/building_atlas.png")),
        }
    }

    /// A survivor portrait, tinted by their hue so repeated faces differ.
    pub fn draw_portrait(&self, index: u8, hue: f32, rect: Rect) {
        let width = self.portraits.width() / PORTRAIT_COLUMNS;
        let height = self.portraits.height() / PORTRAIT_ROWS;
        let column = (index as f32 % PORTRAIT_COLUMNS).floor();
        let row = ((index as f32 / PORTRAIT_COLUMNS).floor() % PORTRAIT_ROWS).floor();
        let inset = width * 0.12;
        let source = Rect::new(
            column * width + inset,
            row * height + inset * 0.6,
            width - inset * 2.0,
            height - inset * 2.0,
        );
        let tint = shift_hue(Color::new(1.0, 0.97, 0.94, 1.0), hue);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.08, 0.09, 0.1, 1.0),
        );
        draw_texture_ex(
            &self.portraits,
            rect.x,
            rect.y,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                source: Some(source),
                ..Default::default()
            },
        );
    }

    /// A building sprite fitted into a destination rectangle, bottom-aligned.
    pub fn draw_building(&self, index: usize, rect: Rect, color: Color) {
        let width = self.buildings.width() / BUILDING_SPRITES;
        let height = self.buildings.height();
        let source = Rect::new(index as f32 * width, height * 0.08, width, height * 0.86);
        let aspect = source.h / source.w;
        let draw_w = rect.w * 1.15;
        let draw_h = draw_w * aspect;
        draw_texture_ex(
            &self.buildings,
            rect.x + (rect.w - draw_w) * 0.5,
            rect.y + rect.h - draw_h,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                source: Some(source),
                ..Default::default()
            },
        );
    }

    /// The backdrop scaled to cover a rectangle.
    pub fn draw_backdrop(&self, rect: Rect, tint: Color) {
        let scale = (rect.w / self.backdrop.width()).max(rect.h / self.backdrop.height());
        let size = vec2(
            self.backdrop.width() * scale,
            self.backdrop.height() * scale,
        );
        draw_texture_ex(
            &self.backdrop,
            rect.x + (rect.w - size.x) * 0.5,
            rect.y + (rect.h - size.y) * 0.5,
            tint,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    }
}
