//! Scene drawing: the cart's `draw_*` functions, as pure functions on a
//! framebuffer. No window, no audio, so every frame can be checked by a test.

use crate::fixed::{int, mul, to_int};
use crate::gfx::fb::Fb;
use crate::grid::{b2px, B_R};
use crate::p8math::{cos, sin};
use crate::sim::spin::px2rpx;
use crate::sim::Game;

/// The aimer's trail dots, at these pixel distances from the gun.
const DOTS: [i32; 3] = [5, 10, 15];

/// cart `draw_game()`, reduced to what M3 needs: background, floor, the board's
/// gumballs as filled circles (the real sprites arrive in M6), and the aimer.
pub fn draw_game(fb: &mut Fb, game: &Game) {
    fb.clear(6);
    fb.rectfill(0, 72, 128, 128, 13);

    for (&cell, &value) in game.board.0.iter() {
        let (gx, gy) = b2px(cell);
        let (x, y) = px2rpx(int(gx), int(gy), game.spin.var);
        fb.circfill(to_int(x), to_int(y), B_R, (value % 10) as u8);
    }

    draw_aimer(fb, game);
}

/// The cart draws `spr(21)` at the aim point plus three dots along the line.
fn draw_aimer(fb: &mut Fb, game: &Game) {
    let (ax, ay) = game.gun.aim();
    let dir = (sin(game.gun.a), cos(game.gun.a));
    for d in DOTS {
        let (dx, dy) = (mul(dir.0, int(d)), mul(dir.1, int(d)));
        fb.pset(to_int(game.gun.x + dx), to_int(game.gun.y + dy), 7);
    }
    fb.circfill(to_int(ax), to_int(ay), 2, 8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::fixed::from_f64;
    use crate::p8math::Rng;
    use crate::sim::gun::Gun;
    use crate::sim::spin::Spin;
    use crate::sim::Game;

    fn hand_board(pairs: &[(u16, u16)]) -> Board {
        Board(pairs.iter().copied().collect())
    }

    fn game_with(board: Board) -> Game {
        Game {
            board,
            spin: Spin::new(),
            gun: Gun::new(),
            rng: Rng::new(1),
        }
    }

    #[test]
    fn the_background_and_the_floor_are_drawn() {
        let mut fb = Fb::new();
        draw_game(&mut fb, &game_with(hand_board(&[(262, 8)])));
        assert_eq!(fb.pget(0, 0), 6, "cls(6) background");
        assert_eq!(fb.pget(0, 120), 13, "rectfill(0,72,128,128,13) floor");
        assert_eq!(fb.pget(127, 72), 13);
        assert_eq!(fb.pget(0, 71), 6);
    }

    #[test]
    fn the_anchor_is_drawn_at_the_screen_centre() {
        let mut fb = Fb::new();
        draw_game(&mut fb, &game_with(hand_board(&[(262, 8)])));
        assert_eq!(fb.pget(64, 64), 8);
        assert_eq!(fb.pget(64, 68), 8); // radius 4 disc
        assert_eq!(fb.pget(64, 69), 6);
    }

    #[test]
    fn cells_are_drawn_at_their_colour() {
        let mut fb = Fb::new();
        draw_game(&mut fb, &game_with(hand_board(&[(262, 8), (263, 3)])));
        assert_eq!(fb.pget(72, 64), 3); // cell 263 sits right of the centre
    }

    #[test]
    fn the_board_rotates_with_the_spin() {
        let mut game = game_with(hand_board(&[(262, 8), (263, 3)]));
        game.spin.var = from_f64(0.25);
        let mut fb = Fb::new();
        draw_game(&mut fb, &game);
        assert_eq!(fb.pget(64, 64), 8, "the anchor never moves");
        assert_eq!(fb.pget(64, 56), 3, "263 rotates from the right to above");
        assert_ne!(fb.pget(72, 64), 3, "and is no longer where it started");
    }

    #[test]
    fn animating_cells_are_drawn_as_their_colour() {
        // 305 = colour 5 with +300 (exploding); the cart draws v % 10.
        let mut fb = Fb::new();
        draw_game(&mut fb, &game_with(hand_board(&[(262, 8), (263, 305)])));
        assert_eq!(fb.pget(72, 64), 5);
    }

    #[test]
    fn the_aimer_is_drawn_above_the_gun() {
        let mut fb = Fb::new();
        draw_game(&mut fb, &game_with(hand_board(&[(262, 8)])));
        // Gun at (64,121) pointing up, aimer at (64,101).
        assert_eq!(fb.pget(64, 101), 8);
        // The three dots at 5, 10 and 15 px.
        assert_eq!(fb.pget(64, 116), 7);
        assert_eq!(fb.pget(64, 111), 7);
        assert_eq!(fb.pget(64, 106), 7);
    }
}
