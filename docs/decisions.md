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
| 2026-11-26 | Board is `BTreeMap<u16,u16>`; iteration always ascends by cell index. | Lua's `pairs()` order is unspecified; tests need one fixed order. Values are `u16` because pop states reach ~609 (`colour + 10*pop + 300`). In a rare equidistant tie a bubble may stick where the cart would not. |
| 2026-11-26 | RNG is seeded xorshift32, not PICO-8's PRNG. | PICO-8's generator is undocumented; a seeded one keeps tests deterministic. Gumball colour sequences will differ from the cart. |
| 2026-11-26 | CI is `.github/workflows/build.yml`: build + test on Windows and Linux, publish the binaries as Actions artifacts. | `AGENTS.md` requires downloadable Windows and Linux builds from CI, so the human can run each merge on his own PC. |
| 2026-11-27 | The GitHub repo was **replaced** (new repo, history rewritten to drop the cart files) instead of rewritten in place. | The cart had been committed in the first commits; a rewrite in place cannot remove GitHub's permanent `refs/pull/*`, which still pointed at those blobs. Issue numbers restarted: milestones are #1–#8. |
| 2026-11-27 | Firing is edge-triggered on `X`; PICO-8 `btnp`'s 15-frame auto-repeat is not modelled yet. | One bubble per press is what the cart does on a fresh press; the repeat only matters across a reload and belongs with the M8 input map. |
| 2026-11-27 | A round opens with `torque=.05` and a shot that clears nothing costs one of four tries, exactly as `new_round()`/`update_tries()` do. | Both change the feel on the first frames, so they are ported rather than tidied; the mass's return shot after four tries is M5.2. |
