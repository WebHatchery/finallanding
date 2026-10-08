//! Painted assets: every building sprite exists in the atlas, its frame is
//! tight to the painting, and it is drawn standing on its own footprint.

use finallanding::data::game_data;
use finallanding::ui::art::{building_atlas, sprite_frames, sprite_placement, BUILDING_SPRITES};
use macroquad::prelude::{Image, Rect};

const TILE: f32 = 32.0;

fn footprint(size: [i32; 2]) -> Rect {
    Rect::new(
        10.0 * TILE,
        10.0 * TILE,
        size[0] as f32 * TILE,
        size[1] as f32 * TILE,
    )
}

#[test]
fn every_sprite_building_has_a_painted_frame() {
    let (_, frames) = building_atlas();
    assert_eq!(frames.len(), BUILDING_SPRITES);
    for def in game_data().buildings.iter() {
        if let Some(sprite) = def.sprite {
            let frame = frames
                .get(sprite)
                .unwrap_or_else(|| panic!("{} uses missing sprite {sprite}", def.id));
            assert!(
                frame.w > 100.0 && frame.h > 100.0,
                "{} frame is empty",
                def.id
            );
        }
    }
}

#[test]
fn frames_are_tight_to_the_painting() {
    let (image, frames) = building_atlas();
    let alpha =
        |x: f32, y: f32| image.bytes[((y as usize) * image.width as usize + x as usize) * 4 + 3];
    for (index, frame) in frames.iter().enumerate() {
        let bottom = frame.y + frame.h - 1.0;
        let right = frame.x + frame.w - 1.0;
        let touches_bottom =
            (0..frame.w as usize).any(|dx| alpha(frame.x + dx as f32, bottom) >= 40);
        let touches_top = (0..frame.w as usize).any(|dx| alpha(frame.x + dx as f32, frame.y) >= 40);
        let touches_left =
            (0..frame.h as usize).any(|dy| alpha(frame.x, frame.y + dy as f32) >= 40);
        let touches_right = (0..frame.h as usize).any(|dy| alpha(right, frame.y + dy as f32) >= 40);
        assert!(
            touches_bottom && touches_top && touches_left && touches_right,
            "sprite {index} has a transparent margin inside its frame"
        );
    }
}

#[test]
fn painted_buildings_stand_on_their_footprint() {
    let (_, frames) = building_atlas();
    for def in game_data().buildings.iter() {
        let Some(sprite) = def.sprite else { continue };
        let rect = footprint(def.size);
        let dest = sprite_placement(frames[sprite], rect);
        let base = dest.y + dest.h;
        let ground = rect.y + rect.h;
        assert!(
            base <= ground && ground - base <= rect.h * 0.05,
            "{} floats {:.1}px above its footprint",
            def.id,
            ground - base
        );
        assert!(dest.y < rect.y, "{} should rise above its tiles", def.id);
    }
}

#[test]
fn painted_buildings_fill_their_footprint_width() {
    let (_, frames) = building_atlas();
    for def in game_data().buildings.iter() {
        let Some(sprite) = def.sprite else { continue };
        let rect = footprint(def.size);
        let dest = sprite_placement(frames[sprite], rect);
        let centre_offset = (dest.x + dest.w * 0.5) - (rect.x + rect.w * 0.5);
        assert!(centre_offset.abs() < 0.5, "{} is off-centre", def.id);
        assert!(
            dest.w >= rect.w && dest.w <= rect.w * 1.1,
            "{} is {:.0}px wide on a {:.0}px footprint",
            def.id,
            dest.w,
            rect.w
        );
    }
}

#[test]
fn transparent_margins_are_trimmed_from_any_atlas() {
    // Two 8×8 columns, each with one opaque block floating in empty space.
    let mut image =
        Image::gen_image_color(16, 8, macroquad::prelude::Color::new(0.0, 0.0, 0.0, 0.0));
    for (x, y) in [(2, 1), (3, 4), (11, 3), (12, 3)] {
        image.set_pixel(x, y, macroquad::prelude::WHITE);
    }
    let frames = sprite_frames(&image, 2);
    assert_eq!(frames[0], Rect::new(2.0, 1.0, 2.0, 4.0));
    assert_eq!(frames[1], Rect::new(11.0, 3.0, 2.0, 1.0));
}
