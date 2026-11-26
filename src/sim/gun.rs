//! The gun: the aimer at the bottom of the screen and the shot direction.
//!
//! The cart keeps `gun = {a=8/16, x=64, y=121}` and moves `a` by 1/256 per frame
//! left or right, within `mid(8/32, a, 24/32)`.

use crate::fixed::Fix;

/// The gun's starting angle, 8/16 turns: straight up.
pub const A_START: Fix = 32_768;
/// The clamp's lower bound, 8/32 turns (aiming up-left).
pub const A_MIN: Fix = 16_384;
/// The clamp's upper bound, 24/32 turns (aiming up-right).
pub const A_MAX: Fix = 49_152;
/// One frame of aiming, 1/256 turns.
pub const A_STEP: Fix = 256;
/// How far the aimer sits from the gun, in pixels.
pub const AIM_LEN: Fix = 20 << 16;

/// cart `gun`: angle plus the pivot the aimer hangs from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gun {
    /// Aiming angle in turns; 0.5 is straight up.
    pub a: Fix,
    pub x: Fix,
    pub y: Fix,
}

impl Default for Gun {
    fn default() -> Self {
        Self::new()
    }
}

impl Gun {
    /// The cart's `gun={a=8/16, x=64, y=121}`.
    pub fn new() -> Self {
        Self {
            a: A_START,
            x: 64 << 16,
            y: 121 << 16,
        }
    }

    /// One frame of the cart's aiming input.
    ///
    /// The cart applies the clamps *after* both buttons, so holding left and
    /// right together moves the angle by 1/256 twice and cancels out.
    pub fn update(&mut self, left: bool, right: bool, up: bool) {
        if left {
            self.a = self.a.wrapping_sub(A_STEP);
        }
        if right {
            self.a = self.a.wrapping_add(A_STEP);
        }
        if up {
            self.a = A_START;
        }
        self.a = self.a.max(A_MIN).min(A_MAX);
    }

    /// Where the aimer is drawn: `(sin(a), cos(a)) * 20 + (x, y)`.
    pub fn aim(&self) -> (Fix, Fix) {
        (
            crate::fixed::mul(crate::p8math::sin(self.a), AIM_LEN) + self.x,
            crate::fixed::mul(crate::p8math::cos(self.a), AIM_LEN) + self.y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::{from_f64, to_int};

    #[test]
    fn the_gun_starts_pointing_straight_up() {
        let g = Gun::new();
        assert_eq!(g.a, from_f64(0.5));
        assert_eq!((to_int(g.x), to_int(g.y)), (64, 121));
        // a = 0.5 is up: sin(0.5) = 0, cos(0.5) = -1.
        let (ax, ay) = g.aim();
        assert_eq!((to_int(ax), to_int(ay)), (64, 101));
    }

    #[test]
    fn left_moves_the_aimer_left() {
        let mut g = Gun::new();
        for _ in 0..30 {
            g.update(true, false, false);
        }
        let (ax, _) = g.aim();
        assert!(ax < from_f64(64.0), "aim x = {ax}");
        assert_eq!(to_int(ax), 50);
    }

    #[test]
    fn right_moves_the_aimer_right() {
        let mut g = Gun::new();
        for _ in 0..30 {
            g.update(false, true, false);
        }
        let (ax, _) = g.aim();
        assert!(ax > from_f64(64.0), "aim x = {ax}");
        // 77, not the mirror of 50: 16.16 quantisation is not symmetric about
        // a = 0.5. Measured, not assumed.
        assert_eq!(to_int(ax), 77);
    }

    #[test]
    fn the_angle_is_clamped_to_eight_to_twentyfour_thirtyseconds() {
        let mut g = Gun::new();
        for _ in 0..400 {
            g.update(true, false, false);
        }
        assert_eq!(g.a, from_f64(0.25));
        let (ax, ay) = g.aim();
        assert_eq!((to_int(ax), to_int(ay)), (44, 121)); // sin(0.25) = -1, cos = 0

        for _ in 0..800 {
            g.update(false, true, false);
        }
        assert_eq!(g.a, from_f64(0.75));
        let (ax, ay) = g.aim();
        assert_eq!((to_int(ax), to_int(ay)), (84, 121));
    }

    #[test]
    fn up_snaps_back_to_vertical() {
        let mut g = Gun::new();
        for _ in 0..40 {
            g.update(true, false, false);
        }
        assert_ne!(g.a, from_f64(0.5));
        g.update(false, false, true);
        assert_eq!(g.a, from_f64(0.5));
    }

    #[test]
    fn holding_both_directions_cancels_out() {
        let mut g = Gun::new();
        g.update(true, true, false);
        assert_eq!(g.a, from_f64(0.5));
    }
}
