//! Scene drawing: the cart's `draw_*` functions, as pure functions on a
//! framebuffer. No window, no audio, so every frame can be checked by a test.
//!
//! cart: `draw_game`, `draw_bub`, `bubcol`, `palswap`, `draw_danger`,
//! `draw_banner`, `score_card`, `nxt_card`, `cprint`, `coprint`, `fade_scr`.

use crate::fixed::{int, to_int};
use crate::gfx::fb::Fb;
use crate::gfx::palette::Pal;
use crate::gfx::sprites;
use crate::gfx::text::{cprint, print};
use crate::grid::b2px;
use crate::sim::spin::px2rpx;
use crate::sim::Game;

/// The cart's `__map__`: 128 columns x 16 rows of tile indices.
pub static MAP: &[u8] = include_bytes!("../../assets/map.bin");
/// Tiles per map row (PICO-8's map is 128 wide).
pub const MAP_COLS: i32 = 128;

/// The aimer's trail dots, at these pixel distances from the gun.
const DOTS: [i32; 3] = [5, 10, 15];

/// cart `palswap(_a,_b,_c)`: recolour the gumball sprite's three shades.
pub fn palswap(fb: &mut Fb, a: u8, b: u8, c: u8) {
    fb.pal.pal(9, a);
    fb.pal.pal(4, b);
    fb.pal.pal(2, c);
    fb.pal.palt(9, a == 0);
    fb.pal.palt(4, b == 0);
    fb.pal.palt(2, c == 0);
}

/// cart `bubcol(_v)`: choose the gumball palette for one cell value.
///
/// An exploding cell (`v > 300`) flickers: while `v % 130 > 70` it is drawn white.
pub fn bubcol(fb: &mut Fb, v: u16) {
    let mut v = v;
    if v > 300 && v % 130 > 70 {
        v = 8;
    }
    match v % 10 {
        1 => palswap(fb, 14, 2, 1),   // pink
        2 => palswap(fb, 8, 2, 1),    // red
        3 => palswap(fb, 12, 1, 1),   // blue
        4 => palswap(fb, 11, 3, 1),   // light green
        5 => palswap(fb, 10, 9, 4),   // yellow
        6 => palswap(fb, 2, 1, 1),    // dark red
        8 => palswap(fb, 7, 15, 4),   // the white anchor
        9 => palswap(fb, 13, 13, 13), // wall grey
        _ => {}
    }
}

/// cart `pal()`: back to the identity palette (colour 0 still transparent).
pub fn reset_pal(fb: &mut Fb) {
    fb.pal = Pal::new();
}

/// cart `draw_bub(_x,_y,_c,[_small])`: one gumball sprite in its colour.
pub fn draw_bub(fb: &mut Fb, x: i32, y: i32, col: u16, small: bool) {
    bubcol(fb, col);
    let sheet = sprites::cart();
    if small {
        fb.spr(&sheet, 1, x - 3, y - 2);
    } else {
        fb.spr(&sheet, 2, x - 4, y - 4);
    }
    reset_pal(fb);
}

/// PICO-8 `map(celx, cely, sx, sy, celw, celh)`: draw tiles from the map.
pub fn draw_map(fb: &mut Fb, celx: i32, cely: i32, sx: i32, sy: i32, celw: i32, celh: i32) {
    let sheet = sprites::cart();
    for cy in 0..celh {
        for cx in 0..celw {
            let i = (cely + cy) * MAP_COLS + celx + cx;
            if i < 0 || i as usize >= MAP.len() {
                continue;
            }
            let tile = MAP[i as usize];
            if tile == 0 {
                continue;
            }
            fb.spr(&sheet, tile as i32, sx + cx * 8, sy + cy * 8);
        }
    }
}

/// The whole map, like the cart's `map()`.
pub fn draw_map_all(fb: &mut Fb) {
    let rows = (MAP.len() as i32) / MAP_COLS;
    draw_map(fb, 0, 0, 0, 0, MAP_COLS, rows);
}

