//! The window: a thin `macroquad` shell around the software framebuffer.
//!
//! All drawing happens into `Fb` inside `gfx::scenes`; this file only reads the
//! keyboard, advances the simulation at a fixed 60 Hz, and uploads the frame as
//! a texture scaled by an integer factor so the 128x128 picture stays crisp.

use std::collections::HashMap;

use bubblegum_spin::audio::{music_enabled, render_music, render_sfx, to_wav, Sfx, SAMPLE_RATE};
use bubblegum_spin::fixed::to_f64;
use bubblegum_spin::gfx::{fb::Fb, menus, scenes::fade_scr};
use bubblegum_spin::save::Save;
use bubblegum_spin::sim::scene::App;
use bubblegum_spin::sim::{Game, Input};
use bubblegum_spin::ui::{Layout, Strip};
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

/// The cart's buttons: the keyboard on a desktop, the on-screen pad on a phone, and
/// both wherever a window happens to have a strip (`BGS_TOUCH=1` gives a desktop one).
fn read_input(layout: &Layout) -> Input {
    let pad = layout.input(&touch_points(layout.strip.is_some()));
    Input {
        left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
        right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
        up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
        down: is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
        // The cart's X button fires; Z is its O button (hold to peek at the HUD).
        fire: is_key_down(KeyCode::X) || is_key_down(KeyCode::K),
        peek: is_key_down(KeyCode::Z) || is_key_down(KeyCode::J),
    }
    .merge(pad)
}

/// Every finger and button currently down, in screen coordinates.
///
/// On the phone that is `touches()`; with `BGS_TOUCH=1` on a desktop, the mouse drives
/// the same strip so the layout can be checked without an APK.
fn touch_points(touch_mode: bool) -> Vec<(f32, f32)> {
    let mut points: Vec<(f32, f32)> = touches()
        .iter()
        .filter(|t| !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled))
        .map(|t| (t.position.x, t.position.y))
        .collect();
    if touch_mode && !cfg!(target_os = "android") && is_mouse_button_down(MouseButton::Left) {
        let (x, y) = mouse_position();
        points.push((x, y));
    }
    points
}

/// Cartesian input from a 4-way pad: which way a press leans, as a unit vector.
fn pad_lean(x: f32, y: f32, cx: f32, cy: f32) -> (f32, f32) {
    let (dx, dy) = (x - cx, y - cy);
    if dx.abs() > dy.abs() {
        (dx.signum(), 0.0)
    } else {
        (0.0, dy.signum())
    }
}

/// The on-screen controls: a pad, `O` and `X`, translucent so the board stays visible.
fn draw_controls(strip: &Strip) {
    let dim = Color::new(1.0, 1.0, 1.0, 0.18);
    let ink = Color::new(1.0, 1.0, 1.0, 0.65);
    let size = strip.side();
    let font = size * 0.4;

    let (px, py, side) = strip.pad();
    draw_rectangle(px, py, side, side, dim);
    let (cx, cy) = (px + side / 2.0, py + side / 2.0);
    let (lx, ly) = pad_lean(cx - side, cy, cx, cy);
    let arm = side * 0.28;
    let thick = side * 0.12;
    // The four arms, so the pad reads as a d-pad rather than a square.
    let arms = [
        (lx, ly),
        (-lx, -ly),
        (
            pad_lean(cx, cy - side, cx, cy).0,
            pad_lean(cx, cy - side, cx, cy).1,
        ),
        (
            pad_lean(cx, cy + side, cx, cy).0,
            pad_lean(cx, cy + side, cx, cy).1,
        ),
    ];
    for (ax, ay) in arms {
        if ax == 0.0 && ay == 0.0 {
            continue;
        }
        let (w, h) = if ax != 0.0 {
            (arm, thick)
        } else {
            (thick, arm)
        };
        draw_rectangle(
            cx + ax * arm / 2.0 - w / 2.0,
            cy + ay * arm / 2.0 - h / 2.0,
            w,
            h,
            ink,
        );
    }

    let (peek, fire) = strip.action_buttons();
    for (x, y, label) in [(peek.0, peek.1, "O"), (fire.0, fire.1, "X")] {
        draw_rectangle(x, y, size, size, dim);
        let metrics = measure_text(label, None, font as u16, 1.0);
        draw_text(
            label,
            x + (size - metrics.width) / 2.0,
            y + (size + metrics.height) / 2.0,
            font,
            ink,
        );
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // cart `_init()`: the hiscore lives beside the executable, or under `$HOME` on a
    // phone. Say where it went once — it is the only way to check this on a device
    // without a debugger, and it is one line in a console nobody reads otherwise.
    let save_path = Save::default_path();
    let save = save_path.as_deref().map(Save::load).unwrap_or_default();
    match save_path.as_deref() {
        Some(path) => eprintln!("save: {} (hiscore {})", path.display(), save.hiscore),
        None => eprintln!("save: none — the hiscore will not persist"),
    }
    // PICO-8 seeds its own generator; the port takes a fresh seed per run so the
    // board is not identical every game. `Rng::new` maps 0 to a constant.
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() ^ d.as_secs() as u32)
        .unwrap_or(1);
    let mut app = App::new_with_save(seed, save, save_path);
    let mut audio = Audio::new();
    if !audio.music_on {
        // One line, once, so it is obvious why there is no theme.
        eprintln!("audio: music off (set BGS_MUSIC=1 to hear it); effects on");
    }
    let mut fb = Fb::new();
    // cart `_draw`: the fade is applied to the palette, then the scene draws.
    fade_scr(&mut fb, to_f64(app.game.fade.value));
    menus::draw(&mut fb, &app);

    let texture = Texture2D::from_rgba8(128, 128, &fb.to_rgba8());
    texture.set_filter(FilterMode::Nearest);

    // macroquad's texture update takes an `Image`; keep one and refill its bytes.
    let mut image = Image {
        width: 128,
        height: 128,
        bytes: Vec::new(),
    };

    // The phone has no keyboard, so its buttons are drawn under the picture;
    // `BGS_TOUCH=1` gives a desktop window the same strip to check the layout.
    let touch = cfg!(target_os = "android") || std::env::var_os("BGS_TOUCH").is_some();

    let mut accumulator = 0.0f32;
    loop {
        // Recomputed every frame: the window can be resized, and on a phone the
        // notches and the rotation can change `screen_width()` at runtime.
        let layout = Layout::for_screen(screen_width(), screen_height(), touch);
        // Fixed step, capped so a stall does not replay hundreds of frames.
        accumulator += get_frame_time().min(0.25);
        let input = read_input(&layout);
        while accumulator >= STEP {
            app.step(&input);
            accumulator -= STEP;
        }
        audio.play(&mut app.game).await;

        fade_scr(&mut fb, to_f64(app.game.fade.value));
        menus::draw(&mut fb, &app);
        image.bytes = fb.to_rgba8();
        texture.update(&image);

        clear_background(BLACK);
        let picture = layout.picture;
        draw_texture_ex(
            &texture,
            picture.x,
            picture.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(picture.side, picture.side)),
                ..Default::default()
            },
        );
        if let Some(strip) = layout.strip {
            draw_controls(&strip);
        }
        next_frame().await;
    }
}
