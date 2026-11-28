# Project Status (updated 2026-11-28)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M7 merged; M8 in review** (branch `hermes/port-m8`). 240 tests green, `clippy -D
warnings` and `fmt --check` clean. `main` = `28fc2ce`. M8 turns the game into the whole
cart: title and how-to screens, the coin drop, the zoom into the board, the cascade and
the score screen, plus the input map.

## Where we are
- Milestones are issues #1–#8; issues and PRs share one number sequence.
- M1: fixed point, y-inverted trig, palette, framebuffer, `--bin shot`.
- M2: hex grid, generated level, board, matching/rooting/pool.
- M3: rotation, torque, gun/aimer, `draw_game`.
- M4: fire, bounce, collide, stick, spin; replay hash.
- M5: score, tries and return shots, rounds, game over, `save.dat`; three hashes.
- M6: sprites, map, Tom Thumb text, HUD cards and banners, `fade_scr`, five goldens.
- M7: `src/audio` synth and `sfx()`/`music()` ids; effects ship, music is muted.
- M8: `sim::scene` (Scene, `nav` menus, `Btnp`, `reset_scene`), the coin drop, the zoom,
  the score screen's dispense/chew/blow, `gfx::menus` with four more goldens, the window
  on the machine, and the input map (`Z` peeks).

## Next step
**Release check**: download this PR's Windows and Linux artifacts, run them on a clean
machine, and tag if they work (`hermes/release`).

## Gotchas
`docs/gotchas.md` (80/80). Three that must never slip: never commit `assets/upstream/`,
PICO-8 trig is y-inverted, and `draw_scene()` maps from cell (16,0).

## Open questions for the human
1. **Music** is still muted (`BGS_MUSIC=1` to hear it); it needs another pass, not a fix.
2. **The feel of a held X**: M8 made firing repeat like the cart's `btnp` (after 15
   frames, then every 4). Say if that reads as too fast on the keyboard.
3. **Windows**: only M4 has been run by you; M5–M8 are untested on it.
