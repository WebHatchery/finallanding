//! Interface geometry at the fixed 1920×1080 virtual resolution.

use finallanding::state::play::PlayState;
use finallanding::ui::actions::Selection;
use finallanding::ui::build_panel::drawer_rect;
use finallanding::ui::camera::{point_center, WorldCamera};
use finallanding::ui::input::pick;
use finallanding::ui::inspector::inspector_rect;
use finallanding::ui::overlays::overlay_rect;
use finallanding::ui::play_screen::is_over_fixed_ui;
use finallanding::ui::theme::*;
use finallanding::world::Tile;
use macroquad::prelude::vec2;

#[test]
fn camera_round_trips_and_zooms_about_the_pointer() {
    let mut camera = WorldCamera::looking_at(Tile::new(40, 30));
    let screen = vec2(700.0, 400.0);
    let world = camera.screen_to_world(screen);
    assert!((camera.world_to_screen(world) - screen).length() < 0.01);
    camera.zoom_at(1.5, screen);
    assert!(
        (camera.world_to_screen(world) - screen).length() < 0.01,
        "the point under the pointer stays put"
    );
    assert_eq!(
        camera.tile_at(camera.world_to_screen(world)),
        camera.tile_at(screen)
    );
}

#[test]
fn panels_fit_between_the_bars_at_1920_by_1080() {
    for rect in [drawer_rect(), inspector_rect(), overlay_rect()] {
        assert!(
            rect.x >= 0.0 && rect.x + rect.w <= VIRTUAL_WIDTH,
            "{rect:?} fits horizontally"
        );
        assert!(
            rect.y >= TOP_BAR_HEIGHT && rect.y + rect.h <= VIRTUAL_HEIGHT - TOOLBAR_HEIGHT,
            "{rect:?} sits between the bars"
        );
    }
}

#[test]
fn tapping_the_map_selects_what_is_under_the_pointer() {
    let sim = finallanding::sim::Sim::new(finallanding::colony::RunSetup {
        colony_name: "Test".into(),
        site: "verdant_basin".into(),
        difficulty: "standard".into(),
        seed: 12,
    });
    let mut play = PlayState::new(sim, false);
    let agent = play
        .sim
        .agents
        .iter()
        .find(|a| a.inside.is_none())
        .expect("someone outdoors");
    let (id, at) = (agent.id, point_center(agent.position));
    play.camera.target = at;
    let screen = play.camera.world_to_screen(at);
    assert_eq!(pick(&play, screen), Some(Selection::Agent(id)));
    assert!(
        is_over_fixed_ui(&play, vec2(960.0, VIRTUAL_HEIGHT - 10.0)),
        "the toolbar is not the map"
    );
    assert!(!is_over_fixed_ui(&play, screen), "the map is open to taps");
}
