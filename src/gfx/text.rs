//! Text: PICO-8's `print`/`cprint`/`coprint` on top of a 3x5 bitmap font.
//!
//! PICO-8's own font is **not** in the cart: it lives in PICO-8's ROM at
//! `0x5600`, and `__gfx__` holds only sprites (see `docs/gotchas.md`). The port
//! bundles **Tom Thumb**, a public-domain 3x5 font that uses the same 4 px cell
//! and 6 px line as PICO-8, so the cart's layout numbers still add up.
//!
//! Provenance and licence: Tom Thumb by Brian Swetland (1999, 3-clause BSD) with
//! modifications by Robey Pointer (2010), released by the author as CC0 / CC-BY
//! 3.0. The full notice is in `assets/FONT-LICENSE.txt`; the glyph table below is
//! that font's data, converted from the BSD/CC0 BDF by way of Adafruit GFX.
//!
//! Metrics: every character advances 4 px (2 px for space in the source font, but
//! PICO-8 always advances 4, and the cart's `cprint` maths assumes it), lines are
//! 6 px apart, and glyphs sit on a 5 px body with 1 px of descender.

use crate::gfx::fb::Fb;

/// One glyph: up to six rows of three pixels (bits 7..5), starting `top` pixels
/// below the text origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    pub top: i8,
    pub height: u8,
    pub rows: [u8; 6],
}

use Glyph as G;

