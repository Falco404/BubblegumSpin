# Project Status (updated 2026-11-28)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1–M6 merged; M7 in review** (PR #15, branch `hermes/port-m7`). 216 tests green,
`clippy -D warnings` and `fmt --check` clean. `main` = `47aa4a6`; the branch adds the
synth, the sfx/music data and the exact `sfx()`/`music()` calls.

Audio is now **effects only**: music ships muted because the ported tracks still do
not sound like the cart after two fixes (the `macroquad` feature, the shifted
instrument table). `BGS_MUSIC=1` brings the music back for a listen.

## Where we are
- Milestones are issues #1–#8; issues and PRs share one number sequence.
- M1: fixed point, y-inverted trig, palette, framebuffer, `--bin shot`.
- M2: hex grid, generated level, board, matching/rooting/pool.
- M3: rotation, torque, gun/aimer, `draw_game`.
- M4: fire, bounce, collide, stick, spin; replay hash.
- M5: score, tries and return shots, rounds, game over, `save.dat`; three hashes.
- M6: sprites, map, Tom Thumb text, HUD cards and banners, `fade_scr`, five goldens.
- M7 (#7): `src/audio` — 64-pitch table, eight waveforms, note effects (slide,
  vibrato, drop, fade), `sfx.bin`/`music.bin` from `tools/p8_import.py`, WAV baking,
  and `Game` queueing exactly the cart's ids (8, 12, 18, 19, 21; music 16, 32, 34).

## Next step
**M8 (issue #8): scenes, tutorial, packaging** — `draw_main`/`draw_how`/`draw_final`,
the coin start, the gumball-machine animation, the score screen, and a release build
with the data files bundled.

## Gotchas
`docs/gotchas.md` (80/80). Two that must never slip: never commit
`assets/upstream/`, and PICO-8 trig is y-inverted.

## Open questions for the human
1. **Listen to it.** Pitch, timing, volume, effects and the instrument *numbers* are
   the cart's, but the eight shapes are approximations (PICO-8's synth is engine code).
   `cargo run --bin wav_dump -- --music 16 --out out/music16.wav` to hear the theme.
2. **Windows**: M4 was confirmed by you; M5–M7 have not been run there, and M7 is the
   first milestone with audio output.
