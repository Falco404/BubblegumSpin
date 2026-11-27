# Project Status (updated 2026-11-27)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M3 merged; M4 in review** (PR #12, branch `hermes/port-m4`). 128 tests green,
`clippy -D warnings` and `fmt --check` clean. `main` = `c743a33`; the branch adds
the fired bubble.

## Where we are
- Milestones are issues #1–#8; issues and PRs share one number sequence.
- M1 (#1): 16.16 fixed point, y-inverted trig, palette, framebuffer, `--bin shot`.
- M2 (#2): hex grid, generated level table, board + matching/rooting/pool.
- M3 (#3): `px2rpx`, torque + 0.96 decay, `calc_torque`, gun/aimer, `draw_game`.
- M4 (#4): `fire`, `update_bubble` (walls, 5-bounce give-up), `overlapping`,
  `get_dest_cell`, `place_bubble`, `reset_bubble`, `update_tries`,
  `update_falling`, the pop animation (`Board::advance_animations`); `X` fires;
  replay hash in `tests/replay.rs`.

## Next step
**M5 (issue #5): rules** — score, misses/punishments, rounds, game over, save file.
Start with the board turn (`fire_bubs`): M4 ends the player's turn after four shots
that clear nothing and then *waits*, because the return shot is M5.2.

## Gotchas
Live in **`docs/gotchas.md`** (80-line cap). Two that must never slip: never commit
`assets/upstream/`, and PICO-8 trig is y-inverted.

## Open questions for the human
1. **Windows**: M4's firing has not been run there (M1–M3 only). The CI artifact is
   the way to check it.
2. **`docs/archive/`** is referenced by `AGENTS.md` but does not exist yet.
3. **Round-start spin**: the cart's `new_round()` sets `torque=.05`, so the mass is
   always turning when a round opens. Ported because the cart does it — say if it
   feels wrong and I will turn it into a decision row instead.