/// The 95 printable ASCII glyphs, 32..=126.
///
/// Kept one per line and out of rustfmt's hands: it is data, not code.
#[rustfmt::skip]
pub const GLYPHS: [Glyph; 95] = [
    G { top: 0, height: 1, rows: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00] }, // 32 space
    G { top: 0, height: 5, rows: [0x80, 0x80, 0x80, 0x00, 0x80, 0x00] }, // 33 !
    G { top: 0, height: 2, rows: [0xa0, 0xa0, 0x00, 0x00, 0x00, 0x00] }, // 34 quotedbl
    G { top: 0, height: 5, rows: [0xa0, 0xe0, 0xa0, 0xe0, 0xa0, 0x00] }, // 35 #
    G { top: 0, height: 5, rows: [0x60, 0xc0, 0x60, 0xc0, 0x40, 0x00] }, // 36 $
    G { top: 0, height: 5, rows: [0x80, 0x20, 0x40, 0x80, 0x20, 0x00] }, // 37 %
    G { top: 0, height: 5, rows: [0xc0, 0xc0, 0xe0, 0xa0, 0x60, 0x00] }, // 38 &
    G { top: 0, height: 2, rows: [0x80, 0x80, 0x00, 0x00, 0x00, 0x00] }, // 39 '
    G { top: 0, height: 5, rows: [0x40, 0x80, 0x80, 0x80, 0x40, 0x00] }, // 40 (
    G { top: 0, height: 5, rows: [0x80, 0x40, 0x40, 0x40, 0x80, 0x00] }, // 41 )
    G { top: 0, height: 3, rows: [0xa0, 0x40, 0xa0, 0x00, 0x00, 0x00] }, // 42 *
    G { top: 1, height: 3, rows: [0x40, 0xe0, 0x40, 0x00, 0x00, 0x00] }, // 43 +
    G { top: 3, height: 2, rows: [0x40, 0x80, 0x00, 0x00, 0x00, 0x00] }, // 44 ,
    G { top: 2, height: 1, rows: [0xe0, 0x00, 0x00, 0x00, 0x00, 0x00] }, // 45 -
    G { top: 4, height: 1, rows: [0x80, 0x00, 0x00, 0x00, 0x00, 0x00] }, // 46 .
    G { top: 0, height: 5, rows: [0x20, 0x20, 0x40, 0x80, 0x80, 0x00] }, // 47 /
    G { top: 0, height: 5, rows: [0x60, 0xa0, 0xa0, 0xa0, 0xc0, 0x00] }, // 48 0
    G { top: 0, height: 5, rows: [0x40, 0xc0, 0x40, 0x40, 0x40, 0x00] }, // 49 1
    G { top: 0, height: 5, rows: [0xc0, 0x20, 0x40, 0x80, 0xe0, 0x00] }, // 50 2
    G { top: 0, height: 5, rows: [0xc0, 0x20, 0x40, 0x20, 0xc0, 0x00] }, // 51 3
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0xe0, 0x20, 0x20, 0x00] }, // 52 4
    G { top: 0, height: 5, rows: [0xe0, 0x80, 0xc0, 0x20, 0xc0, 0x00] }, // 53 5
    G { top: 0, height: 5, rows: [0x60, 0x80, 0xe0, 0xa0, 0xe0, 0x00] }, // 54 6
    G { top: 0, height: 5, rows: [0xe0, 0x20, 0x40, 0x80, 0x80, 0x00] }, // 55 7
    G { top: 0, height: 5, rows: [0xe0, 0xa0, 0xe0, 0xa0, 0xe0, 0x00] }, // 56 8
    G { top: 0, height: 5, rows: [0xe0, 0xa0, 0xe0, 0x20, 0xc0, 0x00] }, // 57 9
    G { top: 1, height: 3, rows: [0x80, 0x00, 0x80, 0x00, 0x00, 0x00] }, // 58 :
    G { top: 1, height: 4, rows: [0x40, 0x00, 0x40, 0x80, 0x00, 0x00] }, // 59 ;
    G { top: 0, height: 5, rows: [0x20, 0x40, 0x80, 0x40, 0x20, 0x00] }, // 60 <
    G { top: 1, height: 3, rows: [0xe0, 0x00, 0xe0, 0x00, 0x00, 0x00] }, // 61 =
    G { top: 0, height: 5, rows: [0x80, 0x40, 0x20, 0x40, 0x80, 0x00] }, // 62 >
    G { top: 0, height: 5, rows: [0xe0, 0x20, 0x40, 0x00, 0x40, 0x00] }, // 63 ?
    G { top: 0, height: 5, rows: [0x40, 0xa0, 0xe0, 0x80, 0x60, 0x00] }, // 64 @
    G { top: 0, height: 5, rows: [0x40, 0xa0, 0xe0, 0xa0, 0xa0, 0x00] }, // 65 A
    G { top: 0, height: 5, rows: [0xc0, 0xa0, 0xc0, 0xa0, 0xc0, 0x00] }, // 66 B
    G { top: 0, height: 5, rows: [0x60, 0x80, 0x80, 0x80, 0x60, 0x00] }, // 67 C
    G { top: 0, height: 5, rows: [0xc0, 0xa0, 0xa0, 0xa0, 0xc0, 0x00] }, // 68 D
    G { top: 0, height: 5, rows: [0xe0, 0x80, 0xe0, 0x80, 0xe0, 0x00] }, // 69 E
    G { top: 0, height: 5, rows: [0xe0, 0x80, 0xe0, 0x80, 0x80, 0x00] }, // 70 F
    G { top: 0, height: 5, rows: [0x60, 0x80, 0xe0, 0xa0, 0x60, 0x00] }, // 71 G
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0xe0, 0xa0, 0xa0, 0x00] }, // 72 H
    G { top: 0, height: 5, rows: [0xe0, 0x40, 0x40, 0x40, 0xe0, 0x00] }, // 73 I
    G { top: 0, height: 5, rows: [0x20, 0x20, 0x20, 0xa0, 0x40, 0x00] }, // 74 J
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0xc0, 0xa0, 0xa0, 0x00] }, // 75 K
    G { top: 0, height: 5, rows: [0x80, 0x80, 0x80, 0x80, 0xe0, 0x00] }, // 76 L
    G { top: 0, height: 5, rows: [0xa0, 0xe0, 0xe0, 0xa0, 0xa0, 0x00] }, // 77 M
    G { top: 0, height: 5, rows: [0xa0, 0xe0, 0xe0, 0xe0, 0xa0, 0x00] }, // 78 N
    G { top: 0, height: 5, rows: [0x40, 0xa0, 0xa0, 0xa0, 0x40, 0x00] }, // 79 O
    G { top: 0, height: 5, rows: [0xc0, 0xa0, 0xc0, 0x80, 0x80, 0x00] }, // 80 P
    G { top: 0, height: 5, rows: [0x40, 0xa0, 0xa0, 0xe0, 0x60, 0x00] }, // 81 Q
    G { top: 0, height: 5, rows: [0xc0, 0xa0, 0xe0, 0xc0, 0xa0, 0x00] }, // 82 R
    G { top: 0, height: 5, rows: [0x60, 0x80, 0x40, 0x20, 0xc0, 0x00] }, // 83 S
    G { top: 0, height: 5, rows: [0xe0, 0x40, 0x40, 0x40, 0x40, 0x00] }, // 84 T
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0xa0, 0xa0, 0x60, 0x00] }, // 85 U
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0xa0, 0x40, 0x40, 0x00] }, // 86 V
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0xe0, 0xe0, 0xa0, 0x00] }, // 87 W
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0x40, 0xa0, 0xa0, 0x00] }, // 88 X
    G { top: 0, height: 5, rows: [0xa0, 0xa0, 0x40, 0x40, 0x40, 0x00] }, // 89 Y
    G { top: 0, height: 5, rows: [0xe0, 0x20, 0x40, 0x80, 0xe0, 0x00] }, // 90 Z
    G { top: 0, height: 5, rows: [0xe0, 0x80, 0x80, 0x80, 0xe0, 0x00] }, // 91 [
    G { top: 1, height: 3, rows: [0x80, 0x40, 0x20, 0x00, 0x00, 0x00] }, // 92 backslash
    G { top: 0, height: 5, rows: [0xe0, 0x20, 0x20, 0x20, 0xe0, 0x00] }, // 93 ]
    G { top: 0, height: 2, rows: [0x40, 0xa0, 0x00, 0x00, 0x00, 0x00] }, // 94 ^
    G { top: 4, height: 1, rows: [0xe0, 0x00, 0x00, 0x00, 0x00, 0x00] }, // 95 _
    G { top: 0, height: 2, rows: [0x80, 0x40, 0x00, 0x00, 0x00, 0x00] }, // 96 `
    G { top: 1, height: 4, rows: [0xc0, 0x60, 0xa0, 0xe0, 0x00, 0x00] }, // 97 a
    G { top: 0, height: 5, rows: [0x80, 0xc0, 0xa0, 0xa0, 0xc0, 0x00] }, // 98 b
    G { top: 1, height: 4, rows: [0x60, 0x80, 0x80, 0x60, 0x00, 0x00] }, // 99 c
    G { top: 0, height: 5, rows: [0x20, 0x60, 0xa0, 0xa0, 0x60, 0x00] }, // 100 d
    G { top: 1, height: 4, rows: [0x60, 0xa0, 0xc0, 0x60, 0x00, 0x00] }, // 101 e
    G { top: 0, height: 5, rows: [0x20, 0x40, 0xe0, 0x40, 0x40, 0x00] }, // 102 f
    G { top: 1, height: 5, rows: [0x60, 0xa0, 0xe0, 0x20, 0x40, 0x00] }, // 103 g
    G { top: 0, height: 5, rows: [0x80, 0xc0, 0xa0, 0xa0, 0xa0, 0x00] }, // 104 h
    G { top: 0, height: 5, rows: [0x80, 0x00, 0x80, 0x80, 0x80, 0x00] }, // 105 i
    G { top: 0, height: 6, rows: [0x20, 0x00, 0x20, 0x20, 0xa0, 0x40] }, // 106 j
    G { top: 0, height: 5, rows: [0x80, 0xa0, 0xc0, 0xc0, 0xa0, 0x00] }, // 107 k
    G { top: 0, height: 5, rows: [0xc0, 0x40, 0x40, 0x40, 0xe0, 0x00] }, // 108 l
    G { top: 1, height: 4, rows: [0xe0, 0xe0, 0xe0, 0xa0, 0x00, 0x00] }, // 109 m
    G { top: 1, height: 4, rows: [0xc0, 0xa0, 0xa0, 0xa0, 0x00, 0x00] }, // 110 n
    G { top: 1, height: 4, rows: [0x40, 0xa0, 0xa0, 0x40, 0x00, 0x00] }, // 111 o
    G { top: 1, height: 5, rows: [0xc0, 0xa0, 0xa0, 0xc0, 0x80, 0x00] }, // 112 p
    G { top: 1, height: 5, rows: [0x60, 0xa0, 0xa0, 0x60, 0x20, 0x00] }, // 113 q
    G { top: 1, height: 4, rows: [0x60, 0x80, 0x80, 0x80, 0x00, 0x00] }, // 114 r
    G { top: 1, height: 4, rows: [0x60, 0xc0, 0x60, 0xc0, 0x00, 0x00] }, // 115 s
    G { top: 0, height: 5, rows: [0x40, 0xe0, 0x40, 0x40, 0x60, 0x00] }, // 116 t
    G { top: 1, height: 4, rows: [0xa0, 0xa0, 0xa0, 0x60, 0x00, 0x00] }, // 117 u
    G { top: 1, height: 4, rows: [0xa0, 0xa0, 0xe0, 0x40, 0x00, 0x00] }, // 118 v
    G { top: 1, height: 4, rows: [0xa0, 0xe0, 0xe0, 0xe0, 0x00, 0x00] }, // 119 w
    G { top: 1, height: 4, rows: [0xa0, 0x40, 0x40, 0xa0, 0x00, 0x00] }, // 120 x
    G { top: 1, height: 5, rows: [0xa0, 0xa0, 0x60, 0x20, 0x40, 0x00] }, // 121 y
    G { top: 1, height: 4, rows: [0xe0, 0x60, 0xc0, 0xe0, 0x00, 0x00] }, // 122 z
    G { top: 0, height: 5, rows: [0x60, 0x40, 0x80, 0x40, 0x60, 0x00] }, // 123 {
    G { top: 0, height: 5, rows: [0x80, 0x80, 0x00, 0x80, 0x80, 0x00] }, // 124 |
    G { top: 0, height: 5, rows: [0xc0, 0x40, 0x20, 0x40, 0xc0, 0x00] }, // 125 }
    G { top: 0, height: 2, rows: [0x60, 0xc0, 0x00, 0x00, 0x00, 0x00] }, // 126 ~
];

