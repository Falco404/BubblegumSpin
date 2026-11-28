//! Screen plumbing: where the 128x128 picture sits in a window, and how a touch
//! becomes one of the cart's six buttons.
//!
//! The phone needs both answers from the same numbers — the picture's offset must be
//! the one the hit-testing uses, or a thumb press lands somewhere else on a tall
//! screen. Everything here is plain arithmetic, so the phone's layout is testable
//! without a phone.

use crate::sim::Input;

/// The cart's screen, in pixels.
pub const GAME: f32 = 128.0;

/// The picture on screen: a square, at an integer scale so the pixels stay crisp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Picture {
    pub x: f32,
    pub y: f32,
    pub side: f32,
}

impl Picture {
    /// Screen pixels per game pixel (an integer, by construction).
    pub fn scale(&self) -> f32 {
        self.side / GAME
    }

    /// A screen point in 128x128 game coordinates, or `None` outside the picture.
    pub fn to_game(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let scale = self.scale();
        let (gx, gy) = ((x - self.x) / scale, (y - self.y) / scale);
        if (0.0..GAME).contains(&gx) && (0.0..GAME).contains(&gy) {
            Some((gx, gy))
        } else {
            None
        }
    }
}

/// The cart's buttons, as PICO-8 names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Up,
    Down,
    /// The O button: hold to peek at the HUD.
    Peek,
    /// The X button: fire, or pick in a menu.
    Fire,
}

/// The band of on-screen controls under the picture.
///
/// A four-way pad in the bottom-left corner, then `O` and `X` on the right, with a
/// dead gap between them so a thumb resting mid-screen does nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strip {
    pub width: f32,
    pub top: f32,
    pub height: f32,
}

impl Strip {
    /// One button's side, in pixels.
    ///
    /// The strip is much taller than a thumb-friendly square on a portrait phone, so
    /// the buttons are squares limited by a fifth of the width, centred in the band.
    pub fn side(&self) -> f32 {
        (self.height.min(self.width * 0.2)).max(64.0)
    }

    /// The top of the button row, centred in the strip.
    pub fn row(&self) -> f32 {
        self.top + (self.height - self.side()) / 2.0
    }

    /// Which button a point in the strip hits, if any.
    pub fn button_at(&self, x: f32, y: f32) -> Option<Button> {
        let side = self.side();
        let top = self.row();
        if y < top || y >= top + side {
            return None;
        }
        // The pad: a square in the left corner, four ways by the longer axis.
        if x < side {
            let (dx, dy) = (x - side / 2.0, y - (top + side / 2.0));
            return Some(if dx.abs() > dy.abs() {
                if dx < 0.0 {
                    Button::Left
                } else {
                    Button::Right
                }
            } else if dy < 0.0 {
                Button::Up
            } else {
                Button::Down
            });
        }
        // Then `O` and `X`, right-aligned.
        if x >= self.width - side {
            return Some(Button::Fire);
        }
        if x >= self.width - 2.0 * side {
            return Some(Button::Peek);
        }
        None
    }

    /// The pad's square, for drawing: `(x, y, side)`.
    pub fn pad(&self) -> (f32, f32, f32) {
        (0.0, self.row(), self.side())
    }

    /// Where to draw `O` and `X`: `(x, y, side)` each, right-aligned.
    pub fn action_buttons(&self) -> ((f32, f32, f32), (f32, f32, f32)) {
        let side = self.side();
        let top = self.row();
        let peek = (self.width - 2.0 * side, top, side);
        let fire = (self.width - side, top, side);
        (peek, fire)
    }
}

/// Where everything goes for a given window size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub picture: Picture,
    /// `None` on a desktop window: the keyboard is the input, and the whole window is
    /// the picture.
    pub strip: Option<Strip>,
}

impl Layout {
    /// The whole window as a picture (desktop).
    pub fn desktop(width: f32, height: f32) -> Self {
        Self::for_screen(width, height, false)
    }

    /// A picture above a control strip (phone), or the plain desktop layout.
    ///
    /// The picture keeps a whole-number scale so its pixels stay square: on a 1080-wide
    /// phone that is 8x (1024 px) with 28 px bars, not 8.4x.
    pub fn for_screen(width: f32, height: f32, touch: bool) -> Self {
        let strip_height = if touch {
            (height * 0.22).clamp(96.0, height * 0.45)
        } else {
            0.0
        };
        let usable = height - strip_height;
        let scale = (width.min(usable) / GAME).floor().max(1.0);
        let side = scale * GAME;
        let picture = Picture {
            x: ((width - side) / 2.0).floor(),
            y: ((usable - side) / 2.0).floor(),
            side,
        };
        let strip = if touch {
            Some(Strip {
                width,
                top: usable,
                height: strip_height,
            })
        } else {
            None
        };
        Self { picture, strip }
    }

