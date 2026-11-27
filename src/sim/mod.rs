//! The pure simulation: board, spin, gun and the fired bubble. No rendering, no
//! audio.
//!
//! Everything here is numbers and grids; `main.rs` reads input, calls
//! [`Game::step`] 60 times a second, and `gfx::scenes` draws the result.

pub mod bubble;
pub mod gun;
pub mod spin;

use crate::board::Board;
use crate::fixed::from_f64;
use crate::p8math::Rng;
use bubble::{Bubble, SHOT_MAX};
use gun::Gun;
use spin::Spin;

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

    /// Whose shot it is. `false` means the mass is shooting back (M5.2).
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

    /// Previous state of the fire button, for the press edge.
    prev_fire: bool,
}

impl Game {
    /// cart `new_game()` + `new_round()`: a fresh level, two colours on the
    /// conveyor, the first bubble dealt, and the round opened with `torque=.05`.
    pub fn new(seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let mut board = Board::new();
        board.set_level(&mut rng, 2);
        let col_pool = board.pool();

        let mut game = Self {
            board,
            spin: Spin::new(),
            gun: Gun::new(),
            rng,
            bubs: Vec::new(),
            f_bubs: Vec::new(),
            player_turn: true,
            in_flight: false,
            in_play: false,
            reloading: false,
            popping: false,
            bounces: 0,
            now_falling: 0,
            col_pool,
            nxt_col: 0,
            nxt_nxt_col: 0,
            shot_cnt: SHOT_MAX,
            shot_count: 1,
            last_shot_max: SHOT_MAX,
            board_shots: 0,
            hud_t: 0,
            prev_fire: false,
        };

        // cart `new_round()`: two colours on the conveyor, then `reset_bubble()`
        // deals the first and shifts the second down.
        game.nxt_nxt_col = game.rnd_col();
        game.nxt_col = game.rnd_col();
        game.reset_bubble();

        // `new_round()` ends with `torque=.05`: every round opens with the mass
        // turning slowly, and the 0.96 decay (110 frames) takes it to a stop.
        game.spin.torque = from_f64(0.05);
        game
    }

    /// One frame at 60 Hz, in the cart's `update_game()` order.
    pub fn step(&mut self, input: &Input) {
        self.update_falling();
        self.spin.step();
        self.popping = self.board.advance_animations();

        self.gun.update(input.left, input.right, input.up);

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
}
