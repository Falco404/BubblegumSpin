//! Rotation of the gumball mass around the screen centre.
//!
//! The grid never rotates: only the *screen positions* do. Each frame the cart
//! integrates `var` (rotational velocity) and maps every cell's grid position
//! through `px2rpx`, and maps a flying bubble back through `rpx2px` when it
//! needs a cell. So the spin changes where a bubble actually sticks.

use crate::fixed::{int, mul, Fix};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::{from_f64, to_int};

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
}