/// Pixels between the starts of two characters.
pub const CHAR_ADVANCE: i32 = 4;
/// Pixels between two lines of text.
pub const LINE_HEIGHT: i32 = 6;

/// The glyph for a character; unknown characters draw `?`.
pub fn glyph(ch: char) -> &'static Glyph {
    let cp = ch as usize;
    if (32..=126).contains(&cp) {
        &GLYPHS[cp - 32]
    } else {
        &GLYPHS[(b'?' as usize) - 32]
    }
}

/// Width of the widest line, in pixels.
pub fn text_width(text: &str) -> i32 {
    text.lines()
        .map(|l| l.chars().count() as i32 * CHAR_ADVANCE)
        .max()
        .unwrap_or(0)
}

/// PICO-8 `print(_t, _x, _y, _c)`: left-aligned, `\n` starts a new line at the
/// original x.
pub fn print(fb: &mut Fb, text: &str, x: i32, y: i32, c: u8) {
    let mut cx = x;
    let mut cy = y;
    for ch in text.chars() {
        if ch == '\n' {
            cx = x;
            cy += LINE_HEIGHT;
            continue;
        }
        let g = glyph(ch);
        for r in 0..g.height as i32 {
            let bits = g.rows[r as usize];
            for col in 0..3 {
                if bits & (0x80 >> col) != 0 {
                    fb.pset(cx + col, cy + g.top as i32 + r, c);
                }
            }
        }
        cx += CHAR_ADVANCE;
    }
}

