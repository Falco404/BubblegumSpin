# Project Status (updated 2026-11-26)
<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now
Approach settled: full Rust rewrite with `macroquad` (see docs/decisions.md).
Milestones M1–M8 are filed as issues #11–#18; **no code written yet**. The plan
lives in `.hermes/plans/2026-11-25_201633-…md`; each issue carries its own task list.

## Last merged
- #9/#10 — repo tidy-up: cart moved to `assets/upstream/` (gitignored),
  `AGENTS.md` updated.

## In progress
- `hermes/docs-plan` — decisions + status recorded, milestone issues opened.
  No game code in this PR, on purpose.

## Next steps (max 3)
1. Human approves the milestone issues.
2. M1 (`hermes/port-m1`): crate, `fixed.rs`, `p8math.rs` (y-inverted trig),
   `gfx/fb.rs`, `bin/shot.rs`, plus the CI YAML proposed in the PR description.
3. M2: grid, level, board, matching/rooting (no screen).

## Gotchas (max 8, one line each; delete when fixed)
- PICO-8 `sin`/`atan2` are y-inverted: `sin(0.25) = -1`, `atan2(0,-1) = 0.25`.
- The aim is **not** mirrored in the cart; a mirrored aim means our trig is wrong.
- `assets/upstream/*` is gitignored on purpose (No License) — `git status` must
  stay clean there.
- Cart literals like `0x.02` are 2/256, not 0.02; don't "clean up" constants.
- `keys(board)` skips cell 262 (the white anchor); rooting depends on it.

## Open questions for the human
- None. (Toolchain: Rust 1.100.0 is in the image; we install nothing.)