/// cart `draw_banner(_x,_y,_x2,_y2,_c,_c2)`: a rounded two-colour box.
pub fn draw_banner(fb: &mut Fb, x0: i32, y0: i32, x1: i32, y1: i32, c: u8, border: u8) {
    fb.rectfill(x0 + 2, y0, x1 - 2, y1, border);
    fb.rectfill(x0 + 1, y0 + 1, x1 - 1, y1 - 1, border);
    fb.rectfill(x0, y0 + 2, x1, y1 - 2, border);

    fb.rectfill(x0 + 2, y0 + 1, x1 - 2, y1 - 1, c);
    fb.rectfill(x0 + 1, y0 + 2, x1 - 1, y1 - 2, c);
}

/// cart `fade_scr(fa)`: the 15-step palette ladder that fades the screen out.
///
/// `fades[n]` is a *row* per draw colour `n` (1..15) and `fi` picks the column:
/// `fi = flr(fa/(1/8)) + 1`. Column 1 is the identity and column 8 is all black.
/// At `fa = 1` the cart asks for column 9, which is `nil`, so `pal(n,nil)` leaves
/// the palette alone — reproduced here as "no remapping".
pub fn fade_scr(fb: &mut Fb, fa: f64) {
    #[rustfmt::skip]
    const LADDER: [[u8; 8]; 15] = [
        [1, 1, 1, 1, 0, 0, 0, 0],
        [2, 2, 2, 1, 1, 0, 0, 0],
        [3, 3, 4, 5, 2, 1, 1, 0],
        [4, 4, 2, 2, 1, 1, 1, 0],
        [5, 5, 2, 2, 1, 1, 1, 0],
        [6, 6, 13, 5, 2, 1, 1, 0],
        [7, 7, 6, 13, 5, 2, 1, 0],
        [8, 8, 9, 4, 5, 2, 1, 0],
        [9, 9, 4, 5, 2, 1, 1, 0],
        [10, 15, 9, 4, 5, 2, 1, 0],
        [11, 11, 3, 4, 5, 2, 1, 0],
        [12, 12, 13, 5, 5, 2, 1, 0],
        [13, 13, 5, 5, 2, 1, 1, 0],
        [14, 9, 9, 4, 5, 2, 1, 0],
        [15, 14, 9, 4, 5, 2, 1, 0],
    ];
    let fa = fa.clamp(0.0, 1.0);
    let fi = (fa * 8.0).floor() as usize;
    if fi >= 8 {
        return; // the cart's `nil` column: nothing is remapped
    }
    for n in 1..=15u8 {
        fb.pal.pal(n, LADDER[(n - 1) as usize][fi]);
    }
}

/// One frame of the game scene.
pub fn draw_game(fb: &mut Fb, game: &Game) {
    fb.clear(6);

    // The five light rays at the top of the cabinet.
    for x in [48, 49, 56, 57, 58] {
        fb.line(x, 0, 0, x * 2, 7);
    }
    fb.rectfill(0, 72, 128, 128, 13);
    draw_map_all(fb);

    // Every settled cell, at its rotated screen position.
    for (&cell, &value) in game.board.0.iter() {
        let (gx, gy) = b2px(cell);
        let (x, y) = px2rpx(int(gx), int(gy), game.spin.var);
        draw_bub(fb, to_int(x), to_int(y), value, false);
    }

    // The bubble in the gun, the one in flight and any that were given up.
    for b in game.bubs.iter().chain(game.f_bubs.iter()) {
        draw_bub(fb, to_int(b.x), to_int(b.y), b.col, false);
    }

    draw_aimer(fb, game);

    // The floating score numbers: a white shadow, then the colour, once the
    // entry is past its half-life (cart `♥ > 60`).
    for entry in &game.score_txt {
        if entry.age > 60 {
            let (gx, gy) = b2px(entry.cell);
            let (x, y) = px2rpx(int(gx), int(gy), game.spin.var);
            let (x, y) = (to_int(x) - 4, to_int(y) - 4);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                print(fb, &entry.text, x + dx, y + dy, 7);
            }
            print(fb, &entry.text, x, y, entry.colour);
        }
    }

    score_card(fb, game);
    nxt_card(fb, game);

    if game.round_clear {
        draw_banner(fb, -2, 29, 130, 39, 12, 7);
        cprint(fb, "round cleared!", 64, 32, 7);
    }
}

