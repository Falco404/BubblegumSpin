# Project Status (updated 2026-11-27)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M3 merged.** Rotation, torque, `calc_torque`, the aimer and the board on screen
work; 106 tests green. No bubble to fire yet (M4) and no sprites yet (M6), so
gumballs are still filled circles.

## Where we are
- The repo was **replaced on 2026-11-27**: history rewritten to drop the cart
  files, so PR numbers restart and the milestones are now issues **#1–#8**
  (M1–#1, M2–#2, M3–#3 closed; M4–#4 … M8–#8 open).
- Done: M1 fixed point, y-inverted trig, palette, framebuffer, `bin/shot`;
  M2 grid, generated level, board; M3 spin, aim, `draw_game`.
- CI builds Windows + Linux artifacts on every merge to `main`.

## Next steps (max 3)
1. Run the window build on Windows — still never tried there (Linux confirmed).
2. M4 (issue #4): fire, bounce, collide, stick, spin — the first playable shot.
3. M5 (issue #5): score, misses, rounds, game over, hiscore file.

## Gotchas
Live in **`docs/gotchas.md`** — read it before touching code. Two that must never
slip: never commit `assets/upstream/`, and PICO-8 trig is y-inverted.

## Open questions for the human
- Windows: does the window open? Linux is confirmed on Apollo.
