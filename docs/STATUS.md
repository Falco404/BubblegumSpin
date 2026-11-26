# Project Status (updated 2026-11-26)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
**M1 done, in review** (PR on `hermes/port-m1`, issue #11). The crate builds,
`cargo test` is 36 green tests, `cargo run --bin shot` writes a 128x128 PNG with
no display. No game rules yet — those start in M2.

## Last merged
- #19 — decisions + status recorded, milestones filed as issues #11–#18.
- #9/#10 — cart moved to `assets/upstream/` (gitignored), `AGENTS.md` updated.

## In progress
- `hermes/port-m1`: fixed point, y-inverted trig, palette, framebuffer, window
  shell, `bin/shot`. CI YAML is proposed in the PR description only.

## Next steps (max 3)
1. Merge M1 after review; add the CI workflow (human).
2. M2 (issue #12): grid, level, board, matching/rooting — no screen.
3. M3 (issue #13): spin, aim, first on-screen board.

## Gotchas (max 8, one line each; delete when fixed)
- PICO-8 `sin`/`atan2` are y-inverted: `sin(0.25) = -1`, `atan2(0,-1) = 0.25`.
- The aim is **not** mirrored in the cart; a mirrored aim means our trig is wrong.
- `assets/upstream/*` is gitignored on purpose (No License) — `git status` must
  stay clean there.
- Cart literals like `0x.02` are 2/256, not 0.02; don't "clean up" constants.
- `cargo run` needs a display; the container has none, so visual proof comes from
  `cargo run --bin shot` + the PNG.
- `keys(board)` skips cell 262 (the white anchor); rooting depends on it.

## Open questions for the human
- Does `cargo run` open the 512x512 palette card on Windows? (Cannot test here.)