/// The cart draws `spr(21)` at the aim point plus three 2x2 dots along the line.
fn draw_aimer(fb: &mut Fb, game: &Game) {
    use crate::fixed::mul;
    use crate::p8math::{cos, sin};

    let (ax, ay) = game.gun.aim();
    let dir = (sin(game.gun.a), cos(game.gun.a));
    for d in DOTS {
        let (dx, dy) = (mul(dir.0, int(d)), mul(dir.1, int(d)));
        let (x, y) = (to_int(game.gun.x + dx), to_int(game.gun.y + dy));
        fb.rectfill(x - 1, y - 1, x, y, 7);
    }
    fb.spr(&sprites::cart(), 21, to_int(ax) - 4, to_int(ay) - 4);
}

/// cart `score_card()`: the score line and the shot pips, sliding in from the top.
fn score_card(fb: &mut Fb, game: &Game) {
    let x = 0;
    let y = to_int(game.score_card_y);
    draw_banner(fb, x - 2, y - 1, x + 130, y + 10, 6, 7);

    print(fb, &format!("score:{}0", game.score), x + 8, y + 2, 5);

    let sheet = sprites::cart();
    for i in 1..=4i32 {
        let sx = 121 - i * 9;
        if i <= game.shot_cnt {
            fb.spr(&sheet, 8, sx, y + 1); // a full pip
        } else {
            fb.spr(&sheet, 24, sx, y + 1); // an empty pip
        }
    }

    if !game.col_pool.is_empty() {
        print(fb, &format!("x{}", game.col_pool.len()), x + 64, y + 2, 5);
    }
}

