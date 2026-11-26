//! PICO-8 maths: 0..1 turns, **y inverted** (screenspace), `atan2` returns 0..1.
//!
//! PICO-8 defines its trig for a screen where y grows downwards, so angles run
//! anticlockwise on screen and the sine is the negative of the textbook one:
//!
//! * `sin(a) = -sin(TAU*a)` — the manual: "PICO-8's SIN() returns an inverted
//!   result to suit screenspace", `sin(0.25) == -1`.
//! * `cos(a) = cos(TAU*a)` — not inverted.
//! * `atan2(dx,dy) = atan2_std(-dy, dx) / TAU`, wrapped to 0..1 — so `0` is
//!   +x, `0.25` is **up**, `0.5` is -x and `0.75` is **down**.
//!
//! Getting this wrong yields `nil` cells from sector lookups and a mirrored
//! aim; see `docs/decisions.md`.

use crate::fixed::{from_f64, to_f64, Fix};

/// PICO-8 `sin`: 0..1 turns, y inverted to suit screenspace, so `sin(0.25) == -1`.
#[inline]
pub fn sin(a: Fix) -> Fix {
    let t = to_f64(a).rem_euclid(1.0);
    from_f64(-(t * std::f64::consts::TAU).sin())
}

/// PICO-8 `cos`: 0..1 turns, **not** inverted (`cos(0.25) == 0`, `cos(0.5) == -1`).
#[inline]
pub fn cos(a: Fix) -> Fix {
    let t = to_f64(a).rem_euclid(1.0);
    from_f64((t * std::f64::consts::TAU).cos())
}

/// PICO-8 `atan2(dx, dy)`: 0 = +x, 0.25 = up (-y), 0.5 = -x, 0.75 = down (+y),
/// wrapped to 0..1 turns and running anticlockwise in screenspace.
///
/// NOTE the argument order is `(dx, dy)`, not Rust's `atan2(y, x)`.
#[inline]
pub fn atan2(dx: Fix, dy: Fix) -> Fix {
    let a = (-to_f64(dy)).atan2(to_f64(dx)) / std::f64::consts::TAU;
    from_f64(a.rem_euclid(1.0))
}

/// PICO-8 `sqrt`. Negative input clamps to 0 (PICO-8 returns 0, not NaN).
#[inline]
pub fn sqrt(a: Fix) -> Fix {
    from_f64(to_f64(a).max(0.0).sqrt())
}

/// PICO-8 `abs`.
#[inline]
pub fn abs(a: Fix) -> Fix {
    a.abs()
}

/// PICO-8 `ceil`: round towards +infinity.
#[inline]
pub fn ceil(a: Fix) -> Fix {
    (a.wrapping_add(0xffff)) & !0xffff
}

/// PICO-8 `mid(lo, v, hi)`: clamp `v` into `[lo, hi]`.
#[inline]
pub fn mid(lo: Fix, v: Fix, hi: Fix) -> Fix {
    v.max(lo).min(hi)
}

/// PICO-8 `band` on plain integers.
#[inline]
pub fn band(a: i32, b: i32) -> i32 {
    a & b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::{int, ONE};

    #[test]
    fn sin_zero_is_zero() {
        assert_eq!(sin(0), 0);
    }

    #[test]
    fn sin_quarter_turn_is_minus_one() {
        assert_eq!(sin(from_f64(0.25)), -ONE);
    }

    #[test]
    fn sin_three_quarter_turn_is_one() {
        assert_eq!(sin(from_f64(0.75)), ONE);
    }

    #[test]
    fn cos_zero_is_one() {
        assert_eq!(cos(0), ONE);
    }

    #[test]
    fn cos_quarter_turn_is_zero() {
        assert_eq!(cos(from_f64(0.25)), 0);
    }

    #[test]
    fn cos_half_turn_is_minus_one() {
        assert_eq!(cos(from_f64(0.5)), -ONE);
    }

    #[test]
    fn atan2_matches_pico8_convention() {
        // raw fixed values: 0.00 / 0.25 / 0.50 / 0.75 turns = 0 / 16384 / 32768 / 49152
        assert_eq!(atan2(int(1), 0), 0); // +x   -> 0.00 turns
        assert_eq!(atan2(0, -int(1)), 16_384); // up   -> 0.25 turns
        assert_eq!(atan2(-int(1), 0), 32_768); // -x   -> 0.50 turns
        assert_eq!(atan2(0, int(1)), 49_152); // down -> 0.75 turns
    }

    #[test]
    fn sqrt_rounds_down_and_clamps_negatives() {
        assert_eq!(sqrt(int(4)), int(2));
        // sqrt(2) = 1.41421... -> 0x1.6a09 = 92681 (truncated)
        assert_eq!(sqrt(int(2)), 92_681);
        assert_eq!(sqrt(from_f64(-1.0)), 0);
    }

    #[test]
    fn ceil_rounds_towards_positive_infinity() {
        assert_eq!(ceil(from_f64(1.25)), int(2));
        assert_eq!(ceil(int(3)), int(3));
        assert_eq!(ceil(from_f64(-1.25)), int(-1));
        assert_eq!(ceil(from_f64(-1.0)), int(-1));
    }

    #[test]
    fn mid_clamps_between_the_bounds() {
        assert_eq!(mid(int(1), int(5), int(4)), int(4));
        assert_eq!(mid(int(1), int(-5), int(4)), int(1));
        assert_eq!(mid(int(1), int(2), int(4)), int(2));
    }

    #[test]
    fn abs_is_signed_magnitude() {
        assert_eq!(abs(from_f64(-0.5)), from_f64(0.5));
        assert_eq!(abs(int(-3)), int(3));
    }
}
