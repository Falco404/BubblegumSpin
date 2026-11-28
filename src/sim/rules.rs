//! The rules: score, rounds, game over and the hiscore.
//!
//! cart: the scoring half of `place_bubble`, `check_clear`, `new_round`,
//! `check_over`, `last_bubs_sort`, `hiscore_check`, `drop_bubs`, `score_text`.

use crate::fixed::{div, flr, from_f64, int, to_int, Fix};
use crate::grid::{b2px, B_R};
use crate::p8math::ceil;
use crate::sim::bubble::{Bubble, ARENA_BOTTOM, ARENA_LEFT, ARENA_RIGHT, ARENA_TOP, SHOT_MAX};
use crate::sim::spin::px2rpx;
use crate::sim::Game;

/// A round starts with two colours; the sixth is the last one.
pub const MIN_COL_CNT: i32 = 2;
pub const MAX_COL_CNT: i32 = 6;
/// cart: `t > round_clear_t + 210`.
pub const ROUND_CLEAR_FRAMES: i32 = 210;

/// One floating score number (cart `score_text`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreText {
    /// The cell whose position the number floats above.
    pub cell: u16,
    /// cart `col_convert(colour)`.
    pub colour: u8,
    /// The cart's `s.."0"`: the points times ten, already formatted.
    pub text: String,
    /// The cart's `♥`: 30 for a match, 0 for a hanging cell; drawn while > 60
    /// and deleted once it passes 120.
    pub age: i32,
}

/// cart `col_convert(_c)`: the palette colour score text is drawn in.
pub fn col_convert(c: u16) -> u8 {
    const COLS: [u8; 6] = [14, 8, 12, 11, 9, 2];
    COLS[(c.clamp(1, 6) - 1) as usize]
}

/// Is this (already rotated) cell centre outside the arena for a radius-4 ball?
///
/// cart `check_over`: `x,y = ceil(x),ceil(y)` first, then
/// `x-b_r < left or x+b_r-1 > right or y-b_r < top or y+b_r-1 > bottom`.
pub fn out_of_arena(x: Fix, y: Fix) -> bool {
    let (x, y) = (ceil(x), ceil(y));
    x - int(B_R) < int(ARENA_LEFT)
        || x + int(B_R - 1) > int(ARENA_RIGHT)
        || y - int(B_R) < int(ARENA_TOP)
        || y + int(B_R - 1) > int(ARENA_BOTTOM)
}

/// cart `ease_in_out(v, tv, dv)`: move `v` towards `tv`, with a fast approach
/// close to the target and a slow one far away. Returns the new value and rate.
pub fn ease_in_out(v: Fix, target: Fix, dv: Fix) -> (Fix, Fix) {
    let easein = int(4);
    let easeout = int(200);
    let mut dv = dv;
    if (target - v).abs() < int(4) {
        dv = div(target - v, easein);
    } else {
        dv += div(target - v, easeout);
    }
    let v = v + dv;
    if (target - v).abs() < from_f64(0.5) {
        (target, dv)
    } else {
        (v, dv)
    }
}

impl Game {
    /// cart `score_text`: queue one floating number.
    pub(crate) fn score_text(&mut self, cell: u16, colour: u8, points: i32, age: i32) {
        self.score_txt.push(ScoreText {
            cell,
            colour,
            text: format!("{points}0"),
            age,
        });
    }

    /// The cart ages the score text inside `draw_game` (`t.♥+=1`); here it is a
    /// step, so the simulation stays deterministic with no screen attached.
    pub fn tick_score_text(&mut self) {
        for entry in &mut self.score_txt {
            entry.age += 1;
        }
        self.score_txt.retain(|entry| entry.age <= 120);
    }

    /// cart `hiscore_check()`: remember and persist a better score.
    pub fn hiscore_check(&mut self) {
        if self.score as i64 > self.hiscore as i64 {
            self.new_hiscore = true;
            self.hiscore = self.score as u32;
            self.save.hiscore = self.hiscore;
            if let Some(path) = &self.save_path {
                let _ = self.save.store(path);
            }
        }
    }

    /// cart `check_clear()`: the board ran out of colours.
    pub fn check_clear(&mut self) {
        self.music(34); // music round clear
        if self.col_cnt < MAX_COL_CNT {
            self.col_cnt += 1;
        }
        self.round_clear = true;
        self.round_clear_t = self.t;
    }

