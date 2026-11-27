//! The cart's 128x128 sprite sheet.
//!
//! PICO-8 stores the sheet as one byte per two pixels, low nibble first, so
//! `assets/sprites.bin` is exactly 8192 bytes. `tools/p8_import.py` writes it from
//! `assets/upstream/bubblegum_spin.p8` (which stays gitignored); the sheet itself is
//! derived data like `src/consts.rs`, not a copy of the cart's source.

/// The cart's sprite sheet: 8192 bytes, two pixels per byte.
pub static CART: &[u8] = include_bytes!("../../assets/sprites.bin");

/// A 128x128 nibble sheet, borrowed from a byte slice.
///
/// Borrowed rather than owned so tests can point at a small `Vec` and the game can
/// point at the compiled-in cart without copying 8 KB around.
#[derive(Debug, Clone, Copy)]
pub struct Sheet<'a> {
    pub data: &'a [u8],
}

/// The cart's sheet.
pub fn cart() -> Sheet<'static> {
    Sheet { data: CART }
}

impl<'a> Sheet<'a> {
    /// One pixel, 0..15. Outside the sheet is 0, like PICO-8's clipping.
    #[inline]
    pub fn pixel(&self, x: i32, y: i32) -> u8 {
        if !(0..128).contains(&x) || !(0..128).contains(&y) {
            return 0;
        }
        let i = (y * 128 + x) as usize;
        let byte = self.data[i / 2];
        if i.is_multiple_of(2) {
            byte & 0x0f
        } else {
            byte >> 4
        }
    }

    /// The top-left pixel of sprite `n` (16 sprites per row).
    #[inline]
    pub fn sprite_origin(n: i32) -> (i32, i32) {
        ((n % 16) * 8, (n / 16) * 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cart_sheet_is_eight_kilobytes() {
        assert_eq!(CART.len(), 8192);
    }

    #[test]
    fn pixels_are_low_nibble_first() {
        // Byte 0 holds (x=0,y=0) in the low nibble and (x=1,y=0) in the high one.
        let data = [0x3a_u8]; // low 0xa = 10 at (0,0), high 0x3 = 3 at (1,0)
        let sheet = Sheet { data: &data };
        assert_eq!(sheet.pixel(0, 0), 10);
        assert_eq!(sheet.pixel(1, 0), 3);
    }

    #[test]
    fn outside_the_sheet_is_zero() {
        let sheet = cart();
        assert_eq!(sheet.pixel(-1, 0), 0);
        assert_eq!(sheet.pixel(0, -1), 0);
        assert_eq!(sheet.pixel(128, 0), 0);
        assert_eq!(sheet.pixel(0, 128), 0);
    }

    #[test]
    fn sprites_are_sixteen_per_row() {
        assert_eq!(Sheet::sprite_origin(0), (0, 0));
        assert_eq!(Sheet::sprite_origin(2), (16, 0));
        assert_eq!(Sheet::sprite_origin(16), (0, 8));
        assert_eq!(Sheet::sprite_origin(21), (40, 8));
        assert_eq!(Sheet::sprite_origin(255), (120, 120));
    }

    #[test]
    fn the_cart_gumball_sprite_is_the_three_shades_and_a_white_spec() {
        // Read off the real sheet: sprite 2 uses 9 (highlight), 4 (shade) and 7
        // (the specular dot), on a transparent 0. `bubcol` recolours 9 and 4 for
        // each gumball colour; 7 stays white.
        let sheet = cart();
        let mut seen = std::collections::BTreeSet::new();
        for dy in 0..8 {
            for dx in 0..8 {
                seen.insert(sheet.pixel(16 + dx, dy));
            }
        }
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), vec![0, 4, 7, 9]);
    }

    #[test]
    fn the_aimer_sprite_is_a_white_ring() {
        // Sprite 21 at (40,8): a hollow 8x8 ring of colour 7.
        let sheet = cart();
        let mut seen = std::collections::BTreeSet::new();
        for dy in 0..8 {
            for dx in 0..8 {
                seen.insert(sheet.pixel(40 + dx, 8 + dy));
            }
        }
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), vec![0, 7]);
    }
}
