//! PICO-8-compatible 16.16 fixed point numbers.
//!
//! A `Fix` is an `i32` holding a signed 16.16 value: 16 integer bits, 16
//! fractional bits. `1.0 == 65536`, the smallest representable step is
//! `1/65536`. Everything in the cart is one of these, so the port uses the same
//! type everywhere and only leaves it inside `p8math`'s trig.

pub type Fix = i32;

pub const ONE: Fix = 1 << 16; // 1.0
pub const HALF: Fix = 1 << 15; // 0.5
pub const EPS: Fix = 1; // 1/65536, the smallest step PICO-8 has

/// PICO-8 integer -> fixed.
#[inline]
pub const fn int(i: i32) -> Fix {
    i.wrapping_shl(16)
}

/// Fixed -> integer, rounding down (same as `flr` then dropping the fraction).
#[inline]
pub const fn to_int(f: Fix) -> i32 {
    f >> 16
}

/// Fixed -> fixed, fractional part cleared (PICO-8 `flr`).
#[inline]
pub const fn flr(f: Fix) -> Fix {
    f & !0xffff
}

/// Fixed multiply. The low 16 bits are dropped, i.e. it floors (PICO-8 behaviour).
#[inline]
pub fn mul(a: Fix, b: Fix) -> Fix {
    ((a as i64 * b as i64) >> 16) as Fix
}

/// Fixed divide. Truncates toward zero.
#[inline]
pub fn div(a: Fix, b: Fix) -> Fix {
    (((a as i64) << 16) / b as i64) as Fix
}

/// Quantise an `f64` to the nearest-below fixed value (truncates toward zero).
///
/// Only for tests, literals and table building — never for gameplay maths.
#[inline]
pub fn from_f64(v: f64) -> Fix {
    (v * 65536.0) as Fix
}

/// Convert a fixed value to `f64`. Only for tests, printing and audio.
#[inline]
pub fn to_f64(f: Fix) -> f64 {
    f as f64 / 65536.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_is_65536() {
        assert_eq!(ONE, 65_536);
    }

    #[test]
    fn int_to_fixed_and_back() {
        assert_eq!(int(2), 131_072);
        assert_eq!(to_int(int(2)), 2);
        assert_eq!(to_int(from_f64(2.5)), 2); // floors
        assert_eq!(to_int(from_f64(-2.5)), -3); // floors, not truncates
    }

    #[test]
    fn mul_matches_hand_math() {
        assert_eq!(mul(from_f64(1.5), from_f64(0.5)), from_f64(0.75));
        assert_eq!(mul(from_f64(2.5), int(2)), int(5));
        assert_eq!(mul(from_f64(-0.5), from_f64(0.5)), from_f64(-0.25));
    }

    #[test]
    fn div_matches_hand_math() {
        assert_eq!(div(ONE, int(2)), HALF);
        // 7/70 = 0.1 -> quantised to 6553 (truncated). Check with:
        //   python3 -c "print((7*65536)//70)"  -> 6553
        assert_eq!(div(int(7), int(70)), 6_553);
    }

    #[test]
    fn flr_clears_fraction() {
        assert_eq!(flr(from_f64(3.75)), int(3));
        assert_eq!(flr(from_f64(-0.25)), int(-1));
    }
}