/// cart `nxt_card()`: the conveyor's next two gumballs.
fn nxt_card(fb: &mut Fb, game: &Game) {
    let x = 82;
    let y = to_int(game.nxt_card_y);
    draw_banner(fb, x - 2, y - 1, x + 130, y + 10, 6, 7);
    print(fb, "next:", x + 1, y + 2, 5);
    draw_bub(fb, x + 25, y + 5, game.nxt_col, false);
    draw_bub(fb, x + 34, y + 5, game.nxt_nxt_col, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::grid::CENTER;

    fn game_with(board: Board) -> Game {
        let mut game = Game::new(1);
        game.board = board;
        game.score_card_y = int(12);
        game.nxt_card_y = int(108);
        game
    }

    fn count(fb: &Fb, c: u8) -> usize {
        fb.px.iter().filter(|&&p| p == c).count()
    }

    #[test]
    fn the_background_floor_and_map_are_drawn() {
        let mut fb = Fb::new();
        draw_game(&mut fb, &game_with(Board::new()));
        assert_ne!(fb.pget(4, 4), 6, "the map's border has been drawn");
        // The map is 128x16 tiles, so it covers the whole screen: the play field
        // is the map, and `cls(6)` only shows through where a tile is 0.
        assert!(fb.px.contains(&13), "the floor colour is on screen");
        assert_ne!(fb.pget(0, 0), 6, "the top-left corner is a map tile");
    }

    #[test]
    fn the_anchor_hangs_at_the_screen_centre_in_its_own_palette() {
        let mut fb = Fb::new();
        draw_game(
            &mut fb,
            &game_with(Board(vec![(CENTER, 8)].into_iter().collect())),
        );
        // The anchor is drawn as `palswap(7,15,4)`; sprite 2's row 0 col 4 is
        // shade 9 (now 7) and row 5 col 0 is shade 4 (now 15).
        assert_eq!(fb.pget(64, 60), 7, "the highlight");
        assert_eq!(fb.pget(60, 65), 15, "the mid shade is peach");
    }

    #[test]
    fn a_gumball_is_the_sprite_not_a_disc() {
        // Sprite 2 has transparent corners, so the pixel at the sprite's corner
        // shows whatever is behind it, and the centre is drawn.
        let mut fb = Fb::new();
        draw_game(
            &mut fb,
            &game_with(Board(vec![(CENTER, 8), (263, 1)].into_iter().collect())),
        );
        assert_eq!(fb.pget(68, 60), 13, "the sprite's corner is transparent");
        assert_ne!(fb.pget(72, 64), 13, "but its centre is drawn");
    }

    #[test]
    fn an_exploding_cell_flickers_white() {
        let mut normal = Fb::new();
        draw_bub(&mut normal, 64, 64, 305, false); // 305 % 130 = 45: not white
        let mut flashing = Fb::new();
        draw_bub(&mut flashing, 64, 64, 380, false); // 380 % 130 = 120: white
        assert_ne!(normal.px, flashing.px, "the flicker frame differs");
        assert!(count(&flashing, 7) > 0, "the white highlight is used");
    }

    #[test]
    fn the_tries_are_drawn_as_pips() {
        let mut fb = Fb::new();
        let mut game = game_with(Board::new());
        game.shot_cnt = 0;
        draw_game(&mut fb, &game);
        let empty: usize = (0..128)
            .flat_map(|x| (0..128).map(move |y| (x, y)))
            .filter(|&(x, y)| (112..=120).contains(&x) && (13..=20).contains(&y))
            .filter(|&(x, y)| fb.pget(x, y) == 1)
            .count();
        assert!(empty > 0, "empty pips use colour 1");
    }

    #[test]
    fn the_round_clear_banner_is_drawn_only_while_clearing() {
        let mut game = game_with(Board::new());
        game.score_card_y = int(-28);
        game.nxt_card_y = int(136);
        let mut plain = Fb::new();
        draw_game(&mut plain, &game);
        let before = count(&plain, 12);

        game.round_clear = true;
        let mut banner = Fb::new();
        draw_game(&mut banner, &game);
        assert!(count(&banner, 12) > before, "the banner adds colour 12");
    }

    #[test]
    fn the_fade_ladder_runs_from_identity_to_black() {
        let mut fb = Fb::new();
        fade_scr(&mut fb, 0.0);
        assert_eq!(fb.shown(7), 7, "column 1 is the identity");

        let mut fb = Fb::new();
        fade_scr(&mut fb, 0.5); // fi = 4, column 4 of colour 7's row is 5
        assert_eq!(fb.shown(7), 5);

        let mut fb = Fb::new();
        fade_scr(&mut fb, 0.9); // fi = 7, the last column is 0
        assert_eq!(fb.shown(7), 0);
        assert_eq!(fb.shown(1), 0);

        // fa = 1 asks for the cart's ninth column, which is nil: no remapping.
        let mut fb = Fb::new();
        fade_scr(&mut fb, 1.0);
        assert_eq!(fb.shown(7), 7);
    }

    #[test]
    fn the_map_draws_tiles_from_the_checked_in_data() {
        let mut fb = Fb::new();
        draw_map(&mut fb, 0, 0, 0, 0, 4, 4);
        assert!(fb.px.iter().any(|&p| p != 0), "something was drawn");
    }

    #[test]
    fn a_score_of_zero_is_still_drawn_as_text() {
        let mut fb = Fb::new();
        let game = game_with(Board::new());
        draw_game(&mut fb, &game);
        // "score:00" starts at x = 8, y = 14 in colour 5.
        let lit = (8..60)
            .flat_map(|x| (14..20).map(move |y| (x, y)))
            .any(|(x, y)| fb.pget(x, y) == 5);
        assert!(lit, "the score text is drawn");
    }
}
