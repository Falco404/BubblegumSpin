# Project Status (updated 2026-11-28)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M8 merged; `v0.2.0` being cut.** Windows, Linux and Android all confirmed working:
the phone runs it, audio works, the pad is 2.1x bigger, back quits. 259 tests green,
`clippy -D warnings` and `fmt --check` clean.

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
**Tag `v0.2.0`** once this version bump is merged: the tag builds Windows, Linux and the
APK and attaches all three to the release. Then the open work is the music (still muted)
and, if reinstall-in-place ever matters, a stable Android signing key.

## Gotchas
`docs/gotchas.md` (77/80). Three that must never slip: never commit `assets/upstream/`,
PICO-8 trig is y-inverted, and the APK's bin/.so/package names must be one word.

## Open questions for the human
1. **Music** is still muted (`BGS_MUSIC=1` to hear it); it needs another pass, not a fix.
2. **Volume keys** do nothing on the phone: miniquad's Java `onKey` swallows every key.
   Options were leave it, an in-game volume row, or vendoring a patched miniquad.
3. Whether the thumb wants a **500 px pad** — 42% of the width is the current cap and
   there is room above 453.
