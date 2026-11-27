# Project Status (updated 2026-11-27)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M4 merged; M5 in review** (PR #13, branch `hermes/port-m5`). 158 tests green,
`clippy -D warnings` and `fmt --check` clean. `main` = `3dd13d3`; the branch adds
the rules loop.

## Where we are
- Milestones are issues #1–#8; issues and PRs share one number sequence.
- M1 (#1): 16.16 fixed point, y-inverted trig, palette, framebuffer, `--bin shot`.
- M2 (#2): hex grid, generated level table, board + matching/rooting/pool.
- M3 (#3): `px2rpx`, torque + 0.96 decay, `calc_torque`, gun/aimer, `draw_game`.
- M4 (#4): fire, bounce, collide, stick, spin; pop animation; replay hash.
- M5 (#5): score + floating score text, tries and the mass's return shot, round
  clear / new round with one more colour, game-over cascade, fade/camera/shake
  arithmetic, `save.dat` hiscore; two more replay hashes (punishment, game over).

## Next step
**M6 (issue #6): art** — sprites, palette swaps, font, HUD, banners, fades. The HUD
is what makes M5's score, tries and hiscore visible; `score_txt` is already queued
for it, and `fade`/`camera` are computed but not yet drawn.

## Gotchas
`docs/gotchas.md` (78/80). Two that must never slip: never commit
`assets/upstream/`, and PICO-8 trig is y-inverted.

## Open questions for the human
1. **The HUD is M6.** The loop works (the mass shoots back, rounds advance, the board
   explodes at game over) but score, tries and hiscore are only in `save.dat` until
   the HUD lands.
2. **Windows**: you confirmed M4; M5's save file has not been run there.
3. `docs/archive/` still does not exist; nothing needs it yet.
