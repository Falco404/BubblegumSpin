//! The 128x128 palette-indexed framebuffer that every frame is drawn into.
//!
//! It is plain data with no GPU and no window, so a whole scene can be rendered
//! and compared (or written to PNG) inside a unit test.

use crate::gfx::palette::{Pal, PALETTE};
use crate::gfx::sprites::Sheet;

/// PICO-8's screen, 128x128 pixels of palette indices.
pub const WIDTH: i32 = 128;
pub const HEIGHT: i32 = 128;

pub struct Fb {
    /// One palette index per pixel, row-major, 128*128 entries.
    pub px: [u8; (WIDTH * HEIGHT) as usize],
    /// `pal()` / `palt()` state used when the buffer is turned into RGB.
    pub pal: Pal,
}

impl Default for Fb {
    fn default() -> Self {
        Self::new()
    }
}

impl Fb {
    /// A black screen.
    pub fn new() -> Self {
        Self {
            px: [0; (WIDTH * HEIGHT) as usize],
            pal: Pal::new(),
        }
    }

    /// PICO-8 `cls(c)`.
    ///
    /// `cls` ignores `palt` (the screen is filled even with colour 0), but it does
    /// honour `pal`.
    pub fn clear(&mut self, c: u8) {
        let c = self.shown(c);
        self.px.fill(c);
    }

    /// The colour actually written for draw colour `c`: PICO-8 applies `pal()` at
    /// draw time, not when the screen is converted.
    #[inline]
    pub fn shown(&self, c: u8) -> u8 {
        self.pal.map[(c & 15) as usize]
    }

