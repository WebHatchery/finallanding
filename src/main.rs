use finallanding::game::Game;
use macroquad::prelude::*;
use macroquad_toolkit::capture;

fn window_conf() -> Conf {
    // Hand-built Conf: arm the hidden capture window ourselves.
    capture::headless::arm("TFL");
    Conf {
        window_title: "The Final Landing".to_owned(),
        window_width: capture::env_i32("TFL_WINDOW_WIDTH", 1920),
        window_height: capture::env_i32("TFL_WINDOW_HEIGHT", 1080),
        window_resizable: true,
        fullscreen: capture::env_bool("TFL_FULLSCREEN", false),
        ..Default::default()
    }
}

/// Headless campaign matrix for pacing review (native only).
fn write_campaign_report() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(path) = capture::env_string("TFL_CAMPAIGN_REPORT_PATH") else {
            return false;
        };
        let days = capture::env_u32("TFL_CAMPAIGN_MAX_DAYS", 420);
        let runs = finallanding::autoplay::report::standard_matrix(days);
        let markdown = finallanding::autoplay::report::markdown(&runs);
        if let Err(error) = std::fs::write(&path, markdown) {
            eprintln!("could not write campaign report {path}: {error}");
        }
        true
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    if let Err(error) = macroquad_toolkit::ui::ensure_default_ui_font() {
        eprintln!("UI font unavailable: {error}");
    }
    if write_campaign_report() {
        return;
    }
    let mut game = Game::new();
    if let Some(configs) = capture::CaptureConfig::all_from_env("TFL") {
        for mut config in configs {
            config.frames = capture::env_u32("TFL_CAPTURE_FRAMES", 12).max(1);
            finallanding::game::begin_capture_scene(&mut game, &config.scene);
            next_frame().await;
            capture::run_capture_once(&config, |dt| game.frame(dt, false)).await;
        }
        return;
    }
    loop {
        game.frame(get_frame_time(), true);
        if game.quit_requested {
            break;
        }
        next_frame().await;
    }
}
