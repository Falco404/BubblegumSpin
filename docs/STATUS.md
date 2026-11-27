# Project Status (updated 2026-11-27)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M3 merged, M4 not started.** 106 tests green, CI green on Windows and Linux.
`main` = `ec78986`. Nothing is half-finished: no open PR, no dirty tree.

## Where we are
- The repo was **replaced on 2026-11-27** to purge the cart from history (it had
  been committed by accident at the start). Milestones are issues **#1–#8**, and
  issues and PRs share one number counter, so the next PR is **#11**.
- M1 (#1): 16.16 fixed point, y-inverted trig, palette, framebuffer, `--bin shot`.
- M2 (#2): hex grid, generated level table, board + matching/rooting/pool.
- M3 (#3): `px2rpx` rotation, torque + 0.96 decay, `calc_torque`, gun/aimer,
  `draw_game`. `cargo run` shows the board; `S` spins the mass (debug only).

## Next step
**M4 (issue #4): the bubble** — fire, bounce, collide, stick, spin. It is the first
playable shot, and the first milestone since M2 that touches game feel, so read
`docs/gotchas.md` first: trig inversion, `u16` cell values and the 110-frame torque
decay all bite here.

## Gotchas
Live in **`docs/gotchas.md`** (80-line cap), now listed in the `AGENTS.md` docs map.
Two that must never slip: never commit `assets/upstream/`, and PICO-8 trig is
y-inverted.

## Open questions for the human
1. **Windows**: the window has still never been run there. Linux is confirmed on
   Apollo and CI builds the Windows artifact on every merge, but nobody has run it.
2. **`docs/archive/`** is referenced by `AGENTS.md` and `gotchas.md` but does not
   exist yet — add it with a one-line README, or wait until something is archived?
