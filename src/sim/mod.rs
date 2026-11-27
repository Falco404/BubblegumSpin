//! The pure simulation: board, spin, gun, the fired bubble and the rules. No
//! rendering, no audio.
//!
//! Everything here is numbers and grids; `main.rs` reads input, calls
//! [`Game::step`] 60 times a second, and `gfx::scenes` draws the result.

pub mod bubble;
pub mod gun;
pub mod rules;
pub mod spin;

use std::path::PathBuf;

use crate::board::Board;
use crate::p8math::Rng;
use crate::save::Save;
use bubble::{Bubble, SHOT_MAX};
use gun::Gun;
use rules::{ScoreText, MIN_COL_CNT, ROUND_CLEAR_FRAMES};
use spin::{Camera, Fade, Shake, Spin};

/// What the player is holding this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    /// The cart's X button. Firing is edge-triggered (see [`Game::step`]).
    pub fire: bool,
}

/// The whole game state.
#[derive(Debug, Clone)]
pub struct Game {
    pub board: Board,
    pub spin: Spin,
    pub gun: Gun,
    pub rng: Rng,

    /// Bubbles that are in the gun or in flight (cart `bubs`).
    pub bubs: Vec<Bubble>,
    /// Bubbles that were given up and are falling off the screen (cart `f_bubs`).
    pub f_bubs: Vec<Bubble>,

    /// Whose shot it is. `false` means the mass is shooting back.
    pub player_turn: bool,
    /// A shot is in the air; nothing else is fired until it resolves.
    pub in_flight: bool,
    /// The shot has cleared the gun, so the top and bottom walls count.
    pub in_play: bool,
    /// A bubble was placed and the conveyor may deal the next one.
    pub reloading: bool,
    /// Something on the board is animating; the cart holds the reload until it
    /// has finished (`if reloading and not popping`).
    pub popping: bool,

    pub bounces: u32,
    pub now_falling: usize,
    /// Frames since the game-over drop started (cart `drop_t`).
    pub drop_t: i32,

    /// The colours currently in play (cart `col_pool`), ascending cell order.
    pub col_pool: Vec<u16>,
    /// The colour in the gun and the one after it (cart `nxt_col`/`nxt_nxt_col`).
    pub nxt_col: u16,
    pub nxt_nxt_col: u16,

    /// Shots that clear nothing before the player's turn ends (cart `shot_cnt`).
    pub shot_cnt: i32,
    /// How many return shots the mass gets, growing each punishment.
    pub shot_count: i32,
    pub last_shot_max: i32,
    /// Return shots the mass still has this punishment.
    pub board_shots: i32,
    /// Frames left on the "your turn" HUD hint (cart `hud_t`).
    pub hud_t: i32,

    /// The running score, in the cart's units (the score card shows `score*10`).
    pub score: i32,
    pub hiscore: u32,
    pub new_hiscore: bool,
    /// Floating score numbers (cart `score_txt`).
    pub score_txt: Vec<ScoreText>,
    /// Colours in this round, 2..=6 (cart `col_cnt`).
    pub col_cnt: i32,
    /// Extra shots earned by reaching the six-colour round (cart `bonus_shots`).
    pub bonus_shots: i32,

    /// The game is over: the board has fallen, no more shots are fired.
    pub over: bool,
    /// The board is cleared and the next round is about to start.
    pub round_clear: bool,
    pub round_clear_t: i32,
    /// Frames since the game started (cart `t`).
    pub t: i32,

    pub fade: Fade,
    pub camera: Camera,
    pub shake: Shake,

    pub save: Save,
    /// Where `save.dat` lives; `None` in tests and in the headless renderer.
    pub save_path: Option<PathBuf>,

    /// Previous state of the fire button, for the press edge.
    prev_fire: bool,
}

impl Game {
    /// A game with no save file behind it: tests and the headless renderer.
    pub fn new(seed: u32) -> Self {
        Self::new_with_save(seed, Save::default(), None)
    }

    /// cart `new_game()`: a hiscore from the save, two colours, and the first
    /// round dealt.
    pub fn new_with_save(seed: u32, save: Save, save_path: Option<PathBuf>) -> Self {
        let mut game = Self {
            board: Board::new(),
            spin: Spin::new(),
            gun: Gun::new(),
            rng: Rng::new(seed),
            bubs: Vec::new(),
            f_bubs: Vec::new(),
            player_turn: true,
            in_flight: false,
            in_play: false,
            reloading: false,
            popping: false,
            bounces: 0,
            now_falling: 0,
            drop_t: 0,
            col_pool: Vec::new(),
            nxt_col: 0,
            nxt_nxt_col: 0,
            shot_cnt: SHOT_MAX,
            shot_count: 1,
            last_shot_max: SHOT_MAX,
            board_shots: 0,
            hud_t: 0,
            score: 0,
            hiscore: save.hiscore,
            new_hiscore: false,
            score_txt: Vec::new(),
            col_cnt: MIN_COL_CNT,
            bonus_shots: -1,
            over: false,
            round_clear: false,
            round_clear_t: 0,
            t: 0,
            fade: Fade::default(),
            camera: Camera::default(),
            shake: Shake::default(),
            save,
            save_path,
            prev_fire: false,
        };
        // cart `new_game()`: `col_cnt = min_col_cnt`, `bonus_shots = -1`, then
        // `new_round()` does the rest (level, conveyor, first bubble, torque .05).
        game.new_round();
        game
    }