    /// cart `new_round()`: a fresh level with one more colour, a new conveyor and
    /// the slow opening spin.
    pub fn new_round(&mut self) {
        self.music(16); // music game
        self.board.set_level(&mut self.rng, self.col_cnt);
        self.col_pool = self.board.pool();
        self.nxt_nxt_col = self.rnd_col();
        self.nxt_col = self.rnd_col();
        self.bubs.clear();
        self.f_bubs.clear();
        self.now_falling = 0;
        self.score_txt.clear();
        self.reset_bubble();
        self.last_shot_max = SHOT_MAX;
        self.shot_cnt = self.last_shot_max;
        self.shot_count = 1;

        // cart: from the six-colour round on, every round hands out one extra
        // shot for the rest of the game.
        if self.col_cnt == MAX_COL_CNT {
            self.bonus_shots += 1;
            self.shot_count += self.bonus_shots;
        }

        self.round_clear = false;
        self.round_clear_t = 0;
        self.spin.torque = from_f64(0.05);
    }

    /// cart `check_over()`: any settled gumball past the arena edge ends the game.
    ///
    /// The cartridge then hands the whole board to the falling list (sorted top to
    /// bottom), empties the board and checks the hiscore.
    pub fn check_over(&mut self) -> bool {
        let out = self.board.0.iter().any(|(&k, &v)| {
            if v >= 10 {
                return false;
            }
            let (gx, gy) = b2px(k);
            let (x, y) = px2rpx(int(gx), int(gy), self.spin.var);
            out_of_arena(x, y)
        });
        if !out {
            return false;
        }

        self.over = true;
        self.music(32); // music game over
        self.shake.amount = from_f64(2.0);
        self.drop_t = 0;
        self.f_bubs = std::mem::take(&mut self.bubs);
        self.last_bubs_sort();
        self.now_falling = self.f_bubs.len();
        self.board.0.clear();
        self.hiscore_check();
        true
    }

    /// cart `last_bubs_sort()`: turn every board cell into a falling bubble at its
    /// rotated position, then insertion-sort the list by ascending y.
    pub fn last_bubs_sort(&mut self) {
        let cells: Vec<(u16, u16)> = self.board.0.iter().map(|(&k, &v)| (k, v)).collect();
        for (i, v) in cells {
            let (gx, gy) = b2px(i);
            let (x, y) = px2rpx(int(gx), int(gy), self.spin.var);
            self.f_bubs.push(Bubble {
                x,
                y,
                dx: 0,
                dy: 0,
                col: v,
                fired: false,
                falling: false,
            });
        }

        // cart `isorty`: insertion sort, ascending y.
        for n in 1..self.f_bubs.len() {
            let mut i = n;
            while i > 0 && self.f_bubs[i].y < self.f_bubs[i - 1].y {
                self.f_bubs.swap(i, i - 1);
                i -= 1;
            }
        }
    }

