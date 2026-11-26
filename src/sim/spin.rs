//! Rotation of the gumball mass around the screen centre.
//!
//! The grid never rotates: only the *screen positions* do. Each frame the cart
//! integrates `var` (rotational velocity) and maps every cell's grid position
//! through `px2rpx`, and maps a flying bubble back through `rpx2px` when it
//! needs a cell. So the spin changes where a bubble actually sticks.

use crate::fixed::{div, int, mul, Fix};
use crate::p8math::{atan2, cos, sin, sqrt};

/// The screen centre the mass rotates around: `(64, 64)`.
pub const C: Fix = int(64);

/// cart `px2rpx(x, y)`: grid-space point -> rotated screen point.
pub fn px2rpx(x: Fix, y: Fix, var: Fix) -> (Fix, Fix) {
    let dx = x - C;
    let dy = y - C;
    let r = sqrt(mul(dx, dx) + mul(dy, dy));
    let ang = atan2(dx, dy) + var;
    (mul(cos(ang), r) + C, mul(sin(ang), r) + C)
}

/// cart `rpx2px(x, y)`: rotated screen point -> grid-space point (the inverse).
pub fn rpx2px(x: Fix, y: Fix, var: Fix) -> (Fix, Fix) {
    let dx = x - C;
    let dy = y - C;
    let r = sqrt(mul(dx, dx) + mul(dy, dy));
    let ang = atan2(dx, dy) - var;
    (mul(cos(ang), r) + C, mul(sin(ang), r) + C)
}

/// cart `update_spinner()`: `var += torque`, then decay.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Spin {
    /// Accumulated rotation, in turns (the cart's `var`).
    pub var: Fix,
    /// Rotational velocity (the cart's `torque`), decays every frame.
    pub torque: Fix,
}

/// The cart's `0.0004` threshold, quantised to 16.16: 26/65536 = 0.0003967.
pub const TORQUE_MIN: Fix = 26;
/// The cart's `0.96` decay, quantised to 16.16.
pub const TORQUE_DECAY: Fix = 62_914;

impl Spin {
    pub fn new() -> Self {
        Self::default()
    }

    /// One frame of `update_spinner()`.
    pub fn step(&mut self) {
        self.var = self.var.wrapping_add(self.torque);
        if self.torque.abs() > TORQUE_MIN {
            self.torque = mul(self.torque, TORQUE_DECAY);
        } else {
            self.torque = 0;
        }
    }

    /// Add a kick to the rotational velocity (the cart does `var += torque` only
    /// after `calc_torque` has replaced `torque`; see M4).
    pub fn add_torque(&mut self, torque: Fix) {
        self.torque = self.torque.wrapping_add(torque);
    }
}