    /// PICO-8 `pset(x, y, c)`. Out-of-bounds writes are silently dropped,
    /// like PICO-8's clipping.
    pub fn pset(&mut self, x: i32, y: i32, c: u8) {
        if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
            return;
        }
        let c = self.shown(c);
        self.px[(y * WIDTH + x) as usize] = c;
    }

    /// PICO-8 `pget(x, y)`; 0 outside the screen.
    pub fn pget(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
            return 0;
        }
        self.px[(y * WIDTH + x) as usize]
    }

    /// PICO-8 `rectfill(x0, y0, x1, y1, c)`: inclusive corners, either order,
    /// clipped to the screen.
    pub fn rectfill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        let xa = x0.min(x1).max(0);
        let xb = x0.max(x1).min(WIDTH - 1);
        let ya = y0.min(y1).max(0);
        let yb = y0.max(y1).min(HEIGHT - 1);
        if xa > xb || ya > yb {
            return;
        }
        let c = self.shown(c);
        for y in ya..=yb {
            let row = (y * WIDTH) as usize;
            for x in xa..=xb {
                self.px[row + x as usize] = c;
            }
        }
    }

    /// PICO-8 `line(x0, y0, x1, y1, c)`: Bresenham, both ends included and clipped.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.pset(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// PICO-8 `spr(n, x, y)`: an 8x8 sprite from the sheet.
    pub fn spr(&mut self, sheet: &Sheet, n: i32, x: i32, y: i32) {
        self.spr_flip(sheet, n, x, y, false);
    }

    /// `spr` with PICO-8's horizontal flip.
    ///
    /// Colour 0 is skipped while `palt(0)` is on, which is PICO-8's default: a
    /// sprite's background is transparent unless asked otherwise.
    pub fn spr_flip(&mut self, sheet: &Sheet, n: i32, x: i32, y: i32, flip_x: bool) {
        let (sx, sy) = Sheet::sprite_origin(n);
        for dy in 0..8 {
            for dx in 0..8 {
                let src_x = sx + if flip_x { 7 - dx } else { dx };
                let c = sheet.pixel(src_x, sy + dy);
                if self.pal.transparent(c) {
                    continue;
                }
                self.pset(x + dx, y + dy, c);
            }
        }
    }

    /// PICO-8 `sspr(sx, sy, sw, sh, dx, dy, dw, dh)`: a scaled sprite blit.
    ///
    /// Nearest neighbour, no filtering: each destination pixel samples the source
    /// pixel its centre falls in, which is what PICO-8's fixed-point scaler does.
    #[allow(clippy::too_many_arguments)]
    pub fn sspr(
        &mut self,
        sheet: &Sheet,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        dx: i32,
        dy: i32,
        dw: i32,
        dh: i32,
    ) {
        if sw <= 0 || sh <= 0 || dw <= 0 || dh <= 0 {
            return;
        }
        for y in 0..dh {
            let src_y = sy + (y as i64 * sh as i64 / dh as i64) as i32;
            for x in 0..dw {
                let src_x = sx + (x as i64 * sw as i64 / dw as i64) as i32;
                let c = sheet.pixel(src_x, src_y);
                if self.pal.transparent(c) {
                    continue;
                }
                self.pset(dx + x, dy + y, c);
            }
        }
    }

    /// `sspr` with the source mirrored, as the cart's optional `flipx` does.
    #[allow(clippy::too_many_arguments)]
    pub fn sspr_flip(
        &mut self,
        sheet: &Sheet,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        dx: i32,
        dy: i32,
        dw: i32,
        dh: i32,
    ) {
        if sw <= 0 || sh <= 0 || dw <= 0 || dh <= 0 {
            return;
        }
        for y in 0..dh {
            let src_y = sy + (y as i64 * sh as i64 / dh as i64) as i32;
            for x in 0..dw {
                let flipped = (dw - 1 - x) as i64;
                let src_x = sx + (flipped * sw as i64 / dw as i64) as i32;
                let c = sheet.pixel(src_x, src_y);
                if self.pal.transparent(c) {
                    continue;
                }
                self.pset(dx + x, dy + y, c);
            }
        }
    }

    /// Flatten the buffer to RGB through `pal` and write it as a PNG.
    ///
    /// This is the headless screenshot path: no GPU, no window.
    pub fn to_png<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), WIDTH as u32, HEIGHT as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;

        let mut rgb = Vec::with_capacity(self.px.len() * 3);
        for &c in self.px.iter() {
            rgb.extend_from_slice(&PALETTE[c as usize]);
        }
        writer.write_image_data(&rgb)?;
        Ok(())
    }

    /// PICO-8 `circfill(x, y, r, c)`: a filled disc of radius `r`.
    ///
    /// PICO-8's own circle routine uses a slightly different pixel pattern at
    /// some radii; this port uses the plain "`dx² + dy² <= r²`" rule, which
    /// matches PICO-8 exactly at r = 0, 1 and 4 (the radii this game uses) and is
    /// only a placeholder for the real gumball sprites anyway (M6).
    pub fn circfill(&mut self, x: i32, y: i32, r: i32, c: u8) {
        if r < 0 {
            return;
        }
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    self.pset(x + dx, y + dy, c);
                }
            }
        }
    }

    /// The same pixels as RGBA8, for uploading to a GPU texture.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.px.len() * 4);
        for &c in self.px.iter() {
            let [r, g, b] = PALETTE[c as usize];
            rgba.extend_from_slice(&[r, g, b, 255]);
        }
        rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_black() {
        let fb = Fb::new();
        assert!(fb.px.iter().all(|&p| p == 0));
        assert_eq!(fb.px.len(), 128 * 128);
    }

    #[test]
    fn clear_paints_every_pixel() {
        let mut fb = Fb::new();
        fb.clear(12);
        assert!(fb.px.iter().all(|&p| p == 12));
    }

    #[test]
    fn pset_and_pget_round_trip() {
        let mut fb = Fb::new();
        fb.pset(10, 20, 8);
        assert_eq!(fb.pget(10, 20), 8);
        assert_eq!(fb.pget(11, 20), 0);
    }

    #[test]
    fn pset_out_of_bounds_is_a_noop() {
        let mut fb = Fb::new();
        for (x, y) in [(-1, 0), (0, -1), (128, 0), (0, 128), (999, 999)] {
            fb.pset(x, y, 9); // must not panic or wrap into the buffer
        }
        assert!(fb.px.iter().all(|&p| p == 0));
    }

    #[test]
    fn pget_out_of_bounds_is_zero() {
        let fb = Fb::new();
        assert_eq!(fb.pget(-1, 0), 0);
        assert_eq!(fb.pget(128, 127), 0);
    }

    #[test]
    fn rectfill_covers_both_corners_inclusively() {
        let mut fb = Fb::new();
        fb.rectfill(2, 2, 4, 4, 7);
        let lit = fb.px.iter().filter(|&&p| p == 7).count();
        assert_eq!(lit, 9); // 3x3, corners included
        assert_eq!(fb.pget(2, 2), 7);
        assert_eq!(fb.pget(4, 4), 7);
        assert_eq!(fb.pget(1, 2), 0);
        assert_eq!(fb.pget(2, 5), 0);
    }

    #[test]
    fn rectfill_accepts_swapped_corners() {
        let mut fb = Fb::new();
        fb.rectfill(4, 4, 2, 2, 7);
        assert_eq!(fb.px.iter().filter(|&&p| p == 7).count(), 9);
        assert_eq!(fb.pget(2, 2), 7);
        assert_eq!(fb.pget(4, 4), 7);
    }

    #[test]
    fn rectfill_clips_at_the_screen_edges() {
        let mut fb = Fb::new();
        fb.rectfill(-5, -5, 0, 0, 3);
        assert_eq!(fb.pget(0, 0), 3);
        assert_eq!(fb.px.iter().filter(|&&p| p == 3).count(), 1);

        let mut fb = Fb::new();
        fb.rectfill(126, 126, 200, 200, 3); // 2x2 inside the screen
        assert_eq!(fb.px.iter().filter(|&&p| p == 3).count(), 4);

        let mut fb = Fb::new();
        fb.rectfill(200, 200, 300, 300, 3); // entirely off-screen
        assert!(fb.px.iter().all(|&p| p == 0));
    }

    fn count(fb: &Fb, c: u8) -> usize {
        fb.px.iter().filter(|&&p| p == c).count()
    }

    #[test]
    fn circfill_radius_zero_is_one_pixel() {
        let mut fb = Fb::new();
        fb.circfill(10, 10, 0, 7);
        assert_eq!(count(&fb, 7), 1);
        assert_eq!(fb.pget(10, 10), 7);
    }

    #[test]
    fn circfill_radius_one_is_a_five_pixel_cross() {
        // PICO-8: circfill(x,y,1) covers the centre and its four neighbours.
        let mut fb = Fb::new();
        fb.circfill(10, 10, 1, 7);
        assert_eq!(count(&fb, 7), 5);
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            assert_eq!(fb.pget(10 + dx, 10 + dy), 7, "missing ({dx},{dy})");
        }
        assert_eq!(fb.pget(11, 11), 0);
    }

    #[test]
    fn circfill_radius_four_is_a_49_pixel_disc() {
        // Lattice points with dx^2 + dy^2 <= 16: 49 of them, 9 wide.
        let mut fb = Fb::new();
        fb.circfill(20, 20, 4, 7);
        assert_eq!(count(&fb, 7), 49);
        assert_eq!(fb.pget(16, 20), 7);
        assert_eq!(fb.pget(24, 20), 7);
        assert_eq!(fb.pget(16, 16), 0, "corner is outside the disc");
    }

    #[test]
    fn circfill_clips_at_the_edges() {
        let mut fb = Fb::new();
        fb.circfill(0, 0, 4, 7); // must not panic or wrap
        assert_eq!(fb.pget(0, 0), 7);
        fb.circfill(127, 127, 4, 9);
        assert_eq!(fb.pget(127, 127), 9);
    }

    #[test]
    fn circfill_draws_over_what_is_there() {
        let mut fb = Fb::new();
        fb.clear(1);
        fb.circfill(64, 64, 2, 8);
        assert_eq!(fb.pget(64, 64), 8);
        assert_eq!(fb.pget(62, 64), 8);
        assert_eq!(fb.pget(67, 64), 1);
    }

    #[test]
    fn to_png_writes_128x128_rgb_through_the_palette() {
        let mut fb = Fb::new();
        fb.clear(8);
        let path = std::env::temp_dir().join("bubblegum-spin-fb-test.png");
        fb.to_png(&path).unwrap();

        let dec = png::Decoder::new(std::fs::File::open(&path).unwrap());
        let mut reader = dec.read_info().unwrap();
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (128, 128));
        assert_eq!(info.color_type, png::ColorType::Rgb);
        assert_eq!(&buf[0..3], &[255, 0, 77]); // colour 8 is red
        assert_eq!(buf.len(), 128 * 128 * 3);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn to_png_follows_pal_remapping() {
        let mut fb = Fb::new();
        fb.pal.pal(1, 7); // draw colour 1 as white
        fb.clear(1);
        let path = std::env::temp_dir().join("bubblegum-spin-fb-pal-test.png");
        fb.to_png(&path).unwrap();

        let dec = png::Decoder::new(std::fs::File::open(&path).unwrap());
        let mut reader = dec.read_info().unwrap();
        let mut buf = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        assert_eq!(&buf[0..3], &[255, 241, 232]);
        std::fs::remove_file(&path).ok();
    }

    /// A sheet with a few pixels lit, for the drawing tests.
    fn sheet_with(pixels: &[(i32, i32, u8)]) -> Vec<u8> {
        let mut data = vec![0u8; 8192];
        for &(x, y, c) in pixels {
            let i = (y * 128 + x) as usize;
            if i.is_multiple_of(2) {
                data[i / 2] |= c & 15;
            } else {
                data[i / 2] |= (c & 15) << 4;
            }
        }
        data
    }

    #[test]
    fn pal_is_applied_when_the_pixel_is_drawn() {
        // PICO-8 remaps at draw time, so a later pal() reset does not repaint.
        let mut fb = Fb::new();
        fb.pal.pal(9, 8);
        fb.pset(1, 1, 9);
        fb.pal.pal_all_reset();
        assert_eq!(fb.pget(1, 1), 8);
    }

    #[test]
    fn spr_draws_sprite_n_at_the_corner() {
        // Sprite 2 starts at (16,0); light its top-left pixel with colour 3.
        let data = sheet_with(&[(16, 0, 3)]);
        let sheet = Sheet { data: &data };
        let mut fb = Fb::new();
        fb.spr(&sheet, 2, 10, 20);
        assert_eq!(fb.pget(10, 20), 3);
        assert_eq!(fb.pget(9, 20), 0, "nothing left of the sprite");
        assert_eq!(fb.pget(18, 20), 0, "nothing right of an 8 px sprite");
    }

    #[test]
    fn spr_skips_colour_zero_while_palt_zero_is_on() {
        let data = sheet_with(&[(16, 0, 3), (17, 0, 0)]);
        let sheet = Sheet { data: &data };
        let mut fb = Fb::new();
        fb.clear(6);
        fb.spr(&sheet, 2, 0, 0);
        assert_eq!(fb.pget(0, 0), 3);
        assert_eq!(fb.pget(1, 0), 6, "colour 0 is transparent by default");

        fb.pal.palt(0, false);
        fb.spr(&sheet, 2, 0, 0);
        assert_eq!(fb.pget(1, 0), 0, "palt(0,false) makes 0 opaque");
    }

    #[test]
    fn spr_flip_mirrors_the_sprite() {
        let data = sheet_with(&[(16, 0, 5)]);
        let sheet = Sheet { data: &data };
        let mut fb = Fb::new();
        fb.spr_flip(&sheet, 2, 0, 0, true);
        assert_eq!(fb.pget(7, 0), 5);
        assert_eq!(fb.pget(0, 0), 0);
    }

    #[test]
    fn sspr_scales_by_nearest_neighbour() {
        // One lit pixel at (0,0) scaled to 4x4 fills that quarter of the target.
        let data = sheet_with(&[(0, 0, 7), (1, 0, 9)]);
        let sheet = Sheet { data: &data };
        let mut fb = Fb::new();
        fb.sspr(&sheet, 0, 0, 2, 1, 10, 20, 4, 2);
        assert_eq!(fb.pget(10, 20), 7);
        assert_eq!(fb.pget(11, 21), 7);
        assert_eq!(fb.pget(12, 20), 9);
        assert_eq!(fb.pget(13, 21), 9);
        assert_eq!(fb.pget(14, 20), 0, "outside the 4 px target");
    }

    #[test]
    fn sspr_refuses_a_degenerate_size() {
        let data = sheet_with(&[(0, 0, 7)]);
        let sheet = Sheet { data: &data };
        let mut fb = Fb::new();
        fb.sspr(&sheet, 0, 0, 0, 0, 0, 0, 8, 8); // must not panic
        assert_eq!(fb.pget(0, 0), 0);
    }

    #[test]
    fn line_includes_both_ends() {
        let mut fb = Fb::new();
        fb.line(0, 0, 10, 10, 4);
        assert_eq!(fb.pget(0, 0), 4);
        assert_eq!(fb.pget(10, 10), 4);
        assert_eq!(fb.px.iter().filter(|&&p| p == 4).count(), 11);
    }

    #[test]
    fn line_clips_instead_of_wrapping() {
        let mut fb = Fb::new();
        fb.line(-5, 0, 5, 0, 4); // must not panic or wrap to the row end
        assert_eq!(fb.pget(0, 0), 4);
        assert_eq!(fb.pget(127, 0), 0);
    }
}
