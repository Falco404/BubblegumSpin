//! Deterministic replay: a scripted 900-frame session hashed into one `u64`.
//!
//! This is the fidelity net for the whole project: any refactor that changes the
//! physics, the placement rules or the turn bookkeeping changes the hash, even
//! when the unit tests still pass. The golden value is recorded from a run of the
//! code that was checked by hand, never from a guess.

use bubblegum_spin::sim::{Game, Input};

/// The hash of the session below, recorded from the first green run of the code
/// that the unit tests in `src/sim/bubble.rs` verify.
const GOLDEN: u64 = 8_430_229_834_542_539_825;

/// A fixed input timeline. Six shots: left, right, upright, straight up again,
/// left again, and one with a held fire button (which must not fire twice).
fn input_at(frame: u32) -> Input {
    let fire = |extra: Input| Input {
        fire: true,
        ..extra
    };
    match frame {
        0..=19 => Input {
            left: true,
            ..Default::default()
        },
        20 => fire(Input::default()),
        21..=139 => Input::default(),
        140..=179 => Input {
            right: true,
            ..Default::default()
        },
        180 => fire(Input::default()),
        181..=299 => Input::default(),
        300..=319 => Input {
            up: true,
            ..Default::default()
        },
        320 => fire(Input::default()),
        321..=429 => Input::default(),
        430 => fire(Input::default()),
        431..=539 => Input::default(),
        540..=559 => Input {
            left: true,
            ..Default::default()
        },
        560 => fire(Input::default()),
        561..=579 => Input {
            fire: true,
            ..Default::default()
        },
        _ => Input::default(),
    }
}

/// FNV-1a over every number that decides where a gumball ends up.
fn mix(h: &mut u64, v: i64) {
    for b in v.to_le_bytes() {
        *h ^= b as u64;
        *h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn state_hash(g: &Game) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (&k, &v) in g.board.0.iter() {
        mix(&mut h, k as i64);
        mix(&mut h, v as i64);
    }
    for b in g.bubs.iter().chain(g.f_bubs.iter()) {
        for v in [
            b.x as i64,
            b.y as i64,
            b.dx as i64,
            b.dy as i64,
            b.col as i64,
            b.fired as i64,
            b.falling as i64,
        ] {
            mix(&mut h, v);
        }
    }
    for v in [
        g.spin.var as i64,
        g.spin.torque as i64,
        g.gun.a as i64,
        g.shot_cnt as i64,
        g.shot_count as i64,
        g.board_shots as i64,
        g.bounces as i64,
        g.hud_t as i64,
        g.nxt_col as i64,
        g.nxt_nxt_col as i64,
        g.player_turn as i64,
        g.in_flight as i64,
        g.in_play as i64,
        g.reloading as i64,
        g.popping as i64,
    ] {
        mix(&mut h, v);
    }
    for &c in &g.col_pool {
        mix(&mut h, c as i64);
    }
    h
}

fn play_900_frames(seed: u32) -> Game {
    let mut g = Game::new(seed);
    for frame in 0..900 {
        g.step(&input_at(frame));
    }
    g
}

#[test]
fn the_scripted_session_reproduces() {
    let g = play_900_frames(7);
    assert_eq!(state_hash(&g), GOLDEN, "the replay hash moved");
}

#[test]
fn the_scripted_session_actually_plays() {
    // Guard against a replay that passes because nothing ever happened. These
    // numbers come from the frame-by-frame trace of the run that produced
    // GOLDEN: six shots, two of them costing a try, three runs cleared.
    let g = Game::new(7);
    let start = g.board.0.len();
    let end = play_900_frames(7);
    assert_eq!(start, 61);
    assert_eq!(end.board.0.len(), 28, "three runs were cleared");
    assert_eq!(end.shot_cnt, 2, "two shots cleared nothing");
    assert_eq!(end.shot_count, 1, "the turn never changed hands");
    assert!(end.player_turn);
    assert!(end.bubs.iter().all(|b| !b.fired), "no shot is in the air");
    assert!(!end.popping, "the last pop has finished");
    assert!(end.board.0.values().all(|&v| v < 10));
}
