//! The 16 PICO-8 colours and the `pal()` / `palt()` display state.

/// The PICO-8 system palette, indices 0..15, as RGB.
///
/// Values are the published PICO-8 palette (PICO-8 wiki / lospec agree); PICO-8
/// itself has no hex literals, so this table is the only place they live.
pub const PALETTE: [[u8; 3]; 16] = [
    [0x00, 0x00, 0x00], // 0  black
    [0x1D, 0x2B, 0x53], // 1  dark blue
    [0x7E, 0x25, 0x53], // 2  dark purple
    [0x00, 0x87, 0x51], // 3  dark green
    [0xAB, 0x52, 0x36], // 4  brown
    [0x5F, 0x57, 0x4F], // 5  dark grey
    [0xC2, 0xC3, 0xC7], // 6  light grey
    [0xFF, 0xF1, 0xE8], // 7  white
    [0xFF, 0x00, 0x4D], // 8  red
    [0xFF, 0xA3, 0x00], // 9  orange
    [0xFF, 0xEC, 0x27], // 10 yellow
    [0x00, 0xE4, 0x36], // 11 green
    [0x29, 0xAD, 0xFF], // 12 blue
    [0x83, 0x76, 0x9C], // 13 lavender
    [0xFF, 0x77, 0xA8], // 14 pink
    [0xFF, 0xCC, 0xAA], // 15 light peach
];

/// Display palette state: `pal()` remapping plus `palt()` transparency.
///
/// Kept as plain data, so a scene can be drawn and diffed without a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pal {
    /// Draw colour `i` is shown as colour `map[i]`.
    pub map: [u8; 16],
    /// `transparent[i]` = colour `i` is not drawn (PICO-8 `palt`).
    pub transparent: [bool; 16],
}

impl Default for Pal {
    fn default() -> Self {
        Self::new()
    }
}

impl Pal {
    /// The identity mapping with nothing transparent.
    pub fn new() -> Self {
        Self {
            map: std::array::from_fn(|i| i as u8),
            transparent: [false; 16],
        }
    }

    /// PICO-8 `pal(c0, c1)`: draw colour `c0` as colour `c1`.
    pub fn pal(&mut self, c0: u8, c1: u8) {
        self.map[(c0 & 15) as usize] = c1 & 15;
    }

    /// PICO-8 `pal(c0)`: undo the remapping of one colour.
    pub fn pal_reset(&mut self, c0: u8) {
        self.map[(c0 & 15) as usize] = c0 & 15;
    }

    /// PICO-8 `pal()`: undo every remapping (transparency is untouched).
    pub fn pal_all_reset(&mut self) {
        self.map = std::array::from_fn(|i| i as u8);
    }

    /// PICO-8 `palt(c, on)`: make colour `c` transparent (or opaque again).
    pub fn palt(&mut self, c: u8, on: bool) {
        self.transparent[(c & 15) as usize] = on;
    }

    /// Is this colour skipped when drawing?
    pub fn transparent(&self, c: u8) -> bool {
        self.transparent[(c & 15) as usize]
    }

    /// The RGB this draw colour ends up as, after `pal()` remapping.
    pub fn rgb(&self, c: u8) -> [u8; 3] {
        PALETTE[(self.map[(c & 15) as usize] & 15) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_is_0_0_0() {
        assert_eq!(PALETTE[0], [0, 0, 0]);
    }

    #[test]
    fn white_is_255_241_232() {
        assert_eq!(PALETTE[7], [255, 241, 232]);
    }

    #[test]
    fn red_is_255_0_77() {
        assert_eq!(PALETTE[8], [255, 0, 77]);
    }

    #[test]
    fn every_colour_is_distinct() {
        for a in 0..16 {
            for b in (a + 1)..16 {
                assert_ne!(PALETTE[a], PALETTE[b], "colours {a} and {b} collide");
            }
        }
    }

    #[test]
    fn pal_starts_as_identity_and_nothing_is_transparent() {
        let p = Pal::new();
        assert_eq!(p.map, std::array::from_fn::<u8, 16, _>(|i| i as u8));
        for c in 0..16u8 {
            assert_eq!(p.rgb(c), PALETTE[c as usize]);
            assert!(!p.transparent(c));
        }
    }

    #[test]
    fn pal_remaps_one_index() {
        let mut p = Pal::new();
        p.pal(1, 3); // draw colour 1 as colour 3
        assert_eq!(p.rgb(1), PALETTE[3]);
        assert_eq!(p.rgb(2), PALETTE[2]);
        p.pal_reset(1);
        assert_eq!(p.rgb(1), PALETTE[1]);
    }

    #[test]
    fn pal_reset_undoes_everything() {
        let mut p = Pal::new();
        p.pal(0, 15);
        p.pal(9, 2);
        p.pal_all_reset();
        for c in 0..16u8 {
            assert_eq!(p.rgb(c), PALETTE[c as usize]);
        }
    }

    #[test]
    fn palt_toggles_transparency() {
        let mut p = Pal::new();
        p.palt(0, true);
        assert!(p.transparent(0));
        assert!(!p.transparent(1));
        p.palt(0, false);
        assert!(!p.transparent(0));
    }
}
