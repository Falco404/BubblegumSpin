//! The window: a thin `macroquad` shell around the software framebuffer.
//!
//! All drawing happens into `Fb`; this file only uploads it as a texture and
//! scales it up by an integer factor so the 128x128 picture stays crisp.

use bubblegum_spin::gfx::{fb::Fb, palette_card};
use macroquad::prelude::*;

/// PICO-8 resolution, 128x128, shown at 4x = 512x512 by default.
const SCALE: f32 = 4.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Bubblegum Spin".to_owned(),
        window_width: (128.0 * SCALE) as i32,
        window_height: (128.0 * SCALE) as i32,
        window_resizable: false,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut fb = Fb::new();
    palette_card(&mut fb);

    let texture = Texture2D::from_rgba8(128, 128, &fb.to_rgba8());
    texture.set_filter(FilterMode::Nearest);

    loop {
        clear_background(BLACK);
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        next_frame().await;
    }
}
