//! The 128x128 palette-indexed framebuffer that every frame is drawn into.
//!
//! It is plain data with no GPU and no window, so a whole scene can be rendered
//! and compared (or written to PNG) inside a unit test.

use crate::gfx::palette::Pal;

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
    pub fn clear(&mut self, c: u8) {
        self.px.fill(c & 15);
    }

    /// PICO-8 `pset(x, y, c)`. Out-of-bounds writes are silently dropped,
    /// like PICO-8's clipping.
    pub fn pset(&mut self, x: i32, y: i32, c: u8) {
        if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
            return;
        }
        self.px[(y * WIDTH + x) as usize] = c & 15;
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
        for y in ya..=yb {
            let row = (y * WIDTH) as usize;
            for x in xa..=xb {
                self.px[row + x as usize] = c & 15;
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
            rgb.extend_from_slice(&self.pal.rgb(c));
        }
        writer.write_image_data(&rgb)?;
        Ok(())
    }

    /// The same pixels as RGBA8, for uploading to a GPU texture.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.px.len() * 4);
        for &c in self.px.iter() {
            let [r, g, b] = self.pal.rgb(c);
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
        fb.clear(1);
        fb.pal.pal(1, 7); // draw colour 1 as white
        let path = std::env::temp_dir().join("bubblegum-spin-fb-pal-test.png");
        fb.to_png(&path).unwrap();

        let dec = png::Decoder::new(std::fs::File::open(&path).unwrap());
        let mut reader = dec.read_info().unwrap();
        let mut buf = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        assert_eq!(&buf[0..3], &[255, 241, 232]);
        std::fs::remove_file(&path).ok();
    }
}
