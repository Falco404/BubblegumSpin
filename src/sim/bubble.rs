//! The fired bubble: flight, wall bounces, collision, sticking, and the turn
//! bookkeeping that follows a shot.
//!
//! cart: `fire`, `update_bubble`, `overlapping`, `get_dest_cell`, `place_bubble`,
//! `reset_bubble`, `update_tries`, `update_falling`.
//!
//! Two cart behaviours are reproduced on purpose because they change the game:
//!
//! * The overlap test runs against the bubble's position *before* this frame's
//!   move, so a bubble sticks up to one step (2.5 px) short of where it looks.
//! * A hue that matches nothing costs a try, and the fourth try that clears
//!   nothing ends the player's turn (`player_turn = false`). The mass's return
//!   shot is M5.2; until then a round waits there.

use crate::board::Board;
use crate::fixed::{flr, int, mul, to_int, Fix, HALF};
use crate::grid::{b2px, neighbors, B_R};
use crate::p8math::{atan2, cos, mid, sin, sqrt};
use crate::sim::spin::{calc_torque, px2rpx, rpx2px};
use crate::sim::Game;

/// cart `bub_spd`: 2.5 px per frame.
pub const BUB_SPD: Fix = 163_840;

/// cart `arena`: the walls the bubble bounces off.
pub const ARENA_LEFT: i32 = 4;
pub const ARENA_RIGHT: i32 = 124;
pub const ARENA_TOP: i32 = 4;
pub const ARENA_BOTTOM: i32 = 124;

/// cart: `if bounces>4` — the fifth wall bounce gives the shot up.
pub const BOUNCE_LIMIT: u32 = 4;

/// cart `shot_max`: shots that clear nothing before the player's turn ends.
pub const SHOT_MAX: i32 = 4;

/// cart `get_dest_cell`'s `.0833`: half a hex sector, in turns.
pub const SECTOR_OFFSET: Fix = 5_459;

/// cart `update_falling`'s gravity and drag.
pub const FALL_GRAVITY: Fix = 6_553; // 0.1
pub const FALL_DRAG: Fix = 58_982; // 0.9
/// cart: a falling bubble is deleted once `y > 136`.
pub const FALL_KILL: i32 = 136;

/// One bubble, whether it is in the gun, in flight or falling.
///
/// `Copy` so the update loop can take a snapshot the way the cart's local `_b`
/// does: the cart keeps working on its copy after `del(bubs, _b)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bubble {
    pub x: Fix,
    pub y: Fix,
    pub dx: Fix,
    pub dy: Fix,
    pub col: u16,
    pub fired: bool,
    pub falling: bool,
}

impl Bubble {
    /// A bubble sitting in the gun's mouth: `gun={x=64, y=121}`.
    pub fn at_gun() -> Self {
        Self {
            x: int(64),
            y: int(121),
            dx: 0,
            dy: 0,
            col: 0,
            fired: false,
            falling: false,
        }
    }
}

/// cart `overlapping(_b)`: the first settled cell whose rotated position is
/// closer than `2*b_r-1` = 7 px to the bubble.
///
/// The bubble's position is floored first (`flr(_b.x)`), and the scan goes in
/// ascending cell order — the deterministic replacement for `pairs()`. In a tie
/// (two equidistant cells) the lowest index wins, which is the one fidelity gap
/// a `BTreeMap` buys us.
pub fn overlapping(board: &Board, var: Fix, b: &Bubble) -> Option<u16> {
    let x2 = flr(b.x);
    let y2 = flr(b.y);
    let limit = int(2 * B_R - 1);
    for &k in board.0.keys() {
        let (gx, gy) = b2px(k);
        let (x, y) = px2rpx(int(gx), int(gy), var);
        let (dx, dy) = (x - x2, y - y2);
        if sqrt(mul(dx, dx) + mul(dy, dy)) < limit {
            return Some(k);
        }
    }
    None
}

/// cart `get_dest_cell(_h,_x,_y)`: which neighbour of the cell the bubble touched
/// the bubble is coming from, as a cell index.
///
/// `ni = flr((atan2(x-hx, y-hy) + .0833) * 6) + 1`, with `ni > 6 -> 1`. The
/// `+1` is because the cart's `neighbors()` table is 1-based; here it is the
/// array index plus one turned back into an index.
pub fn get_dest_cell(hit: u16, x: Fix, y: Fix, var: Fix) -> Option<u16> {
    let (hx, hy) = b2px(hit);
    let (x, y) = rpx2px(x, y, var);
    let ang = atan2(x - int(hx), y - int(hy));
    let ni = to_int(flr(mul(ang + SECTOR_OFFSET, int(6)))) + 1;
    let ni = if (1..=6).contains(&ni) { ni } else { 1 };
    neighbors(hit)[(ni - 1) as usize]
}

