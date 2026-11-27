# Project Status (updated 2026-11-27)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M5 merged; M6 in review** (PR #14, branch `hermes/port-m6`). 192 tests green,
`clippy -D warnings` and `fmt --check` clean. `main` = `89ee622`; the branch adds
the art layer.

## Where we are
- Milestones are issues #1–#8; issues and PRs share one number sequence.
- M1: fixed point, y-inverted trig, palette, framebuffer, `--bin shot`.
- M2: hex grid, generated level, board, matching/rooting/pool.
- M3: rotation, torque, gun/aimer, `draw_game`.
- M4: fire, bounce, collide, stick, spin; replay hash.
- M5: score, tries and return shots, rounds, game over, `save.dat`; three hashes.
- M6 (#6): the cart's sprite sheet and map (`tools/p8_import.py` → `assets/*.bin`),
  `spr`/`sspr`/`line`, `bubcol`/`palswap`, Tom Thumb text (`cprint`/`coprint`), the
  HUD cards and pips, the round-clear banner, `fade_scr`, and five byte-compared
  golden frames in `tests/golden/`.

## Next step
**M7 (issue #7): audio** — the 4-channel PICO-8 synth, sfx/music data and the exact
calls. M8 then owns the menus/tutorial screens and packaging.

## Gotchas
`docs/gotchas.md` (80/80). Two that must never slip: never commit
`assets/upstream/`, and PICO-8 trig is y-inverted.

## Open questions for the human
1. **The menu screens** (`draw_main`, `draw_how`, `draw_final`, `draw_coin`, and the
   gumball-machine animation) are listed under M6 but need scene state that M8 owns.
   I left them for M8 — say if you want them pulled forward.
2. **Font**: Tom Thumb, not PICO-8's (it is not in the cart). Swapping in a real
   PICO-8 dump later is one table.
3. **Windows**: M4 was confirmed by you; M5/M6 have not been run there.
