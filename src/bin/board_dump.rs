//! Print the initial level as ASCII: `.` empty, `W` wall, `8` anchor, `9` gumball.
//!
//! The output is the table the tests assert, so it is the quickest way to see
//! whether the generated `LEVEL` still matches the cart.

fn main() {
    for row in bubblegum_spin::consts::level_ascii() {
        println!("{row}");
    }
}
