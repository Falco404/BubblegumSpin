//! The board: which cells hold which colour.
//!
//! The cart keeps this as a Lua table keyed by cell index, and iterates it with
//! `pairs()`. This port uses a `BTreeMap`, so every iteration is in ascending
//! cell order — the deterministic choice recorded in `docs/decisions.md`.
//!
//! Cell values are the cart's own encoding, kept as-is: the colour is `v % 10`,
//! and popping/exploding add 10/300 on top (M4 onward). `keys()` skips any value
//! of 10 or more, which is what keeps animating cells out of the matching and
//! rooting passes during the frames they animate.

use std::collections::{BTreeMap, BTreeSet};

use crate::consts::LEVEL;
use crate::grid::{neighbors, CENTER};
use crate::p8math::Rng;

/// The white anchor cell's value.
pub const ANCHOR: u16 = 8;

/// Cell index -> cell value. Absent keys are empty cells.
///
/// Values are `u16`, not `u8`: a cell holds `colour + 10*popping + 300*exploded`
/// and the cart only deletes a popper once `v > 600` (cart line 707), so values
/// reach ~609 mid-animation. `u8` would overflow in M4.
///
/// A `BTreeMap` (not a `HashMap`) so iteration is always ascending by cell index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Board(pub BTreeMap<u16, u16>);

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
                    self.0.insert(i as u16, ANCHOR);
                }
                _ => {
                    let c = rng.colour(col_cnt);
                    self.0.insert(i as u16, u16::from(c));
                }
            }
        }
    }

    /// The value of one cell, if it is occupied.
    #[inline]
    pub fn get(&self, i: u16) -> Option<u16> {
        self.0.get(&i).copied()
    }

    /// cart `keys(board)`: the settled cells, ascending, without the anchor.
    ///
    /// `v < 10` is what keeps animating cells (colour + 10*pop + 300*explode) out
    /// of this list, and `k != 262` keeps the white anchor out so it is never
    /// treated as unrooted.
    pub fn keys(&self) -> Vec<u16> {
        self.0
            .iter()
            .filter(|(&k, &v)| v < 10 && k != CENTER)
            .map(|(&k, _)| k)
            .collect()
    }

    /// cart `get_matching(i, col)`: every cell connected to `i` with exactly that
    /// value. Sorted ascending, so callers get the same order as the cart's
    /// `pairs()` replacement.
    ///
    /// The cart recurses through `neighbors()` including the `nil` ones; here the
    /// recursion stops wherever `get()` returns `None`, which is the same thing.
    pub fn get_matching(&self, i: u16, col: u16) -> Vec<u16> {
        let mut matches = BTreeSet::new();
        let mut visited = BTreeSet::new();
        self.mtch_rec(i, col, &mut matches, &mut visited);
        matches.into_iter().collect()
    }

    /// cart `get_mtch_rec(i, col, m, v)`.
    fn mtch_rec(&self, i: u16, col: u16, m: &mut BTreeSet<u16>, v: &mut BTreeSet<u16>) {
        if !v.insert(i) {
            return;
        }
        if self.get(i) != Some(col) {
            return;
        }
        m.insert(i);
        for n in neighbors(i).into_iter().flatten() {
            self.mtch_rec(n, col, m, v);
        }
    }

    /// cart `get_unrooted()`: settled cells not reachable from the anchor.
    ///
    /// The flood starts at the anchor's six neighbours, not at the anchor itself
    /// (the cart does the same), and stops at empty cells and at anything
    /// animating (`value > 10`).
    pub fn get_unrooted(&self) -> Vec<u16> {
        let mut unrooted: BTreeSet<u16> = self.keys().into_iter().collect();
        let mut visited = BTreeSet::new();
        for rn in neighbors(CENTER).into_iter().flatten() {
            self.ur_rec(rn, &mut unrooted, &mut visited);
        }
        unrooted.into_iter().collect()
    }

    /// cart `get_ur_rec(i, u, v)`.
    fn ur_rec(&self, i: u16, u: &mut BTreeSet<u16>, v: &mut BTreeSet<u16>) {
        if !v.insert(i) {
            return;
        }
        let Some(value) = self.get(i) else {
            return;
        };
        if value > 10 {
            return;
        }
        u.remove(&i);
        for n in neighbors(i).into_iter().flatten() {
            self.ur_rec(n, u, v);
        }
    }

    /// cart `update_pool()`'s result: the distinct colour of every settled cell,
    /// excluding the anchor's colour, in cell order.
    pub fn pool(&self) -> Vec<u16> {
        let mut pool = Vec::new();
        for k in self.keys() {
            let value = self.0[&k];
            let colour = value % 10;
            if !pool.contains(&colour) && value != ANCHOR {
                pool.push(colour);
            }
        }
        pool
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

    /// A board from explicit (cell, value) pairs, for tests that need a shape the
    /// level does not have.
    fn board_from(pairs: &[(u16, u16)]) -> Board {
        Board(pairs.iter().copied().collect())
    }

    #[test]
    fn keys_are_the_settled_cells_without_the_anchor() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        let k = board.keys();
        assert_eq!(k.len(), 60);
        assert!(!k.contains(&crate::grid::CENTER));
        for i in k {
            assert!(board.get(i).unwrap() < 10);
        }
    }

    #[test]
    fn keys_are_in_ascending_index_order() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        let k = board.keys();
        assert!(k.windows(2).all(|w| w[0] < w[1]), "not ascending: {k:?}");
    }

    #[test]
    fn keys_skips_animating_cells() {
        // 315 = colour 5 popping, 20 = colour 0 popping; both are >= 10.
        let board = board_from(&[(1, 3), (2, 315), (3, 20), (crate::grid::CENTER, 8)]);
        assert_eq!(board.keys(), vec![1]);
    }

    #[test]
    fn matching_finds_the_connected_run_only() {
        // 240-241-242 are adjacent (see grid tests); 300 is the same colour but
        // far away.
        let board = board_from(&[(240, 4), (241, 4), (242, 4), (300, 4)]);
        assert_eq!(board.get_matching(241, 4), vec![240, 241, 242]);
    }

    #[test]
    fn matching_from_an_empty_cell_is_empty() {
        let board = board_from(&[(240, 4), (241, 4)]);
        assert!(board.get_matching(500, 4).is_empty());
    }

    #[test]
    fn matching_stops_at_other_colours() {
        // 240 red, 241 blue, 242 red: the blues cut the reds apart.
        let board = board_from(&[(240, 4), (241, 2), (242, 4)]);
        assert_eq!(board.get_matching(240, 4), vec![240]);
    }

    #[test]
    fn matching_is_sorted_and_unique() {
        let board = board_from(&[(240, 4), (241, 4), (242, 4)]);
        let m = board.get_matching(240, 4);
        assert!(m.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn a_fresh_board_has_nothing_unrooted() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        assert!(board.get_unrooted().is_empty());
    }

    #[test]
    fn a_detached_blob_is_unrooted() {
        // Only the anchor plus two cells nowhere near it.
        let board = board_from(&[(crate::grid::CENTER, 8), (100, 3), (101, 3)]);
        assert_eq!(board.get_unrooted(), vec![100, 101]);
    }

    #[test]
    fn removing_every_anchor_neighbour_unroots_everything() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        for n in crate::grid::neighbors(crate::grid::CENTER).iter().flatten() {
            board.0.remove(n);
        }
        assert_eq!(board.get_unrooted().len(), 54); // 60 - the 6 roots
    }

    #[test]
    fn removing_one_interior_cell_disconnects_nothing() {
        // Cell 263 sits inside the blob; the level is thick enough that losing it
        // keeps every other cell attached. (The M2 issue guessed otherwise; the
        // real flood fill says so — see the PR.)
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        board.0.remove(&263);
        assert!(board.get_unrooted().is_empty());
    }

    #[test]
    fn pool_is_the_distinct_colours_of_the_settled_cells() {
        let board = board_from(&[(200, 2), (201, 1), (202, 2), (crate::grid::CENTER, 8)]);
        assert_eq!(board.pool(), vec![2, 1]);
    }

    #[test]
    fn pool_ignores_the_anchor_colour() {
        let board = board_from(&[(200, 1), (crate::grid::CENTER, 8)]);
        assert_eq!(board.pool(), vec![1]);
    }

    #[test]
    fn pool_on_a_fresh_board_has_at_most_col_cnt_colours() {
        let mut board = Board::new();
        board.set_level(&mut Rng::new(1), 2);
        let pool = board.pool();
        assert!(!pool.is_empty() && pool.len() <= 2, "pool: {pool:?}");
        assert!(pool.iter().all(|&c| (1..=2).contains(&c)));
    }
}
