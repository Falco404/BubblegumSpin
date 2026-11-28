# Project Status (updated 2026-11-28)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M8 merged and `v0.2.0` released** — Windows, Linux and Android, all three confirmed
on real hardware; the release's assets were downloaded back and hashed. 259 tests green,
`clippy -D warnings` and `fmt --check` clean. `main` = the #21 merge.

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
  `ui::Layout` (whole-number scale, touch strip, `input()`), the on-screen pad (2.1x),
  `[package.metadata.android]`, the `android` CI job, logcat panics, and `App::back()`.

## Next step
**The music.** It has been muted since M7: the notes, timing and volumes are the cart's,
the eight instrument *numbers* are right, but the tracks still do not sound like the cart.
`BGS_MUSIC=1` plays them. Also worth doing: a stable Android signing key (a repo secret)
so an APK updates in place, and the release workflow's fixed publish step has not run
since — the next tag is its first test.

## Gotchas
`docs/gotchas.md` (77/80). Three that must never slip: never commit `assets/upstream/`,
PICO-8 trig is y-inverted, and the APK's bin/.so/package names must be one word.

## Open questions for the human
1. **Music**: another listening pass, or drop the port of the tracks for good?
2. **Volume keys** do nothing on the phone (miniquad's Java `onKey` swallows every key);
   you said the buttons are not needed, so it is parked.
