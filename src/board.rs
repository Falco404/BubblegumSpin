//! The board: which cells hold which colour.
//!
//! The cart keeps this as a Lua table keyed by cell index, and iterates it with
//! `pairs()`. This port uses a `BTreeMap`, so every iteration is in ascending
//! cell order — the deterministic choice recorded in `docs/decisions.md`.
//!
//! Cell values are the cart's own encoding, kept as-is: the colour is `v % 10`,
//! and popping/exploding add 10/300 on top (M4 onward). `keys()` skips values
//! >= 10, which is what keeps animating cells out of the matching and rooting
//! passes during the frames they animate.

use std::collections::BTreeMap;

use crate::consts::LEVEL;
use crate::grid::CENTER;
use crate::p8math::Rng;

/// Cell index -> cell value. Absent keys are empty cells.
///
/// A `BTreeMap` (not a `HashMap`) so iteration is always ascending by cell index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Board(pub BTreeMap<u16, u8>);

impl Board {
    /// An empty board.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// cart `set_level()`: lay out `LEVEL`, replacing every non-empty cell with
    /// a random colour except the white anchor.
    ///
    /// The cart also has a `v == 7 and b_r == 3` branch that recolours the wall
    /// differently for the small-gumball mode; `b_r` is always 4 in this game, so
    /// both 7 and 9 fall through to the random colour and the branch is dead.
    pub fn set_level(&mut self, rng: &mut Rng, col_cnt: i32) {
        self.0.clear();
        for (i, &v) in LEVEL.iter().enumerate() {
            match v {
                0 => {}
                8 => {
                    self.0.insert(i as u16, 8);
                }
                _ => {
                    let c = rng.colour(col_cnt);
                    self.0.insert(i as u16, c);
                }
            }
        }
    }

    /// The value of one cell, if it is occupied.
    #[inline]
    pub fn get(&self, i: u16) -> Option<u8> {
        self.0.get(&i).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p8math::Rng;

    #[test]
    fn set_level_gives_61_coloured_cells() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        assert_eq!(board.0.len(), 61);
    }

    #[test]
    fn set_level_keeps_the_white_anchor() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        assert_eq!(board.0.get(&crate::grid::CENTER), Some(&8));
    }

    #[test]
    fn set_level_only_uses_the_requested_colours() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        for (&k, &v) in board.0.iter() {
            if k == crate::grid::CENTER {
                continue;
            }
            assert!((1..=2).contains(&v), "cell {k} holds {v}");
        }
    }

    #[test]
    fn set_level_is_deterministic_for_a_seed() {
        let mut a = Board::new();
        let mut b = Board::new();
        a.set_level(&mut Rng::new(42), 3);
        b.set_level(&mut Rng::new(42), 3);
        assert_eq!(a.0, b.0);

        let mut c = Board::new();
        c.set_level(&mut Rng::new(43), 3);
        assert_ne!(a.0, c.0);
    }

    #[test]
    fn set_level_replaces_whatever_was_there() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        board.set_level(&mut Rng::new(1), 2);
        assert_eq!(board.0.len(), 61);
    }
}