    /// One frame at 60 Hz: the cart's `_update60` around its `update_game()`.
    pub fn step(&mut self, input: &Input) {
        self.t += 1;
        self.fade.step();

        if self.over {
            // cart `update_over`: no shooting, just the cascade.
            self.drop_bubs();
            self.update_falling();
        } else {
            self.update_game(input);
        }

        // The cart ages score text in `draw_game`; doing it here keeps the
        // simulation complete without a screen.
        self.tick_score_text();
        self.camera.step();
    }

    /// cart `update_game()`, in the cart's order.
    fn update_game(&mut self, input: &Input) {
        self.update_falling();
        self.spin.step();
        self.popping = self.board.advance_animations();

        if self.col_pool.is_empty() && !self.popping && !self.round_clear {
            self.check_clear();
        }
        if self.round_clear && self.t > self.round_clear_t + ROUND_CLEAR_FRAMES {
            self.new_round();
        }

        self.update_hud();

        if self.check_over() || self.round_clear {
            return;
        }

        self.gun.update(input.left, input.right, input.up);

        // The mass's return shot: fired as soon as its turn starts (M5.2).
        if !self.player_turn && !self.in_flight {
            self.in_flight = true;
            self.fire_bubs();
        }

        if self.reloading && !self.popping {
            self.reset_bubble();
        }

        // The cart uses `btnp(X)`: only a fresh press fires, not a held button.
        let pressed = input.fire && !self.prev_fire;
        self.prev_fire = input.fire;
        if pressed && !self.in_flight {
            self.in_flight = true;
            self.fire();
        }

        if !self.in_flight {
            return;
        }

        let mut i = 0;
        while i < self.bubs.len() {
            if self.bubs[i].fired {
                let before = self.bubs.len();
                if self.update_bubble(i) {
                    return;
                }
                // A bubble that gave up was removed; do not skip its successor.
                if self.bubs.len() == before {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    /// cart `update_hud()`: the hint timer. The card easing and drawing are M6.
    fn update_hud(&mut self) {
        if self.hud_t > 0 {
            self.hud_t -= 1;
        }
    }

    /// The cart's `#keys(board)`, used as the torque divisor.
    pub fn mass(&self) -> usize {
        self.board.keys().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::{from_f64, to_int};
    use crate::grid::CENTER;

    #[test]
    fn a_new_game_has_the_level_ready() {
        let g = Game::new(7);
        assert_eq!(g.board.0.len(), 61);
        assert_eq!(g.board.get(CENTER), Some(8));
        assert_eq!(g.mass(), 60);
        assert_eq!(g.spin.var, 0);
        assert_eq!(g.spin.torque, from_f64(0.05));
        assert_eq!(g.col_cnt, 2);
        assert_eq!(g.score, 0);
    }

    #[test]
    fn a_new_game_has_one_bubble_in_the_gun_and_two_on_the_conveyor() {
        let g = Game::new(7);
        assert_eq!(g.bubs.len(), 1);
        assert!(!g.bubs[0].fired);
        assert_eq!((to_int(g.bubs[0].x), to_int(g.bubs[0].y)), (64, 121));
        assert!(g.col_pool.contains(&g.nxt_col));
        assert!(g.col_pool.contains(&g.nxt_nxt_col));
        assert_eq!(g.shot_cnt, SHOT_MAX);
        assert_eq!(g.shot_count, 1);
    }

    #[test]
    fn the_round_opens_with_a_slow_spin_that_decays_to_a_stop() {
        // The cart's `new_round()` sets torque=.05; with the 0.96 decay that is
        // the 110-frame, 79_886 (1.21896 turns) walk already pinned in spin.rs.
        let mut g = Game::new(7);
        for _ in 0..200 {
            g.step(&Input::default());
        }
        assert_eq!(g.spin.torque, 0, "the drift has stopped");
        assert_eq!(g.spin.var, 79_886);
    }

    #[test]
    fn aiming_and_firing_are_independent() {
        let mut g = Game::new(7);
        let left = Input {
            left: true,
            ..Default::default()
        };
        for _ in 0..10 {
            g.step(&left);
        }
        assert_eq!(g.gun.a, from_f64(0.5) - 10 * 256);
        assert!(!g.in_flight, "aiming does not fire");
    }

    #[test]
    fn firing_moves_the_bubble_up_the_screen() {
        let mut g = Game::new(7);
        let fire = Input {
            fire: true,
            ..Default::default()
        };
        g.step(&fire);
        assert!(g.in_flight);
        assert!(g.bubs[0].y < from_f64(121.0), "y = {}", g.bubs[0].y);
    }

    #[test]
    fn the_hiscore_comes_from_the_save_and_not_from_the_game() {
        let save = Save {
            hiscore: 9_000,
            completed: true,
        };
        let g = Game::new_with_save(7, save, None);
        assert_eq!(g.hiscore, 9_000);
        assert_eq!(g.score, 0);
        assert!(!g.new_hiscore);
    }

    #[test]
    fn t_counts_the_frames() {
        let mut g = Game::new(7);
        for _ in 0..17 {
            g.step(&Input::default());
        }
        assert_eq!(g.t, 17);
    }
}
