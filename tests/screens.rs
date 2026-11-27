//! Screen tests: whole frames rendered headless and compared with committed
//! golden PNGs, pixel for pixel.
//!
//! Run with `UPDATE_GOLDENS=1 cargo test --test screens` after an intentional
//! visual change; the diff report names the first differing pixel.

use std::path::PathBuf;

use bubblegum_spin::fixed::from_f64;
use bubblegum_spin::gfx::fb::Fb;
use bubblegum_spin::gfx::scenes::draw_game;
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