impl Game {
    /// cart `fire()`: send the bubble in the gun up the aim line at 2.5 px/frame.
    pub fn fire(&mut self) {
        let (ax, ay) = (sin(self.gun.a), cos(self.gun.a));
        if let Some(b) = self.bubs.first_mut() {
            b.dx = mul(ax, BUB_SPD);
            b.dy = mul(ay, BUB_SPD);
            b.fired = true;
        }
    }

    /// cart `rnd_col()`: `flr(rnd(col_pool))`, i.e. a random member of the pool.
    ///
    /// PICO-8's `rnd(table)` picks a member; the `flr` is a no-op on colours.
    /// An empty pool means the round is over (M5); the cart would return `nil`,
    /// so the guard just keeps the type honest.
    pub fn rnd_col(&mut self) -> u16 {
        if self.col_pool.is_empty() {
            return 1;
        }
        let n = self.col_pool.len() as i32;
        let i = to_int(mul(self.rng.rnd(), int(n))) as usize;
        self.col_pool[i.min(self.col_pool.len() - 1)]
    }

    /// cart `reset_bubble()`: deal the next colour and hand the player the turn.
    pub fn reset_bubble(&mut self) {
        self.bubs.push(Bubble {
            col: self.nxt_col,
            ..Bubble::at_gun()
        });
        self.nxt_col = self.nxt_nxt_col;
        self.nxt_nxt_col = self.rnd_col();

        self.bounces = 0;
        self.player_turn = true;
        self.in_play = false;
        self.in_flight = false;
        self.hud_t = 180;
        self.reloading = false;
    }

    /// cart `update_tries()`: one shot that cleared nothing.
    ///
    /// When `shot_cnt` runs out the player's turn ends and the mass gets
    /// `shot_count` shots back (M5.2). `last_shot_max` is set once to `shot_max`
    /// and never decremented, so the cart's `if last_shot_max==0` branch is dead
    /// and is not ported.
    pub fn update_tries(&mut self) {
        self.shot_cnt -= 1;
        if self.shot_cnt == 0 {
            self.player_turn = false;
            self.board_shots = self.shot_count;
            self.shot_count += 1;
            self.shot_cnt = self.last_shot_max;
        }
    }

    /// cart `place_bubble(i, col)`: put the colour into the cell, then run the
    /// matching pass.
    ///
    /// A run of three or more gets `+300` (the cart's "explode" step; the pop
    /// animation then adds 10 a frame until `> 600` deletes it). Anything left
    /// hanging gets `+10`. No run means the placement costs a try. Scoring and
    /// sound are M5/M7.
    pub fn place_bubble(&mut self, i: u16, col: u16) {
        self.board.0.insert(i, col);

        if self.player_turn {
            let matches = self.board.get_matching(i, col);
            if matches.len() > 2 {
                for m in matches {
                    if let Some(v) = self.board.0.get_mut(&m) {
                        *v += 300;
                    }
                }
                for u in self.board.get_unrooted() {
                    if let Some(v) = self.board.0.get_mut(&u) {
                        *v += 10;
                    }
                }
            } else {
                self.update_tries();
            }
        }

        self.col_pool = self.board.pool();
    }

    /// cart `update_falling()`: detached bubbles fall with gravity and drag.
    ///
    /// The cart sets `now_falling` and staggers the drop with `drop_bubs()`
    /// (M5.6); M4 only ever has one falling bubble, which falls immediately.
    pub fn update_falling(&mut self) {
        let mut i = 0;
        while i < self.f_bubs.len() {
            if self.f_bubs[i].falling {
                if self.f_bubs[i].y > int(FALL_KILL) {
                    self.f_bubs.remove(i);
                    continue;
                }
                let b = &mut self.f_bubs[i];
                b.x += b.dx;
                b.y += b.dy;
                b.dy += FALL_GRAVITY;
                b.dx = mul(b.dx, FALL_DRAG);
            }
            i += 1;
        }
    }

