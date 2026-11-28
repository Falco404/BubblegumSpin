# Project Status (updated 2026-11-28)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M8 merged and `v0.1.0` released** (Windows, Linux, both confirmed on Windows).
**Android in review** (issue #18, branch `hermes/android`): the save path, on-screen
controls, the APK metadata and CI are in, the APK builds, and the first phone crash
(a 0×0 surface panicking the layout on frame one) is fixed with logcat logging added.
It runs on the phone (audio works); its pad is now 2.1x bigger, `O`/`X` 1.35x, and back
steps to the title then quits. A6 — the `v0.2.0` release — follows. 259 tests green.

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
- Android (#18): `Save::default_path` order (`BGS_SAVE` → `$HOME` → beside the exe),
  `ui::Layout` (whole-number picture scale, touch strip, `input()`), the on-screen pad,
  `[package.metadata.android]`, and the `android` CI job.

## Next step
**Android**: get the APK out of CI, install it on the phone, and fix what the thumb
finds (issue #18). Then `v0.2.0` with the APK attached.

## Gotchas
`docs/gotchas.md` (80/80). Three that must never slip: never commit `assets/upstream/`,
PICO-8 trig is y-inverted, and `draw_scene()` maps from cell (16,0).

## Open questions for the human
1. **Music** is still muted (`BGS_MUSIC=1` to hear it); it needs another pass, not a fix.
2. **The feel of a held X**: M8 made firing repeat like the cart's `btnp` (after 15
   frames, then every 4). Say if that reads as too fast on the keyboard.
3. **The phone's controls** are untested — the pad's size and the strip's 22% came from
   arithmetic, not a thumb.