/// PICO-8 `cprint(_t, _x, _y, _c)`: centred on `x`, i.e. `x - #t*2`.
pub fn cprint(fb: &mut Fb, text: &str, x: i32, y: i32, c: u8) {
    print(fb, text, x - text_width(text) / 2, y, c);
}

/// PICO-8 `coprint(_t, _x, _y, _c, _c2)`: centred text with a one-pixel outline.
pub fn coprint(fb: &mut Fb, text: &str, x: i32, y: i32, c: u8, outline: u8) {
    for (dx, dy) in [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ] {
        cprint(fb, text, x + dx, y + dy, outline);
    }
    cprint(fb, text, x, y, c);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_printable_ascii_has_a_glyph() {
        assert_eq!(GLYPHS.len(), 95);
        for cp in 32..=126u8 {
            let g = glyph(cp as char);
            assert!(g.height <= 6, "{} is {g:?}", cp as char);
            assert!(g.top >= 0 && g.top as u8 + g.height <= 6, "{}", cp as char);
        }
    }

    #[test]
    fn unknown_characters_fall_back_to_a_question_mark() {
        assert_eq!(glyph('\u{1f600}'), glyph('?'));
    }

    #[test]
    fn a_space_advances_four_pixels_and_draws_nothing() {
        let mut fb = Fb::new();
        print(&mut fb, " ", 10, 20, 7);
        assert!(fb.px.iter().all(|&p| p == 0));
        assert_eq!(text_width(" "), 4);
    }

    #[test]
    fn capital_a_is_the_three_by_five_shape() {
        // From the font data: .#. / #.# / ### / #.# / #.#
        let mut fb = Fb::new();
        print(&mut fb, "A", 0, 0, 7);
        let lit: Vec<(i32, i32)> = (0..6)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .filter(|&(x, y)| fb.pget(x, y) == 7)
            .collect();
        assert_eq!(
            lit,
            vec![
                (1, 0),
                (0, 1),
                (2, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (0, 3),
                (2, 3),
                (0, 4),
                (2, 4)
            ]
        );
    }

    #[test]
    fn text_width_counts_four_pixels_a_character() {
        assert_eq!(text_width("abc"), 12);
        assert_eq!(text_width(""), 0);
        assert_eq!(text_width("ab\ncde"), 12, "the widest line wins");
    }

    #[test]
    fn cprint_centres_on_the_x_coordinate() {
        // "ab" is 8 px wide, so it starts at 64 - 4.
        let mut fb = Fb::new();
        cprint(&mut fb, "ab", 64, 0, 7);
        let mut leftmost = 128;
        for y in 0..6 {
            for x in 0..128 {
                if fb.pget(x, y) == 7 {
                    leftmost = leftmost.min(x);
                }
            }
        }
        assert_eq!(leftmost, 60);
    }

    #[test]
    fn coprint_puts_an_outline_around_the_text() {
        let mut fb = Fb::new();
        coprint(&mut fb, "I", 64, 20, 12, 7);
        // "I" is three pixels wide at its widest row, centred; the outline is
        // white one pixel outside the blue glyph.
        let blue: Vec<(i32, i32)> = (0..128)
            .flat_map(|y| (0..128).map(move |x| (x, y)))
            .filter(|&(x, y)| fb.pget(x, y) == 12)
            .collect();
        let white = (0..128)
            .flat_map(|y| (0..128).map(move |x| (x, y)))
            .filter(|&(x, y)| fb.pget(x, y) == 7)
            .count();
        assert!(!blue.is_empty());
        assert!(white > blue.len(), "outline: {white} vs {}", blue.len());
    }

    #[test]
    fn a_newline_moves_six_pixels_down_and_back_to_x() {
        let mut fb = Fb::new();
        print(&mut fb, "I\nI", 10, 10, 7);
        assert_eq!(fb.pget(11, 10), 7);
        assert_eq!(fb.pget(11, 16), 7, "second line is 6 px lower");
    }

    #[test]
    fn drawing_past_the_edge_clips_instead_of_panicking() {
        let mut fb = Fb::new();
        print(&mut fb, "xxxxxx", 120, 124, 7);
        // Nothing crashed and the last visible row was written.
        assert!(fb.px.contains(&7));
    }
}