    /// cart's miss path: the bubble stops being fired, drops into `f_bubs` at
    /// half speed, and the shot is spent.
    fn give_up(&mut self, i: usize, b: &mut Bubble, removed: bool) {
        b.falling = true;
        b.fired = false;
        b.dx = mul(b.dx, HALF);
        b.dy = mul(b.dy, HALF);
        self.f_bubs.push(*b);
        self.now_falling = 1;
        if !removed {
            self.bubs.remove(i);
        }
        self.update_tries();
        // The branch is on the *new* `player_turn`: on the fourth miss
        // `update_tries` has just handed the turn to the mass (cart line 1855).
        if self.player_turn {
            self.reloading = true;
        } else {
            self.in_flight = false;
            self.board_shots -= 1;
        }
    }

    /// cart `update_bubble(_b)`: one frame of flight for `bubs[i]`.
    ///
    /// Returns `true` when the bubble stopped (`update_game` returns then, so no
    /// other bubble moves that frame).
    pub fn update_bubble(&mut self, i: usize) -> bool {
        let mut b = self.bubs[i];
        let (mut nx, mut ny) = (b.x + b.dx, b.y + b.dy);
        let mut removed = false;

        if self.player_turn {
            // `in_play` arms the top/bottom walls once the shot has cleared the
            // gun. It tests the *old* y, so the first frame after firing still
            // counts as "not in play".
            if b.y < int(120) {
                self.in_play = true;
            }

            if nx < int(ARENA_LEFT + B_R) || nx > int(ARENA_RIGHT - B_R - 1) {
                nx = mid(int(ARENA_LEFT), b.x, int(ARENA_RIGHT));
                b.dx = -b.dx;
                self.bounces += 1;
            }

            if self.in_play && (ny < int(ARENA_TOP + B_R) || ny > int(ARENA_BOTTOM - B_R)) {
                ny = mid(int(ARENA_TOP), b.y, int(ARENA_BOTTOM));
                b.dy = -b.dy;
                self.bounces += 1;
            }

            if self.bounces > BOUNCE_LIMIT {
                self.give_up(i, &mut b, false);
                removed = true;
            }
        }

        // The cart checks overlap at the *unmoved* position, then moves.
        if let Some(hit) = overlapping(&self.board, self.spin.var, &b) {
            let mass = self.board.keys().len().max(1);
            self.spin
                .add_torque(calc_torque(b.x, b.y, b.dx, b.dy, mass));

            match get_dest_cell(hit, b.x, b.y, self.spin.var) {
                Some(dest) => self.place_bubble(dest, b.col),
                // The cart would do `board[nil] = col` and error. The shipped
                // level never asks for a cell outside the grid, but a rotated
                // edge cell can: the shot is lost rather than the game crashing.
                None => {
                    if !removed {
                        self.give_up(i, &mut b, false);
                        removed = true;
                    }
                }
            }

            if !removed {
                self.bubs.remove(i);
            }
            if self.player_turn {
                self.reloading = true;
            } else if self.board_shots > 0 {
                self.in_flight = false;
                self.board_shots -= 1;
            } else {
                self.reloading = true;
            }
            return true;
        }

        if !removed {
            b.x = nx;
            b.y = ny;
            // Write the whole bubble back: a wall bounce flipped `dx`/`dy` above,
            // and the cart's `_b` is a reference, so its flips stick.
            self.bubs[i] = b;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::ANCHOR;
    use crate::fixed::from_f64;
    use crate::grid::CENTER;
    use crate::sim::gun::A_MIN;
    use crate::sim::Input;

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
    fn firing_straight_up_gives_no_x_speed_and_minus_two_and_a_half_y() {
        let mut g = Game::new(7);
        g.fire();
        let b = g.bubs[0];
        assert!(b.fired, "the bubble is in flight");
        assert_eq!(b.dx, 0);
        assert_eq!(b.dy, -163_840); // -2.5 in 16.16
    }

    #[test]
    fn firing_left_gives_minus_two_and_a_half_x() {
        let mut g = Game::new(7);
        g.gun.a = A_MIN; // 0.25 turns, straight left
        g.fire();
        let b = g.bubs[0];
        assert_eq!(b.dx, -163_840);
        assert_eq!(b.dy, 0);
    }

    #[test]
    fn overlapping_misses_at_seven_pixels_and_hits_at_six() {
        // Cell 262 sits at (64,64). The cart's test is `< 2*b_r-1`, i.e. < 7 px.
        let board = board_from(&[(CENTER, ANCHOR)]);
        let bub = |x: Fix, y: Fix| Bubble {
            x,
            y,
            ..Bubble::at_gun()
        };
        assert_eq!(overlapping(&board, 0, &bub(int(71), int(64))), None);
        assert_eq!(overlapping(&board, 0, &bub(int(70), int(64))), Some(CENTER));
        // The bubble's position is floored first, so 70.9 is a hit and 71.0 is not.
        assert_eq!(
            overlapping(&board, 0, &bub(from_f64(70.9), int(64))),
            Some(CENTER)
        );
        assert_eq!(overlapping(&board, 0, &bub(from_f64(71.0), int(64))), None);
    }

    #[test]
    fn a_spinning_mass_moves_the_cell_a_bubble_can_hit() {
        // 263 sits right of the centre at (72,64) and turns to (64,56) at a
        // quarter turn. A bubble parked at (72,64) hits it at rest and misses it
        // once the mass has turned.
        let board = board_from(&[(263, 2)]);
        let parked = Bubble {
            x: int(72),
            y: int(64),
            ..Bubble::at_gun()
        };
        assert_eq!(overlapping(&board, 0, &parked), Some(263));
        assert_eq!(overlapping(&board, from_f64(0.25), &parked), None);
    }

    #[test]
    fn the_destination_is_the_neighbours_cell_in_the_sector_the_bubble_came_from() {
        // All six sectors of cell 262, with the bubble 8 px away. Values computed
        // independently in Python from the cart's formula
        // `flr((atan2(x-hx,y-hy)+.0833)*6)+1` with 16.16 quantisation.
        let dest = |dx: f64, dy: f64| {
            let (x, y) = (int(64) + from_f64(dx), int(64) + from_f64(dy));
            get_dest_cell(CENTER, x, y, 0)
        };
        assert_eq!(dest(8.0, 0.0), Some(263)); // e
        assert_eq!(dest(0.0, -8.0), Some(241)); // ne
        assert_eq!(dest(-4.0, -6.9282), Some(240)); // nw
        assert_eq!(dest(-8.0, 0.0), Some(261)); // w
        assert_eq!(dest(-4.0, 6.9282), Some(282)); // sw
        assert_eq!(dest(4.0, 6.9282), Some(283)); // se
    }

    #[test]
    fn a_matching_placement_explodes_the_whole_run() {
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (240, 3), (241, 3)]);
        g.place_bubble(242, 3);
        assert_eq!(g.board.get(240), Some(303));
        assert_eq!(g.board.get(241), Some(303));
        assert_eq!(g.board.get(242), Some(303));
    }