    /// cart `drop_bubs()`: release the game-over cascade one bubble at a time.
    ///
    /// The divisor is `flr(rnd(#f_bubs/20))`, which is 0 for any list shorter than
    /// 20 — and PICO-8's `x % 0` is 0, so short lists release one bubble every
    /// frame. That is why the guard is explicit here: in Rust it would panic.
    pub fn drop_bubs(&mut self) {
        if self.now_falling == 0 {
            return;
        }
        self.drop_t += 1;
        let n = self.f_bubs.len() as i32;
        let divisor = to_int(flr(self.rng.rnd_fix(div(int(n), int(20)))));
        let release = divisor == 0 || self.drop_t % divisor == 0;
        if !release {
            return;
        }
        if self.now_falling <= self.f_bubs.len() {
            let i = self.now_falling - 1;
            self.f_bubs[i].falling = true;
            self.f_bubs[i].dy = 0;
            self.now_falling -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, ANCHOR};
    use crate::fixed::{from_f64, int, to_int};
    use crate::grid::CENTER;
    use crate::sim::bubble::SHOT_MAX;
    use crate::sim::gun::A_MIN;
    use crate::sim::{Game, Input};

    fn board_from(pairs: &[(u16, u16)]) -> Board {
        Board(pairs.iter().copied().collect())
    }

    fn still() -> Input {
        Input::default()
    }

    fn fired() -> Input {
        Input {
            fire: true,
            ..Default::default()
        }
    }

    #[test]
    fn a_match_scores_combo_times_the_pool() {
        // Three of colour 3 with a pool of two colours: 1*2 + 2*2 + 3*2 = 12.
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (240, 3), (241, 3)]);
        g.col_pool = vec![1, 2];
        g.place_bubble(242, 3);
        assert_eq!(g.score, 12);
        assert_eq!(g.score_txt.len(), 3);
    }

    #[test]
    fn score_text_entries_are_the_combo_times_ten_and_live_91_frames() {
        // The cart writes `s.."0"` (i.e. s*10) and starts a match entry at 30.
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (240, 3), (241, 3)]);
        g.col_pool = vec![1, 2];
        g.place_bubble(242, 3);
        let texts: Vec<_> = g.score_txt.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, vec!["20", "40", "60"]);
        assert!(g.score_txt.iter().all(|s| s.age == 30));
        assert!(g.score_txt.iter().all(|s| s.colour == col_convert(3)));

        for _ in 0..90 {
            g.tick_score_text();
        }
        assert_eq!(g.score_txt.len(), 3, "still visible at age 120");
        g.tick_score_text();
        assert!(g.score_txt.is_empty(), "deleted at age 121");
    }

    #[test]
    fn unrooted_cells_also_score_and_are_text_at_age_zero() {
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (240, 3), (241, 3), (100, 4)]);
        g.col_pool = vec![1, 2];
        g.place_bubble(242, 3);
        // 3 matches (2+4+6) then the hanging cell (8).
        assert_eq!(g.score, 20);
        assert_eq!(g.score_txt.len(), 4);
        assert_eq!(g.score_txt[3].text, "80");
        assert_eq!(g.score_txt[3].age, 0);
    }

    #[test]
    fn the_arena_edge_is_exclusive_at_one_hundred_and_twenty_one() {
        // `ceil(x) - b_r < 4` on the left and `ceil(x) + b_r-1 > 124` on the
        // right, so cell centres live in [8, 121].
        assert!(!out_of_arena(int(121), int(64)));
        assert!(out_of_arena(int(122), int(64)));
        assert!(!out_of_arena(int(8), int(64)));
        assert!(out_of_arena(int(7), int(64)));
        assert!(!out_of_arena(int(64), int(121)));
        assert!(out_of_arena(int(64), int(122)));
        // The cart ceils first, so 121.2 is 122 and therefore out.
        assert!(!out_of_arena(from_f64(121.0), int(64)));
        assert!(out_of_arena(from_f64(121.2), int(64)));
    }

    #[test]
    fn a_cell_at_the_edge_ends_the_game() {
        // Cell 270 is row 12, col 18: b2px = (128, 64), past the right wall.
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (270, 3)]);
        assert!(g.check_over());
        assert!(g.over);
        assert!(g.board.0.is_empty(), "the board is emptied");
        // The gun's loaded bubble falls too: the cart does `f_bubs = bubs`.
        assert_eq!(g.f_bubs.len(), 3, "two gumballs and the loaded bubble fall");
        assert_eq!(g.now_falling, 3);
    }

    #[test]
    fn a_cell_just_inside_the_edge_does_not_end_the_game() {
        // Cell 269 is row 12, col 17: b2px = (120, 64), 120 + 3 = 123 <= 124.
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (269, 3)]);
        assert!(!g.check_over());
        assert!(!g.over);
        assert_eq!(g.board.0.len(), 2);
    }

    #[test]
    fn the_falling_board_is_sorted_from_the_top() {
        let mut g = Game::new(7);
        // 262 at (64,64), 269 at (120,64), 250 is row 11 col 19: (140, 57) at rest.
        g.board = board_from(&[(CENTER, ANCHOR), (269, 3), (250, 4)]);
        g.check_over();
        let ys: Vec<i32> = g.f_bubs.iter().map(|b| to_int(b.y)).collect();
        assert!(ys.windows(2).all(|w| w[0] <= w[1]), "ascending y: {ys:?}");
    }

    #[test]
    fn a_cleared_board_starts_the_round_transition() {
        let mut g = Game::new(7);
        g.col_pool.clear();
        g.step(&still());
        assert!(g.round_clear);
        assert_eq!(g.col_cnt, 3, "the next round adds a colour");
        assert_eq!(g.round_clear_t, g.t);
    }

    #[test]
    fn the_next_round_starts_two_hundred_and_ten_frames_later() {
        let mut g = Game::new(7);
        g.col_pool.clear();
        g.step(&still());
        let cleared = g.round_clear_t;
        for _ in 0..(ROUND_CLEAR_FRAMES as usize) {
            g.step(&still());
        }
        assert!(g.round_clear, "still in the transition");
        assert_eq!(g.t, cleared + ROUND_CLEAR_FRAMES);
        g.step(&still());
        assert!(!g.round_clear, "the new round is running");
    }

    #[test]
    fn a_new_round_relays_the_level_and_resets_the_tries() {
        let mut g = Game::new(7);
        let col_cnt = g.col_cnt;
        g.new_round();
        assert_eq!(g.col_cnt, col_cnt, "col_cnt is bumped by check_clear only");
        assert_eq!(g.board.0.len(), 61);
        assert_eq!(g.shot_cnt, SHOT_MAX);
        assert_eq!(g.shot_count, 1);
        assert_eq!(g.spin.torque, from_f64(0.05));
        assert_eq!(g.bubs.len(), 1);
        assert!(!g.bubs[0].fired);
        assert!(g.f_bubs.is_empty());
        assert!(g.score_txt.is_empty());
        assert!(!g.round_clear);
        assert_eq!(g.now_falling, 0);
    }

    #[test]
    fn the_sixth_colour_brings_bonus_shots() {
        let mut g = Game::new(7);
        g.col_cnt = 6;
        g.bonus_shots = -1;
        g.new_round();
        assert_eq!(g.bonus_shots, 0);
        assert_eq!(g.shot_count, 1);
        g.new_round();
        assert_eq!(g.bonus_shots, 1);
        assert_eq!(g.shot_count, 2, "one extra shot per six-colour round");
    }

    #[test]
    fn drop_bubs_with_a_short_list_releases_one_bubble_a_frame() {
        // `drop_t % flr(rnd(#f_bubs/20))`: with 3 bubbles the divisor is 0, and
        // PICO-8's `% 0` is 0, so one bubble is released every frame.
        let mut g = Game::new(7);
        g.f_bubs = (0..3)
            .map(|i| crate::sim::bubble::Bubble {
                x: int(40 + i * 8),
                y: int(40),
                falling: false,
                ..crate::sim::bubble::Bubble::at_gun()
            })
            .collect();
        g.now_falling = 3;
        g.drop_bubs();
        assert_eq!(
            g.f_bubs.iter().filter(|b| b.falling).count(),
            1,
            "exactly one bubble starts falling"
        );
        assert_eq!(g.now_falling, 2);
    }

    #[test]
    fn nothing_falls_before_the_board_is_released() {
        let mut g = Game::new(7);
        g.drop_bubs(); // no f_bubs, now_falling 0: must not panic or divide by zero
        assert!(g.f_bubs.is_empty());
    }

    #[test]
    fn four_player_misses_hand_the_mass_two_shots_then_three_then_four() {
        // The cart's off-by-one: `board_shots = shot_count` is checked *after* the
        // next bubble is fired, so board_shots = 1 means two return shots.
        let mut g = Game::new(7);
        for expected in [2, 3, 4] {
            for _ in 0..4 {
                g.update_tries();
            }
            assert!(!g.player_turn, "four tries end the player's turn");

            let mut fired = 0;
            for _ in 0..2000 {
                let before = g.bubs.iter().filter(|b| b.fired).count();
                g.step(&still());
                let after = g.bubs.iter().filter(|b| b.fired).count();
                if before == 0 && after == 1 {
                    fired += 1;
                }
                if g.player_turn {
                    break;
                }
            }
            assert_eq!(fired, expected, "the mass gets {expected} shots back");
            assert!(g.player_turn, "the turn comes back");
            assert_eq!(g.shot_cnt, SHOT_MAX);
        }
        assert_eq!(g.shot_count, 4, "one more return shot each punishment");
    }

    #[test]
    fn a_hiscore_only_moves_up_and_is_saved() {
        let dir = std::env::temp_dir().join(format!("bgs-m5-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("save.dat");
        let _ = std::fs::remove_file(&path);

        let mut g = Game::new_with_save(7, crate::save::Save::default(), Some(path.clone()));
        g.score = 500;
        g.hiscore_check();
        assert_eq!(g.hiscore, 500);
        assert!(g.new_hiscore);
        assert_eq!(crate::save::Save::load(&path).hiscore, 500);

        g.score = 400;
        g.hiscore_check();
        assert_eq!(g.hiscore, 500, "a lower score does not overwrite it");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn firing_into_the_mass_after_a_game_over_does_nothing() {
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (270, 3)]);
        g.check_over();
        let board = g.board.0.len();
        g.step(&fired());
        g.step(&still());
        assert_eq!(g.board.0.len(), board, "no bubble is dealt after over");
        assert!(g.bubs.iter().all(|b| !b.fired));
    }

    #[test]
    fn six_thousand_frames_of_play_stay_sane() {
        // A smoke test for the whole loop: the mass shoots back, boards grow, runs
        // pop, rounds can clear. It has to stay inside sane bounds and never panic.
        let mut g = Game::new(7);
        for frame in 0..6_000u32 {
            let input = if frame % 150 == 0 { fired() } else { still() };
            g.step(&input);
            assert!(g.score >= 0, "frame {frame}");
            assert!(g.board.0.len() <= 525, "frame {frame}");
            assert!(g.bubs.len() <= 8, "frame {frame}: bubbles piling up");
            assert!(g.f_bubs.len() < 1_000, "frame {frame}");
            assert!((2..=6).contains(&g.col_cnt), "frame {frame}");
            if g.over {
                assert!(g.board.0.is_empty(), "frame {frame}: board after game over");
            }
        }
        assert_eq!(g.t, 6_000);
    }

    #[test]
    fn the_easing_slides_a_card_into_place_and_stops_exactly() {
        // The HUD card starts off-screen at -28 and is pulled to 12.
        let (mut y, mut dy) = (int(-28), 0);
        let mut frames = 0;
        while y != int(12) && frames < 400 {
            let (ny, ndy) = ease_in_out(y, int(12), dy);
            y = ny;
            dy = ndy;
            frames += 1;
        }
        assert_eq!(y, int(12), "landed on the target");
        assert!(frames < 200, "took {frames} frames");
        assert!(frames > 5, "it eases instead of snapping");
    }

    #[test]
    fn the_easing_does_nothing_when_already_there() {
        let (y, dy) = ease_in_out(int(12), int(12), 0);
        assert_eq!((y, dy), (int(12), 0));
    }

    #[test]
    fn every_event_the_cart_sounded_queues_the_same_effect() {
        // fire
        let mut g = Game::new(7);
        g.step(&fired());
        assert!(g.take_sfx().contains(&18), "fire");

        // a wall bounce
        let mut g = Game::new(7);
        g.gun.a = A_MIN;
        g.step(&fired());
        let mut heard = Vec::new();
        for _ in 0..60 {
            g.step(&still());
            heard.extend(g.take_sfx());
        }
        assert!(heard.contains(&8), "bounce: {heard:?}");

        // a match, and the placement that found it
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (240, 3), (241, 3)]);
        g.col_pool = vec![1, 2];
        g.place_bubble(242, 3);
        assert!(g.take_sfx().contains(&12), "match");

        // a placement that matches nothing is a "hit"
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR)]);
        g.place_bubble(500, 2);
        assert!(g.take_sfx().contains(&8), "hit");

        // the five-bounce give-up
        let mut g = Game::new(7);
        g.gun.a = A_MIN;
        g.step(&fired());
        let mut heard = Vec::new();
        for _ in 0..400 {
            g.step(&still());
            heard.extend(g.take_sfx());
        }
        assert!(heard.contains(&19), "missed completely: {heard:?}");

        // the mass's return shot
        let mut g = Game::new(7);
        for _ in 0..4 {
            g.update_tries();
        }
        let mut heard = Vec::new();
        for _ in 0..200 {
            g.step(&still());
            heard.extend(g.take_sfx());
        }
        assert!(heard.contains(&21), "rebuttal: {heard:?}");
    }

    #[test]
    fn the_music_calls_are_the_carts() {
        // A new game starts the round theme.
        let mut g = Game::new(7);
        assert!(g.take_music().contains(&16), "round music");

        // Clearing the board plays the clear jingle.
        let mut g = Game::new(7);
        g.col_pool.clear();
        g.step(&still());
        assert!(g.take_music().contains(&34), "round clear");

        // And the wall ends it.
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (270, 3)]);
        g.check_over();
        assert!(g.take_music().contains(&32), "game over");
    }

    #[test]
    fn a_drain_takes_everything_once() {
        let mut g = Game::new(7);
        g.step(&fired());
        assert!(!g.take_sfx().is_empty());
        assert!(g.take_sfx().is_empty(), "a second drain finds nothing");
    }
}
