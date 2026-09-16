use finallanding::game::Game;
use finallanding::ui;
use macroquad::prelude::*;
use macroquad_toolkit::capture;

fn window_conf() -> Conf {
    // Hand-built Conf means no automatic arming: without this the capture run
    // puts a full game window on the desktop for its whole duration.
    capture::headless::arm("TFL");

    // Built by hand (not capture::capture_window_conf) to keep TFL_FULLSCREEN
    // support; high_dpi stays at its false default, so captures are already
    // pixel-aligned.
    Conf {
        window_title: "The Final Landing".to_owned(),
        window_width: capture::env_i32("TFL_WINDOW_WIDTH", 1280),
        window_height: capture::env_i32("TFL_WINDOW_HEIGHT", 720),
        window_resizable: true,
        fullscreen: capture::env_bool("TFL_FULLSCREEN", false),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    ui::font::init_ui_font();
    if export_playthrough_report_if_requested() {
        return;
    }

    let mut game: Game = Game::new().await;

    // Screenshot harness: when TFL_CAPTURE_PATH is set, render deterministic
    // frames and write PNGs from one process/window.
    if let Some(configs) = capture::CaptureConfig::all_from_env("TFL") {
        for mut config in configs {
            config.frames = capture::env_u32("TFL_CAPTURE_FRAMES", 8).max(1);
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |_dt| {
                clear_background(BLACK);
                game.update();
                game.draw();
                if config.scene == "smoke_toolkit_tooltips" {
                    for bounds in [Rect::new(50.0, 120.0, 180.0, 90.0), Rect::new(650.0, 120.0, 320.0, 90.0)] {
                        ui::tooltip::draw_tooltip_at(
                            vec2(bounds.right(), bounds.bottom()), bounds,
                            "WWW — A very long colony infrastructure title",
                            "Wide glyphs and long reports must fit inside the available tooltip width.",
                        );
                    }
                }
            })
            .await;
        }
        return;
    }

    loop {
        clear_background(BLACK);
        game.update();
        game.draw();
        next_frame().await
    }
}

fn export_playthrough_report_if_requested() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(path) = capture::env_string("TFL_PLAYTHROUGH_REPORT_PATH") else {
            return false;
        };

        let reports = finallanding::systems::playtest_system::PlaytestSystem::capture_report_set();
        let markdown =
            finallanding::systems::playtest_system::PlaytestSystem::playthrough_report_markdown(
                &reports,
            );
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent).expect("failed to create playthrough report directory");
        }
        std::fs::write(&path, markdown).expect("failed to write playthrough report");
        true
    }

    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}
