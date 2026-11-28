//! Drawing: PICO-8-shaped primitives on a 128x128 indexed framebuffer.

pub mod fb;
pub mod palette;
pub mod scenes;
pub mod sprites;
pub mod text;

use fb::Fb;

/// The M1 palette test card: 16 colour bars, then one 14x14 block per colour.
///
/// Drawn by both `cargo run` (into the window) and `cargo run --bin shot`
/// (into a PNG), so the two paths can be compared without a GPU.
pub fn palette_card(fb: &mut Fb) {
    fb.clear(1);
    for i in 0..16u8 {
        let x = i as i32 * 8;
        fb.rectfill(x, 0, x + 7, 15, i);
    }
    for i in 0..16u8 {
        let cx = (i as i32 % 8) * 16 + 1;
        let cy = 32 + (i as i32 / 8) * 32;
        fb.rectfill(cx, cy, cx + 13, cy + 13, i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_card_draws_the_bars_in_index_order() {
        let mut fb = Fb::new();
        palette_card(&mut fb);
        for i in 0..16u8 {
            assert_eq!(fb.pget(i as i32 * 8, 0), i, "bar {i} has the wrong colour");
            assert_eq!(fb.pget(i as i32 * 8 + 7, 15), i, "bar {i} is too narrow");
        }
        assert_eq!(fb.pget(0, 16), 1, "background should be dark blue");
    }

    #[test]
    fn palette_card_draws_the_blocks() {
        let mut fb = Fb::new();
        palette_card(&mut fb);
        assert_eq!(fb.pget(1, 32), 0); // block 0, top-left
        assert_eq!(fb.pget(14, 45), 0); // block 0, bottom-right
        assert_eq!(fb.pget(113, 32), 7); // block 7 is white
        assert_eq!(fb.pget(113, 77), 15); // block 15
    }
}