    #[test]
    fn a_matching_placement_drops_anything_already_hanging() {
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (240, 3), (241, 3), (100, 4), (101, 4)]);
        g.place_bubble(242, 3);
        assert_eq!(g.board.get(100), Some(14));
        assert_eq!(g.board.get(101), Some(14));
    }

    #[test]
    fn a_placement_that_matches_nothing_costs_a_try() {
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (240, 3)]);
        let before = g.shot_cnt;
        g.place_bubble(500, 2);
        assert_eq!(g.board.get(500), Some(2));
        assert_eq!(g.shot_cnt, before - 1);
    }

    #[test]
    fn four_placements_that_match_nothing_end_the_players_turn() {
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR)]);
        // Four cells far apart, so no placement can match anything.
        for i in [40, 100, 160, 220] {
            g.place_bubble(i, 2);
        }
        assert!(!g.player_turn);
        assert_eq!(g.board_shots, 1);
        assert_eq!(g.shot_count, 2);
        assert_eq!(g.shot_cnt, SHOT_MAX);
    }

    #[test]
    fn reset_bubble_hands_over_the_conveyor() {
        let mut g = Game::new(7);
        g.nxt_col = 5;
        g.nxt_nxt_col = 6;
        g.bounces = 3;
        g.in_flight = true;
        g.reset_bubble();
        let b = *g.bubs.last().unwrap();
        assert_eq!(b.col, 5);
        assert_eq!((b.x, b.y), (int(64), int(121)));
        assert!(!b.fired);
        assert_eq!(g.nxt_col, 6);
        assert!(g.col_pool.contains(&g.nxt_nxt_col));
        assert_eq!(g.bounces, 0);
        assert_eq!(g.hud_t, 180);
        assert!(g.player_turn);
        assert!(!g.in_flight);
        assert!(!g.reloading);
    }

    #[test]
    fn a_straight_shot_sticks_to_the_wall_row_and_costs_a_try() {
        // A bare board: one gumball at cell 346, which sits at (64,92) — the
        // bottom wall row. Fired straight up from (64,121) the bubble reaches
        // y = 98.5 on frame 10, sticks one cell south-west of 346 (cart sector
        // 5, i.e. cell 366) and, matching nothing, spends a try.
        let mut g = Game::new(7);
        g.board = board_from(&[(CENTER, ANCHOR), (346, 7)]);
        // A still mass, so the shot lands where the geometry says. The spin
        // moving the target is `a_spinning_mass_moves_the_cell_a_bubble_can_hit`.
        g.spin.var = 0;
        g.spin.torque = 0;
        let col = g.bubs[0].col;
        g.step(&fired());
        for _ in 0..20 {
            g.step(&still());
        }
        assert_eq!(g.board.0.len(), 3, "one gumball was added");
        assert_eq!(g.board.get(366), Some(col), "cell 366");
        assert_eq!(g.shot_cnt, 3, "a shot that clears nothing costs a try");
        // The shot is perfectly centred, so the lever arm is zero: a cart
        // signature we must not "fix". `an_off_centre_hit_spins_the_mass` is
        // the other half.
        assert_eq!(g.spin.torque, 0);
        assert!(g.bubs.iter().all(|b| !b.fired));
        // The conveyor dealt the next bubble on the following frame.
        assert_eq!(g.bubs.len(), 1);
        assert!(!g.reloading);
    }

    #[test]
    fn an_off_centre_hit_spins_the_mass() {
        let mut g = Game::new(7);
        g.spin.var = 0;
        g.spin.torque = 0;
        g.gun.a = from_f64(0.45); // a little to the left of straight up
        g.step(&fired());
        let mut spun = false;
        for _ in 0..40 {
            g.step(&still());
            spun |= g.spin.torque != 0;
        }
        assert!(spun, "an off-centre hit twists the mass");
    }

    #[test]
    fn a_shot_at_the_real_level_always_resolves_without_panicking() {
        // Seed-independent property: the shot resolves, at most one gumball is
        // added, and the pop animation finishes and leaves settled cells behind.
        for seed in 1..=12 {
            let mut g = Game::new(seed);
            let before = g.board.0.len();
            g.step(&fired());
            for _ in 0..30 {
                g.step(&still());
            }
            assert!(
                g.bubs.iter().all(|b| !b.fired),
                "seed {seed}: the shot never resolved"
            );
            assert!(
                g.board.0.len() <= before + 1,
                "seed {seed}: a shot added more than one gumball"
            );
            for _ in 0..40 {
                g.step(&still());
            }
            assert!(!g.popping, "seed {seed}: still animating");
            assert!(
                g.board.0.values().all(|&v| v < 10),
                "seed {seed}: an animating value survived"
            );
        }
    }

    #[test]
    fn five_wall_bounces_give_up_the_shot() {
        let mut g = Game::new(7);
        g.gun.a = A_MIN; // straight left, so it never rises into the mass
        g.step(&fired());
        let mut frames = 0;
        while g.f_bubs.is_empty() && frames < 400 {
            g.step(&still());
            frames += 1;
        }
        assert!(frames < 400, "the shot never gave up");
        // The arena is 116 px wide and the bubble moves 2.5 px a frame, so a
        // bounce costs ~46 frames. Five bounces therefore take about 200 — this
        // bound is what caught a bubble that stuck to the wall and "bounced"
        // once a frame instead of crossing the arena.
        assert!(
            frames > 180,
            "the bubble crossed the arena: {frames} frames"
        );
        assert_eq!(g.bounces, 5, "the cart gives up on the fifth bounce");
        assert_eq!(g.f_bubs.len(), 1);
        assert!(g.f_bubs[0].falling);
        assert!(!g.f_bubs[0].fired);
        assert_eq!(g.shot_cnt, 3);

        // The next frame the conveyor hands over a new bubble and clears bounces.
        g.step(&still());
        assert_eq!(g.bounces, 0);
        assert_eq!(g.bubs.len(), 1);
        assert!(!g.bubs[0].fired);
    }

    #[test]
    fn a_falling_bubble_leaves_the_screen() {
        let mut g = Game::new(7);
        g.f_bubs.push(Bubble {
            x: int(64),
            y: int(130),
            dy: from_f64(2.0),
            falling: true,
            ..Bubble::at_gun()
        });
        for _ in 0..20 {
            g.step(&still());
        }
        assert!(g.f_bubs.is_empty(), "past y = 136 it is deleted");
    }

    #[test]
    fn firing_only_happens_on_a_fresh_press() {
        let mut g = Game::new(7);
        g.step(&fired());
        g.step(&fired());
        g.step(&fired());
        assert_eq!(g.bubs.iter().filter(|b| b.fired).count(), 1);
    }
}
