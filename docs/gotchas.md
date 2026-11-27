# Gotchas

**Limit: 80 lines** — condense, never just append, and move stale material to
`docs/archive/`. Things that will bite you, **one line each; delete the line when
it stops being true.** Everything here was learned the hard way — usually from a
test that failed for a reason I had assumed away. `docs/STATUS.md` links here; the
code wins if the two ever disagree.

## The cart and the licence

- **Never commit `assets/upstream/`** — the cart is "No License". It was committed
  by accident in the first commits and the whole GitHub repo had to be replaced to
  purge it from history.
- Write cart details down in `reference/NOTES.md` in your own words. The port
  matches behaviour, it does not copy code.

## Fixed point and trig

- All game maths is 16.16 (`Fix = i32`), and PICO-8 literals quantise as well:
  `0.0004` → `26`, `0.96` → `62914`. Compare against the quantised constant.
- PICO-8 `sin`/`atan2` are **y-inverted**: `sin(0.25) = -1`, `atan2(0,-1) = 0.25`
  (up), `atan2(0,1) = 0.75` (down). Textbook trig gives a mirrored aim and `nil`.
- Angles run **anticlockwise** on screen: a quarter turn sends `(72,64)` to
  `(64,56)`; `(64,64)` never moves. A failing aim test means the trig is wrong.
- Trig is `f64` quantised to 16.16, so `px2rpx` is **not bit-exact**: worst measured
  error is 324/65536 px (identity), 256 (full turn), 512 (round trip). Tests carry
  the measurement next to the tolerance instead of pretending it is exact.
- `mid(lo,v,hi)` clamps with sorted bounds; the cart tolerates `lo > hi`, this port
  does not.

## Torque and aiming

- Torque from `0.05` needs **110** frames to reach exactly 0, not the 60 the plan
  guessed. At 60 frames: `torque = 271`, `var = 74_338`.
- Aiming is asymmetric by one pixel: 30 frames of left → x = 50, right → x = 77.
- `S` / space is a **debug** spin key, not a cart button — removed in M4, which
  replaced it with firing.
- The cart's `new_round()` sets `torque=.05`: every round opens with the mass
  turning, and the 0.96 decay takes 110 frames to stop it. Do not zero it.
- `update_bubble` mutates `dx`/`dy` on a local copy of the bubble; a Rust port must
  write the whole struct back or a wall bounce flips a value nobody stores (the
  symptom is five "bounces" in five frames). Pin the frame count, not just the
  bounce count.

## Grid and board

- Cell values are `u16`: they hold `colour + 10*pop + 300*explode` and reach ~609;
  the cart deletes a popper only at `v > 600`. Settled means `v < 10`.
- `keys()` skips `v >= 10` (animating cells) **and** the centre `262` (the anchor).
- `get_dest_cell` can return `nil` (`neighbors()` has no cell outward of an edge);
  the cart would do `board[nil]=col` and error. The port gives the shot up instead.
- `neighbors()` wraps the last column into the next row (`x > 21`) and never bounds
  `y` upward, so indexes above 524 simply read as absent. Both quirks are the cart's.
- Layout: `b2px(i) = (x*8 + (-16 + 4 if row is odd), y*7 - 20)`, `B_R = 4`,
  21 columns × 25 rows, centre cell 262 at pixel (64,64).
- `src/consts.rs` is **generated** by `tools/extract_level.py`: change the tool,
  then run `python3 tools/extract_level.py --check`.

## Rendering and evidence

- `circfill` is plain `dx² + dy² <= r²`. It matches PICO-8 at r = 0, 1 and 4 only,
  and is a placeholder until the real sprites arrive in M6.
- There is **no display** in the container: render frames with
  `cargo run --bin shot -- --scene game --out out/x.png` and inspect the PNG.

## Tooling and the repo

- Rust and the system libraries are already in the image: **do not install
  anything.** If `cargo fetch` fails, stop and ask.
- CI (`.github/workflows/build.yml`) builds and tests on Windows and Linux and
  uploads `bubblegum-spin-windows` / `bubblegum-spin-linux` artifacts.
- The `gh` token is **fine-grained**: it pushes, opens issues and PRs, but cannot
  create or delete repositories or change repository settings (that needs the
  Administration permission — ask the human).
- Issues and pull requests **share one number sequence** here: the milestones are
  issues #1–#8, so pull requests start at #9.
- `git filter-branch` leaves `refs/original/…` behind and the old blobs stay
  reachable until that ref is deleted **and** `git gc --prune=now` runs — a
  "rewritten" file can still be served from a repo that looks clean.
