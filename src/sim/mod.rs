//! The pure simulation: board, spin and gun. No rendering, no audio.
//!
//! Everything here is numbers and grids; `main.rs` reads input, calls
//! [`Game::step`] 60 times a second, and `gfx::scenes` draws the result.

pub mod gun;
pub mod spin;

use crate::board::Board;
use crate::fixed::Fix;
use crate::p8math::Rng;
use gun::Gun;
use spin::Spin;

/// What the player is holding this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    /// Debug-only: hold to keep the mass spinning. Not a cart button.
    pub debug_spin: bool,
}

/// Torque added per frame while the debug spin key is held. With the cart's
/// 0.96 decay this settles at about 0.02 turns per frame (7 degrees).
const DEBUG_TORQUE: Fix = 52; // 0.0008 in 16.16

/// The whole game state.
#[derive(Debug, Clone)]
pub struct Game {
    pub board: Board,
    pub spin: Spin,
    pub gun: Gun,
    pub rng: Rng,
}

impl Game {
    /// cart `init_game()`'s board: a fresh level, `col_cnt = min_col_cnt = 2`.
    pub fn new(seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let mut board = Board::new();
        board.set_level(&mut rng, 2);
        Self {
            board,
            spin: Spin::new(),
            gun: Gun::new(),
            rng,
        }
    }

    /// One frame at 60 Hz.
    pub fn step(&mut self, input: &Input) {
        self.gun.update(input.left, input.right, input.up);
        if input.debug_spin {
            self.spin.add_torque(DEBUG_TORQUE);
        }
        self.spin.step();
    }

    /// The cart's `#keys(board)`, used as the torque divisor.
    pub fn mass(&self) -> usize {
        self.board.keys().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::from_f64;
    use crate::grid::CENTER;

    #[test]
    fn a_new_game_has_the_level_ready() {
        let g = Game::new(7);
        assert_eq!(g.board.0.len(), 61);
        assert_eq!(g.board.get(CENTER), Some(8));
        assert_eq!(g.mass(), 60);
        assert_eq!(g.spin.var, 0);
        assert_eq!(g.spin.torque, 0);
    }

    #[test]
    fn nothing_moves_without_input() {
        let mut g = Game::new(7);
        for _ in 0..120 {
            g.step(&Input::default());
        }
        assert_eq!(g.spin.var, 0);
        assert_eq!(g.spin.torque, 0);
        assert_eq!(g.gun.a, from_f64(0.5));
    }

    #[test]
    fn the_debug_key_spins_the_mass_and_it_keeps_turning() {
        let mut g = Game::new(7);
        let holding = Input {
            debug_spin: true,
            ..Default::default()
        };
        for _ in 0..120 {
            g.step(&holding);
        }
        assert!(g.spin.torque.abs() > 1000, "torque = {}", g.spin.torque);
        let after_hold = g.spin.var;
        for _ in 0..60 {
            g.step(&Input::default());
        }
        // Torque decays but the angle kept going, so var is further from zero.
        assert!(g.spin.var > after_hold);
    }

    #[test]
    fn aiming_works_through_step() {
        let mut g = Game::new(7);
        let left = Input {
            left: true,
            ..Default::default()
        };
        for _ in 0..10 {
            g.step(&left);
        }
        assert_eq!(g.gun.a, from_f64(0.5) - 10 * 256);
    }
}
