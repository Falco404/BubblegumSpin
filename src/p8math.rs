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

use crate::fixed::{from_f64, int, mul, to_f64, Fix};

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

/// A seeded xorshift32 generator.
///
/// **Not** PICO-8's PRNG: PICO-8's generator is undocumented and the cart's
/// `rnd()` sequence cannot be reproduced. This one is written down, seeded per
/// test, and therefore deterministic — see `docs/decisions.md`. The visible
/// consequence is that gumball colours do not come out in the cart's order.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u32,
}

impl Rng {
    /// A generator with a fixed seed. Seed 0 would lock xorshift32 at zero, so
    /// it is replaced by a constant.
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9 } else { seed },
        }
    }

    /// The next raw 32-bit value (xorshift32).
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// PICO-8 `rnd()`: a fixed-point value in `[0, 1)`.
    pub fn rnd(&mut self) -> Fix {
        (self.next_u32() >> 16) as Fix
    }

    /// PICO-8 `rnd(n)`: a fixed-point value in `[0, n)`.
    pub fn rnd_fix(&mut self, n: Fix) -> Fix {
        mul(self.rnd(), n)
    }

    /// `flr(rnd(col_cnt) + 1)`: the cart's way of picking a gumball colour,
    /// so the result is `1..=col_cnt`.
    pub fn colour(&mut self, col_cnt: i32) -> u8 {
        (crate::fixed::to_int(mul(self.rnd(), int(col_cnt))) + 1) as u8
    }
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

    #[test]
    fn rng_with_the_same_seed_gives_the_same_sequence() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..16 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn rng_matches_the_xorshift32_reference_values() {
        // Computed independently: x ^= x<<13; x ^= x>>17; x ^= x<<5 (u32 wrap).
        let mut r = Rng::new(1);
        let got: Vec<u32> = (0..3).map(|_| r.next_u32()).collect();
        assert_eq!(got, vec![270_369, 67_634_689, 2_647_435_461]);
    }

    #[test]
    fn rng_seed_zero_still_produces_a_sequence() {
        // xorshift32 is stuck at 0, so the seed is forced off zero.
        let mut r = Rng::new(0);
        let first = r.next_u32();
        assert_ne!(first, 0);
        assert_ne!(first, r.next_u32());
    }

    #[test]
    fn rnd_is_in_the_half_open_unit_interval() {
        let mut r = Rng::new(3);
        for _ in 0..1000 {
            let v = r.rnd();
            assert!((0..ONE).contains(&v), "rnd() out of range: {v}");
        }
    }

    #[test]
    fn colour_is_one_based_and_within_range() {
        let mut r = Rng::new(5);
        for _ in 0..1000 {
            let c = r.colour(2);
            assert!((1..=2).contains(&c), "colour out of range: {c}");
        }
    }

    #[test]
    fn colour_uses_every_value_of_the_range() {
        let mut r = Rng::new(11);
        let mut seen = [0usize; 7];
        for _ in 0..600 {
            seen[r.colour(6) as usize] += 1;
        }
        assert_eq!(seen[0], 0, "colour 0 is not a real colour");
        for (colour, &n) in seen.iter().enumerate().skip(1) {
            assert!(n > 50, "colour {colour} came up {n} times");
        }
    }
}
