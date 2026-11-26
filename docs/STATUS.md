# Project Status (updated 2026-11-26)

<!-- Keep under 40 lines. Rewrite, don't append. -->

## Now

Approach not chosen yet. The repo is prepared: the original cart lives outside git
in `assets/upstream/`, `reference/` keeps only the notes, no code yet.

## Last merged

- Game files moved out of `reference/` into gitignored `assets/upstream/`.

## In progress

- Planning only: approach comparison (PICOLOVE / Rust rewrite / Rust + embedded
  Lua) is written up and waiting for the owner's decision.

## Next steps (max 3)

1. Owner picks an approach.
2. Install Rust toolchain + system libs.
3. Milestone 1: fixed point, headless framebuffer, first PNG.

## Gotchas (max 8, one line each; delete when fixed)

- `assets/upstream/bubblegum_spin-4.p8.png` has no decodable payload (re-encoded);
  use the `.p8` text export next to it for gfx/map/sfx/music.
- Never `git add -f` anything under `assets/upstream/`; the cart is "No License".
- `docs/` files were stored with escaped markdown and CRLF; keep new edits plain.

## Open questions for the human

- Which approach (see the comparison)?
