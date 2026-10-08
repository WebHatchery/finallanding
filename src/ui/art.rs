//! Painted art embedded in the binary: the crash-site backdrop, survivor
//! portraits and the building atlas.

use macroquad::prelude::*;
use macroquad_toolkit::colors::shift_hue;

const PORTRAIT_COLUMNS: f32 = 3.0;
const PORTRAIT_ROWS: f32 = 2.0;
/// Painted buildings in the atlas, one per equal-width column.
pub const BUILDING_SPRITES: usize = 5;
/// Pixels at least this opaque count as part of a painted sprite.
const OPAQUE_ALPHA: u8 = 40;
/// A painted building spans this much of its footprint's width.
const SPRITE_WIDTH: f32 = 1.06;
/// The painted base sits this far (as a share of footprint height) above the
/// footprint's lower edge, so the building stands inside its own tiles.
const SPRITE_BASE_INSET: f32 = 0.03;

pub const BUILDING_ATLAS_PNG: &[u8] = include_bytes!("../../assets/art/building_atlas.png");

pub struct Art {
    pub backdrop: Texture2D,
    pub portraits: Texture2D,
    pub buildings: Texture2D,
    /// The opaque bounds of each painted building in the atlas.
    pub building_frames: Vec<Rect>,
}

/// The tight opaque bounds of each equal-width column of a sprite atlas, so
/// transparent margins never lift a sprite off the ground it is drawn on.
pub fn sprite_frames(image: &Image, columns: usize) -> Vec<Rect> {
    let (width, height) = (image.width as usize, image.height as usize);
    let column_width = width / columns.max(1);
    let opaque = |x: usize, y: usize| image.bytes[(y * width + x) * 4 + 3] >= OPAQUE_ALPHA;
    (0..columns)
        .map(|column| {
            let (left, right) = (column * column_width, (column + 1) * column_width);
            let mut bounds: Option<(usize, usize, usize, usize)> = None;
            for y in 0..height {
                for x in left..right {
                    if opaque(x, y) {
                        let (x0, y0, x1, y1) = bounds.unwrap_or((x, y, x, y));
                        bounds = Some((x0.min(x), y0.min(y), x1.max(x), y1.max(y)));
                    }
                }
            }
            bounds.map_or(Rect::new(left as f32, 0.0, 0.0, 0.0), |(x0, y0, x1, y1)| {
                Rect::new(
                    x0 as f32,
                    y0 as f32,
                    (x1 - x0 + 1) as f32,
                    (y1 - y0 + 1) as f32,
                )
            })
        })
        .collect()
}

/// The decoded building atlas and its frames.
pub fn building_atlas() -> (Image, Vec<Rect>) {
    let image = Image::from_file_with_format(BUILDING_ATLAS_PNG, Some(ImageFormat::Png))
        .expect("the embedded building atlas is a valid PNG");
    let frames = sprite_frames(&image, BUILDING_SPRITES);
    (image, frames)
}

/// Where a painted building is drawn for a footprint: as wide as its tiles,
/// rising above them in three-quarter view, with its painted base resting on
/// the footprint's lower edge.
pub fn sprite_placement(frame: Rect, footprint: Rect) -> Rect {
    let width = footprint.w * SPRITE_WIDTH;
    let height = width * frame.h / frame.w.max(1.0);
    let bottom = footprint.y + footprint.h * (1.0 - SPRITE_BASE_INSET);
    Rect::new(
        footprint.x + (footprint.w - width) * 0.5,
        bottom - height,
        width,
        height,
    )
}

fn texture(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    texture.set_filter(FilterMode::Linear);
    texture
}

impl Art {
    pub fn load() -> Self {
        let (atlas, building_frames) = building_atlas();
        let buildings = Texture2D::from_image(&atlas);
        buildings.set_filter(FilterMode::Linear);
        Self {
            backdrop: texture(include_bytes!("../../assets/art/crash_site_backdrop.png")),
            portraits: texture(include_bytes!("../../assets/art/survivor_portraits.png")),
            buildings,
            building_frames,
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

    /// A painted building standing on its footprint, with a contact shadow
    /// where its base meets the ground.
    pub fn draw_building(&self, index: usize, footprint: Rect, color: Color) {
        let Some(frame) = self.building_frames.get(index).copied() else {
            return;
        };
        let dest = sprite_placement(frame, footprint);
        let base = dest.y + dest.h;
        draw_ellipse(
            footprint.x + footprint.w * 0.53,
            base - footprint.h * 0.12,
            footprint.w * 0.54,
            footprint.h * 0.2,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.32),
        );
        draw_texture_ex(
            &self.buildings,
            dest.x,
            dest.y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(frame),
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
