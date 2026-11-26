# Decisions in force

Only decisions that still apply. Keep under 30 lines. Significant decisions
only: date, what, short why. When replaced, move the row to
docs/archive/decisions-history.md.

| Date | Decision | Why |
|---|---|---|
| 2026-11-26 | Full rewrite in Rust with PICO-8 16.16 fixed point (approach b), not PICOLOVE or embedded Lua. | Only a rewrite can be made bit-faithful: both emulator routes silently turn the physics into floats, and the spin is the game. |
| 2026-11-26 | `macroquad` 0.4 for window, input and audio. | Fewest dependencies, works on Windows and Linux now and gives the shortest path to Android; system libs already in the image. |
| 2026-11-26 | Render into our own 128×128 palette-indexed framebuffer, integer-scaled to the window (default 4× = 512×512). | Faithfulness, and it makes `pal`/`palt` and headless PNG screenshots trivial. |
| 2026-11-26 | **Private use only.** No public release, and the cart stays out of git (`assets/upstream/` is gitignored). | The cart is published "No License", so a port is a derivative work we may not distribute. Bee_Randon is credited in the README. |
| 2026-11-26 | Trig is quantised `f64` with PICO-8's y-inversion: `sin(a) = -sin(TAU*a)`, `atan2(dx,dy) = atan2_std(-dy,dx)/TAU` mod 1. | PICO-8 angles run anticlockwise in screenspace; textbook trig gives `nil` cells and a mirrored aim. Not a lookup table, so values may differ in the last 1/65536. |
| 2026-11-26 | Board is `BTreeMap<u16,u8>`; iteration always ascends by cell index. | Lua's `pairs()` order is unspecified; tests need one fixed order. In a rare equidistant tie a bubble may stick where the cart would not. |
| 2026-11-26 | RNG is seeded xorshift32, not PICO-8's PRNG. | PICO-8's generator is undocumented; a seeded one keeps tests deterministic. Gumball colour sequences will differ from the cart. |
