//! The hex grid the gumballs live on.
//!
//! The cart stores the board as a Lua table keyed by a flat cell index,
//! `y * 21 + x`, with odd rows shifted right by half a gumball. Two of the
//! cart's quirks are reproduced on purpose (see `docs/decisions.md`):
//!
//! * `neighbors()` checks `_x > 21`, so a cell in the last column of a row gets
//!   a neighbour that is really column 0 of the next row.
//! * there is no lower/upper `_y` check, so the bottom row can produce indices
//!   past the end of the level (525..); look them up in the board and they are
//!   simply absent, which is exactly what happens in Lua.

/// Cells per row.
pub const COLS: u16 = 21;
/// Rows in the play field.
pub const ROWS: u16 = 25;
/// Gumball radius in pixels; the cart's `b_r`, always 4.
pub const B_R: i32 = 4;
/// `board[262]` is the white anchor gumball at the screen centre (64,64).
pub const CENTER: u16 = 262;

/// cart `b2px(i)`: cell index -> pixel position of the gumball's centre.
#[inline]
pub fn b2px(i: u16) -> (i32, i32) {
    let y = (i / COLS) as i32;
    let x = (i % COLS) as i32;
    let mut offx = -16;
    let offy = -20;
    if y & 1 == 1 {
        offx += B_R;
    }
    (x * (B_R * 2) + offx, y * (B_R * 2 - 1) + offy)
}

/// cart `neighbors(i)`: the six surrounding cells, in the order
/// e, ne, nw, w, sw, se. `None` where the cart would produce `nil`.
///
/// This is not a clean hex neighbourhood: it keeps the cart's `_x > 21` check
/// and its missing upper `y` bound, because both change which cells count as
/// adjacent (and therefore which gumballs fall).
#[inline]
pub fn neighbors(i: u16) -> [Option<u16>; 6] {
    let x = (i % COLS) as i32;
    let y = (i / COLS) as i32;
    let offx = if y % 2 == 1 { 1 } else { 0 };
    let n2b = |x: i32, y: i32| -> Option<u16> {
        if x < 0 || x > 21 || y < 0 {
            return None;
        }
        Some((y * COLS as i32 + x) as u16)
    };
    [
        n2b(x + 1, y),
        n2b(x + offx, y - 1),
        n2b(x - 1 + offx, y - 1),
        n2b(x - 1, y),
        n2b(x - 1 + offx, y + 1),
        n2b(x + offx, y + 1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_is_64_64() {
        assert_eq!(b2px(CENTER), (64, 64));
    }

    #[test]
    fn even_row_offsets() {
        assert_eq!(b2px(176), (48, 36)); // row 8, col 8
    }

    #[test]
    fn odd_row_shifts_right() {
        // row 13 is odd: offx = -16 + 4 = -12, so x = 10*8 - 12 = 68, y = 13*7 - 20 = 71
        assert_eq!(b2px(283), (68, 71));
    }

    #[test]
    fn center_neighbors_are_the_six_around_it() {
        assert_eq!(
            neighbors(CENTER),
            [
                Some(263),
                Some(241),
                Some(240),
                Some(261),
                Some(282),
                Some(283)
            ]
        );
    }

    #[test]
    fn last_column_wraps_into_the_next_row_on_purpose() {
        // cell 230 is row 10, col 20; its "east" neighbour should not exist, but
        // the cart's `_x > 21` check lets it through as index 231 (row 11, col 0).
        assert_eq!(neighbors(230)[0], Some(231));
    }

    #[test]
    fn bottom_row_can_produce_indices_past_the_level() {
        // cell 514 is row 24, col 10; the cart has no upper y check, so the two
        // "south" neighbours land past the end of the level table.
        assert_eq!(neighbors(514)[4], Some(534)); // sw = 25*21 + 9
        assert_eq!(neighbors(514)[5], Some(535)); // se = 25*21 + 10
        assert_eq!(neighbors(514)[1], Some(493)); // ne = 23*21 + 10
    }

    #[test]
    fn negative_indices_are_none() {
        // Row 0 has no neighbours above it, and column 0 has no "west".
        let ns = neighbors(0);
        assert_eq!(ns[0], Some(1)); // e
        assert_eq!(ns[1], None); // ne (y - 1)
        assert_eq!(ns[2], None); // nw (y - 1)
        assert_eq!(ns[3], None); // w
        assert_eq!(ns[4], None); // sw (x - 1)
        assert_eq!(ns[5], Some(21)); // se = next row, col 0
    }
}
