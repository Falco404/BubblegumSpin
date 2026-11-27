//! The window: a thin `macroquad` shell around the software framebuffer.
//!
//! All drawing happens into `Fb` inside `gfx::scenes`; this file only reads the
//! keyboard, advances the simulation at a fixed 60 Hz, and uploads the frame as
//! a texture scaled by an integer factor so the 128x128 picture stays crisp.

use bubblegum_spin::gfx::{fb::Fb, scenes::draw_game};
use bubblegum_spin::save::Save;
use bubblegum_spin::sim::{Game, Input};
use macroquad::prelude::*;

/// PICO-8's `_update60`: one simulation step per 1/60 s.
const STEP: f32 = 1.0 / 60.0;
/// Default window size: 4x the 128x128 screen.
const DEFAULT_SCALE: f32 = 4.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Bubblegum Spin".to_owned(),
        window_width: (128.0 * DEFAULT_SCALE) as i32,
        window_height: (128.0 * DEFAULT_SCALE) as i32,
        window_resizable: true,
        high_dpi: false,
        ..Default::default()
    }
}

fn read_input() -> Input {
    Input {
        left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
        right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
        up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
        // The cart's X button; Z is its O button, which M5 will use.
        fire: is_key_down(KeyCode::X) || is_key_down(KeyCode::K),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // cart `_init()`: the hiscore lives beside the executable.
    let save_path = Save::default_path();
    let save = save_path.as_deref().map(Save::load).unwrap_or_default();
    // PICO-8 seeds its own generator; the port takes a fresh seed per run so the
    // board is not identical every game. `Rng::new` maps 0 to a constant.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() ^ d.as_secs() as u32)
        .unwrap_or(1);
    let mut game = Game::new_with_save(seed, save, save_path);
    let mut fb = Fb::new();
    draw_game(&mut fb, &game);

    let texture = Texture2D::from_rgba8(128, 128, &fb.to_rgba8());
    texture.set_filter(FilterMode::Nearest);

    // macroquad's texture update takes an `Image`; keep one and refill its bytes.
    let mut image = Image {
        width: 128,
        height: 128,
        bytes: Vec::new(),
    };

    let mut accumulator = 0.0f32;
    loop {
        // Fixed step, capped so a stall does not replay hundreds of frames.
        accumulator += get_frame_time().min(0.25);
        let input = read_input();
        while accumulator >= STEP {
            game.step(&input);
            accumulator -= STEP;
        }

        draw_game(&mut fb, &game);
        image.bytes = fb.to_rgba8();
        texture.update(&image);

        clear_background(BLACK);
        let scale = (screen_width() / 128.0)
            .min(screen_height() / 128.0)
            .floor()
            .max(1.0);
        let (w, h) = (128.0 * scale, 128.0 * scale);
        draw_texture_ex(
            &texture,
            ((screen_width() - w) / 2.0).floor(),
            ((screen_height() - h) / 2.0).floor(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
        next_frame().await;
    }
}