/// cart `calc_torque(_b)`: how hard a bubble hitting at `(x, y)` moving
/// `(dx, dy)` twists the mass.
///
/// The lever arm is the angle from the bubble to the centre, the force is the
/// angle of travel, and the kick is `r * sin(a - b) / 70`, divided by the number
/// of gumballs so a lighter board spins harder. That last division is the game's
/// signature feel — players complained about "spinning wildly" and it is
/// intended.
pub fn calc_torque(x: Fix, y: Fix, dx: Fix, dy: Fix, mass: usize) -> Fix {
    // Angle of the lever arm: from the bubble to the centre.
    let a = atan2(C - x, C - y);
    // Angle of the force: the direction the bubble is travelling.
    let b = atan2(x - (x + dx), y - (y + dy));
    let theta = a.wrapping_sub(b);
    let r = sqrt(mul(C - x, C - x) + mul(C - y, C - y));
    let tau = div(mul(r, sin(theta)), int(70));
    div(tau, int(mass as i32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::{from_f64, to_int};

    #[test]
    fn a_shot_through_the_centre_gives_no_torque() {
        // Bubble 24 px above the centre, moving straight down: the lever arm and
        // the force point the same way, so sin(theta) = 0.
        let tau = calc_torque(from_f64(64.0), from_f64(40.0), 0, from_f64(2.5), 60);
        assert!(tau.abs() <= 64, "tau = {tau}");
    }

    #[test]
    fn a_shot_past_the_centre_spins_the_mass() {
        // 16 px to the right, still moving down: an off-centre hit.
        let tau = calc_torque(from_f64(80.0), from_f64(40.0), 0, from_f64(2.5), 60);
        assert!(tau.abs() > 64, "tau = {tau}");
    }

    #[test]
    fn more_mass_means_less_spin() {
        let light = calc_torque(from_f64(80.0), from_f64(40.0), 0, from_f64(2.5), 30);
        let heavy = calc_torque(from_f64(80.0), from_f64(40.0), 0, from_f64(2.5), 60);
        assert_eq!(heavy, -249); // pinned: 16.16, i.e. -0.0038 turns per frame
        // Fixed-point division truncates, so this is twice within one LSB.
        assert!((light - heavy * 2).abs() <= 1, "{light} vs {heavy}");
    }

    #[test]
    fn the_two_sides_spin_in_opposite_directions() {
        let right = calc_torque(from_f64(80.0), from_f64(40.0), 0, from_f64(2.5), 60);
        let left = calc_torque(from_f64(48.0), from_f64(40.0), 0, from_f64(2.5), 60);
        assert_ne!(right.signum(), left.signum());
    }

    /// PICO-8's angles are 0..1 turns and y is inverted, so a quarter turn
    /// rotates a point to the right of the centre to above it: (72,64) -> (64,56).
    #[test]
    fn px2rpx_identity_at_zero() {
        let points = [
            (64, 64),
            (72, 64),
            (64, 72),
            (56, 64),
            (64, 56),
            (72, 72),
            (56, 56),
            (72, 56),
            (56, 72),
            (0, 0),
            (127, 127),
            (0, 127),
            (127, 0),
            (20, 100),
            (100, 20),
            (40, 40),
            (90, 90),
            (64, 100),
            (100, 64),
            (33, 77),
        ];
        for (x, y) in points {
            let (rx, ry) = px2rpx(from_f64(x as f64), from_f64(y as f64), 0);
            // 16.16 sqrt/sin/cos quantise, so this is not bit-exact: the worst
            // error measured over these 20 points is ~324/65536 px (~0.005 px),
            // at the screen edge where the radius is largest.
            assert!(
                (rx - from_f64(x as f64)).abs() <= 1024 && (ry - from_f64(y as f64)).abs() <= 1024,
                "({x},{y}) -> ({rx},{ry}) is not the identity"
            );
        }
    }

    #[test]
    fn quarter_turn_rotates_right_to_up() {
        let var = from_f64(0.25);
        let (rx, ry) = px2rpx(from_f64(72.0), from_f64(64.0), var);
        assert_eq!((rx, ry), (4_194_304, 3_670_016)); // (64.0, 56.0) in 16.16
        assert_eq!((to_int(rx), to_int(ry)), (64, 56));
    }

    #[test]
    fn a_full_turn_returns_to_the_start() {
        let p = (from_f64(80.0), from_f64(40.0));
        let (rx, ry) = px2rpx(p.0, p.1, from_f64(1.0));
        // Measured: 120/65536 px (~0.002 px) of drift over a full turn.
        assert!(
            (rx - p.0).abs() <= 1024 && (ry - p.1).abs() <= 1024,
            "{rx},{ry}"
        );
    }

    #[test]
    fn rpx2px_undoes_px2rpx() {
        let var = from_f64(0.125);
        let (x, y) = (from_f64(90.0), from_f64(30.0));
        let (rx, ry) = px2rpx(x, y, var);
        let (bx, by) = rpx2px(rx, ry, var);
        assert!(
            (bx - x).abs() <= 1024 && (by - y).abs() <= 1024,
            "{bx},{by} vs {x},{y}"
        );
    }

    #[test]
    fn the_centre_never_moves() {
        for t in [0.0, 0.1, 0.37, 0.5, 0.75] {
            let (rx, ry) = px2rpx(from_f64(64.0), from_f64(64.0), from_f64(t));
            assert_eq!((to_int(rx), to_int(ry)), (64, 64));
        }
    }

    #[test]
    fn one_step_adds_the_torque_to_the_angle() {
        let mut spin = Spin::new();
        spin.torque = from_f64(0.05); // 3276 in 16.16
        spin.step();
        assert_eq!(spin.var, 3_276);
        // 0.05 * 0.96 = 0.048 -> 3144 (quantised)
        assert_eq!(spin.torque, 3_144);
    }

    #[test]
    fn decay_stops_and_snaps_to_zero() {
        // The cart keeps decaying while |torque| > 0.0004 (26 in 16.16) and then
        // sets it to exactly 0. Hand-computed with the same fixed-point rules:
        // zero is reached on frame 110.
        let mut spin = Spin::new();
        spin.torque = from_f64(0.05);
        let mut frames = 0;
        while spin.torque != 0 && frames < 500 {
            spin.step();
            frames += 1;
        }
        assert_eq!(frames, 110);
        assert_eq!(spin.var, 79_886); // 1.21896 turns
    }

    #[test]
    fn sixty_frames_of_decay_have_not_stopped_yet() {
        let mut spin = Spin::new();
        spin.torque = from_f64(0.05);
        for _ in 0..60 {
            spin.step();
        }
        // The plan guessed "60 steps ends at zero"; it does not. Measured:
        assert_eq!((spin.torque, spin.var), (271, 74_338));
    }

    #[test]
    fn the_angle_wraps_and_never_panics() {
        let mut spin = Spin::new();
        spin.var = Fix::MAX;
        spin.torque = Fix::MAX;
        spin.step(); // must not panic in debug builds
        assert!(spin.torque != Fix::MAX);
    }
}
