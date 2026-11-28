//! The window: a thin `macroquad` shell around the software framebuffer.
//!
//! All drawing happens into `Fb` inside `gfx::scenes`; this file only reads the
//! keyboard, advances the simulation at a fixed 60 Hz, and uploads the frame as
//! a texture scaled by an integer factor so the 128x128 picture stays crisp.

use std::collections::HashMap;

use bubblegum_spin::audio::{music_enabled, render_music, render_sfx, to_wav, Sfx, SAMPLE_RATE};
use bubblegum_spin::gfx::{fb::Fb, scenes::draw_game};
use bubblegum_spin::save::Save;
use bubblegum_spin::sim::{Game, Input};
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

/// The game's audio: each effect and track is rendered to a WAV and handed to
/// `macroquad` the first time it is asked for, so startup stays instant.
///
/// The simulation only queues ids (`Game::take_sfx`/`take_music`); this is the only
/// place that touches the audio device.
struct Audio {
    sfx: HashMap<u8, Sound>,
    music: HashMap<u8, Sound>,
    playing_music: Option<u8>,
    /// Background music ships off: the tracks still do not sound like the cart, and
    /// the effects are confirmed. `BGS_MUSIC=1` turns it back on.
    music_on: bool,
}

impl Audio {
    fn new() -> Self {
        Self {
            sfx: HashMap::new(),
            music: HashMap::new(),
            playing_music: None,
            music_on: music_enabled(),
        }
    }

    async fn sfx(&mut self, id: u8) -> Option<Sound> {
        if let Some(sound) = self.sfx.get(&id) {
            return Some(sound.clone());
        }
        let samples = render_sfx(&Sfx::from_id(id), SAMPLE_RATE);
        let wav = to_wav(&samples, SAMPLE_RATE);
        let sound = match load_sound_from_bytes(&wav).await {
            Ok(sound) => sound,
            Err(err) => {
                // Say so once per effect instead of failing silently: a missing
                // audio device and a backend without its feature both land here.
                eprintln!("audio: sfx {id} could not be loaded: {err}");
                return None;
            }
        };
        self.sfx.insert(id, sound.clone());
        Some(sound)
    }

    async fn music(&mut self, id: u8) -> Option<Sound> {
        if let Some(sound) = self.music.get(&id) {
            return Some(sound.clone());
        }
        let samples = render_music(id, SAMPLE_RATE);
        let wav = to_wav(&samples, SAMPLE_RATE);
        let sound = match load_sound_from_bytes(&wav).await {
            Ok(sound) => sound,
            Err(err) => {
                eprintln!("audio: music {id} could not be loaded: {err}");
                return None;
            }
        };
        self.music.insert(id, sound.clone());
        Some(sound)
    }

    /// Play everything the simulation asked for this frame.
    async fn play(&mut self, game: &mut Game) {
        for id in game.take_sfx() {
            if let Some(sound) = self.sfx(id).await {
                play_sound(
                    &sound,
                    PlaySoundParams {
                        looped: false,
                        volume: 0.6,
                    },
                );
            }
        }
        // The queue is drained either way, so the simulation behaves exactly as the
        // cart's does whether or not the tracks are audible.
        for id in game.take_music() {
            if !self.music_on || self.playing_music == Some(id) {
                continue;
            }
            if let Some(current) = self.playing_music {
                if let Some(old) = self.music.get(&current).cloned() {
                    stop_sound(&old);
                }
            }
            if let Some(sound) = self.music(id).await {
                play_sound(
                    &sound,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.5,
                    },
                );
                self.playing_music = Some(id);
            }
        }
    }
}

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
    let mut audio = Audio::new();
    if !audio.music_on {
        // One line, once, so it is obvious why there is no theme.
        eprintln!("audio: music off (set BGS_MUSIC=1 to hear it); effects on");
    }
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
        audio.play(&mut game).await;

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
