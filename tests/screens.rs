//! Screen tests: whole frames rendered headless and compared with committed
//! golden PNGs, pixel for pixel.
//!
//! Run with `UPDATE_GOLDENS=1 cargo test --test screens` after an intentional
//! visual change; the diff report names the first differing pixel.

use std::path::PathBuf;

use bubblegum_spin::fixed::from_f64;
use bubblegum_spin::fixed::to_f64;
use bubblegum_spin::gfx::fb::Fb;
use bubblegum_spin::gfx::menus;
use bubblegum_spin::gfx::scenes::{draw_game, fade_scr};
use bubblegum_spin::sim::scene::{App, Scene, FINAL_NAV};
use bubblegum_spin::sim::{Game, Input};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Render one frame the way the `shot` binary does.
fn render(game: &Game) -> Fb {
    let mut fb = Fb::new();
    draw_game(&mut fb, game);
    fb
}

fn idle(frames: u32) -> Game {
    let mut g = Game::new(7);
    for _ in 0..frames {
        g.step(&Input::default());
    }
    g
}

fn compare(name: &str, fb: &Fb) {
    let path = golden_dir().join(format!("{name}.png"));
    let tmp = std::env::temp_dir().join(format!("bgs-screen-{name}.png"));
    fb.to_png(&tmp).expect("write the frame");

    if std::env::var("UPDATE_GOLDENS").is_ok() {
        std::fs::create_dir_all(golden_dir()).unwrap();
        std::fs::copy(&tmp, &path).unwrap();
        println!("updated {}", path.display());
        return;
    }

    let got = std::fs::read(&tmp).expect("read the rendered frame");
    let want = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => panic!(
            "missing golden {}: run UPDATE_GOLDENS=1 cargo test --test screens",
            path.display()
        ),
    };
    assert_eq!(
        got.len(),
        want.len(),
        "{name}: the frame size changed (PNG lengths differ)"
    );
    if got != want {
        // Report the first pixel whose colour differs, so a failure is actionable.
        let a = decode(&got);
        let b = decode(&want);
        let first = a
            .iter()
            .zip(b.iter())
            .position(|(x, y)| x != y)
            .unwrap_or(0);
        panic!(
            "{name}: pixels differ (first at index {first}, rendered {} vs golden {}). \
             Run UPDATE_GOLDENS=1 cargo test --test screens if the change was intended.",
            a[first], b[first]
        );
    }
}

/// Decode a PNG of this size into RGB triples (only used for the failure message).
fn decode(bytes: &[u8]) -> Vec<u8> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .expect("decode png");
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("png frame");
    assert_eq!((info.width, info.height), (128, 128));
    buf
}

#[test]
fn the_opening_frame_matches_its_golden() {
    // The cards have slid in and the round-start spin has stopped.
    let game = idle(200);
    compare("game-idle", &render(&game));
}

#[test]
fn a_bubble_in_flight_matches_its_golden() {
    let mut g = Game::new(7);
    for frame in 0..40 {
        let input = Input {
            fire: frame == 20,
            ..Default::default()
        };
        g.step(&input);
    }
    compare("game-shot", &render(&g));
}

#[test]
fn the_round_clear_banner_matches_its_golden() {
    let mut g = idle(200);
    g.col_pool.clear();
    g.step(&Input::default());
    g.step(&Input::default());
    assert!(g.round_clear);
    compare("round-clear", &render(&g));
}

#[test]
fn the_game_over_cascade_matches_its_golden() {
    let mut g = Game::new(7);
    for _ in 0..200 {
        g.step(&Input::default());
    }
    // Plant a gumball past the right wall: the cart ends the game.
    g.board.0.insert(270, 3);
    for _ in 0..40 {
        g.step(&Input::default());
    }
    assert!(g.over);
    compare("game-over", &render(&g));
}

#[test]
fn a_quarter_turn_frame_matches_its_golden() {
    let mut g = idle(200);
    g.spin.var = from_f64(0.25);
    compare("game-quarter-turn", &render(&g));
}

/// The scene machine on a given screen, driven the way a player gets there.
fn app_on(scene: &str) -> App {
    let mut app = App::new(7);
    let settle = |app: &mut App| {
        for _ in 0..200 {
            if app.game.fade.faded {
                return;
            }
            app.step(&Input::default());
        }
    };
    let tap = |app: &mut App, input: Input| {
        app.step(&input);
        app.step(&Input::default());
    };
    match scene {
        "how" => {
            settle(&mut app);
            tap(
                &mut app,
                Input {
                    down: true,
                    ..Default::default()
                },
            );
            tap(
                &mut app,
                Input {
                    fire: true,
                    ..Default::default()
                },
            );
        }
        "coin" => {
            settle(&mut app);
            tap(
                &mut app,
                Input {
                    fire: true,
                    ..Default::default()
                },
            );
        }
        "final" => {
            app.game.new_game();
            app.game.score = 700;
            app.game.fade.finish_in();
            app.scene = Scene::Final;
            app.nav = &FINAL_NAV;
            app.finalani = 1;
            app.gumballs = 7;
            app.blow_force = 7;
        }
        _ => settle(&mut app),
    }
    app
}

/// One frame of a menu scene, faded and drawn exactly as the window does it.
fn scene_frame(scene: &str, frames: u32) -> Fb {
    let mut app = app_on(scene);
    for _ in 0..frames {
        app.step(&Input::default());
    }
    let mut fb = Fb::new();
    fade_scr(&mut fb, to_f64(app.game.fade.value));
    menus::draw(&mut fb, &app);
    fb
}

#[test]
fn the_title_screen_matches_its_golden() {
    // Sixty frames: the cart fades the title in over fifty.
    compare("m8-main", &scene_frame("main", 60));
}

#[test]
fn the_how_to_page_matches_its_golden() {
    compare("m8-how", &scene_frame("how", 2));
}

#[test]
fn the_coin_drop_matches_its_golden() {
    compare("m8-coin", &scene_frame("coin", 100));
}

#[test]
fn the_score_screen_matches_its_golden() {
    // Long enough for the dispense, the chew and the bubble to finish.
    compare("m8-final", &scene_frame("final", 500));
}