    /// The buttons held by these touch points (screen coordinates).
    pub fn input(&self, touches: &[(f32, f32)]) -> Input {
        let mut input = Input::default();
        let Some(strip) = self.strip else {
            return input;
        };
        for &(x, y) in touches {
            match strip.button_at(x, y) {
                Some(Button::Left) => input.left = true,
                Some(Button::Right) => input.right = true,
                Some(Button::Up) => input.up = true,
                Some(Button::Down) => input.down = true,
                Some(Button::Peek) => input.peek = true,
                Some(Button::Fire) => input.fire = true,
                None => {}
            }
        }
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A phone in portrait: 1080x2400, the shape the layout has to survive.
    fn phone() -> Layout {
        Layout::for_screen(1080.0, 2400.0, true)
    }

    #[test]
    fn a_desktop_window_keeps_its_centred_square_and_has_no_buttons() {
        let layout = Layout::desktop(512.0, 512.0);
        assert_eq!(
            layout.picture,
            Picture {
                x: 0.0,
                y: 0.0,
                side: 512.0
            }
        );
        assert_eq!(layout.picture.scale(), 4.0);
        assert!(layout.strip.is_none());
        assert_eq!(layout.input(&[(10.0, 10.0)]), Input::default());
    }

    #[test]
    fn the_picture_sits_above_the_strip_at_a_whole_pixel_scale() {
        let layout = phone();
        let strip = layout.strip.expect("a phone has a control strip");
        // 22% of 2400 is 528; the picture gets what is left and is an integer scale.
        assert_eq!(strip.height, 528.0);
        assert_eq!(strip.top, 1872.0);
        assert_eq!(layout.picture.side, 1024.0, "8x, not 8.4x");
        assert_eq!(layout.picture.scale(), 8.0);
        assert_eq!(layout.picture.x, 28.0);
        assert_eq!(layout.picture.y, 424.0);
    }

    #[test]
    fn a_wide_window_leaves_side_bars_and_the_strip_stays_at_the_bottom() {
        let layout = Layout::for_screen(2400.0, 1080.0, true);
        let strip = layout.strip.unwrap();
        assert_eq!(strip.height, 237.6);
        assert_eq!(strip.top, 842.4);
        // The picture is limited by the height left over, not the width.
        assert_eq!(layout.picture.side, 768.0);
        assert_eq!(layout.picture.x, 816.0);
    }

    #[test]
    fn the_pad_splits_into_four_directions() {
        let strip = phone().strip.unwrap();
        let (px, py, side) = strip.pad();
        let (cx, cy) = (px + side / 2.0, py + side / 2.0);
        assert_eq!(strip.button_at(cx - 100.0, cy), Some(Button::Left));
        assert_eq!(strip.button_at(cx + 100.0, cy), Some(Button::Right));
        assert_eq!(strip.button_at(cx, cy - 100.0), Some(Button::Up));
        assert_eq!(strip.button_at(cx, cy + 100.0), Some(Button::Down));
    }

    #[test]
    fn the_right_side_is_peek_then_fire() {
        let strip = phone().strip.unwrap();
        let (peek, fire) = strip.action_buttons();
        assert_eq!(
            strip.button_at(peek.0 + 10.0, peek.1 + 10.0),
            Some(Button::Peek)
        );
        assert_eq!(
            strip.button_at(fire.0 + 10.0, fire.1 + 10.0),
            Some(Button::Fire)
        );
        assert_eq!(
            strip.button_at(1080.0 - 1.0, strip.row() + 10.0),
            Some(Button::Fire),
            "the fire button reaches the right edge"
        );
    }

    #[test]
    fn the_gap_between_the_pad_and_the_buttons_is_dead() {
        let strip = phone().strip.unwrap();
        let (peek, _fire) = strip.action_buttons();
        let (_, _, side) = strip.pad();
        assert_eq!(
            strip.button_at(540.0, strip.row() + 10.0),
            None,
            "the middle of the strip is dead"
        );
        assert!(peek.0 >= side, "the buttons do not overlap the pad");
    }

    #[test]
    fn a_point_above_the_strip_is_not_a_button() {
        let strip = phone().strip.unwrap();
        assert_eq!(strip.button_at(100.0, strip.top - 1.0), None);
        assert_eq!(strip.button_at(100.0, strip.top + strip.height), None);
    }

    #[test]
    fn two_fingers_press_two_buttons() {
        let layout = phone();
        let strip = layout.strip.unwrap();
        let (px, py, side) = strip.pad();
        let (_peek, fire) = strip.action_buttons();
        let input = layout.input(&[
            (px + side * 0.2, py + side / 2.0),
            (fire.0 + side / 2.0, fire.1 + side / 2.0),
        ]);
        assert!(input.left, "aiming left");
        assert!(input.fire, "and firing at the same time");
        assert!(!input.right && !input.peek && !input.up && !input.down);
    }

    #[test]
    fn a_tap_maps_into_the_128_pixel_picture() {
        let layout = phone();
        let p = layout.picture;
        assert_eq!(p.to_game(p.x, p.y), Some((0.0, 0.0)));
        let (gx, gy) = p.to_game(p.x + p.side - 1.0, p.y + p.side - 1.0).unwrap();
        assert!((gx - 127.875).abs() < 0.001, "gx = {gx}");
        assert!((gy - 127.875).abs() < 0.001, "gy = {gy}");
        assert_eq!(p.to_game(p.x - 1.0, p.y), None, "outside the picture");
        assert_eq!(p.to_game(p.x, p.y + p.side), None);
    }
}
