# Gotchas

**Limit: 80 lines.** One line each: **delete the line when it stops being true.**
`docs/STATUS.md` links here; the code wins if the two ever disagree.

## The cart and the licence

- **Never commit `assets/upstream/`** — the cart is "No License"; it was committed
  by accident early on and the repo had to be replaced to purge the history.
- Write cart details in `reference/NOTES.md` in your own words: behaviour, not code.

## Fixed point and trig

- Everything is 16.16 (`Fix = i32`) and literals quantise too: `0.0004` → 26,
  `0.96` → 62914. Compare against the quantised constant, never the decimal.
- PICO-8 `sin`/`atan2` are **y-inverted**: `sin(0.25) = -1`, `atan2(0,-1) = 0.25`
  (up). Textbook trig gives a mirrored aim and `nil` cells.
- Angles run **anticlockwise**: a quarter turn sends `(72,64)` to `(64,56)`, and
  `(64,64)` never moves.
- `px2rpx` is **not bit-exact** (`f64` quantised): worst error 324/65536 px at the
  identity, 512 over a round trip. Tests carry the measurement, not a guess.
- `mid(lo,v,hi)` clamps with sorted bounds; the cart tolerates `lo > hi`, we do not.

## Torque and aiming

- Torque `0.05` needs **110** frames to reach exactly 0, not the 60 the plan
  guessed; at 60 frames `torque = 271`, `var = 74_338`.
- Aiming is asymmetric by one pixel: 30 frames left → x = 50, right → x = 77.
- `new_round()` sets `torque=.05`, so every round opens with the mass turning.
  Do not zero it.
- `update_bubble` mutates its own copy: Rust must write the **whole struct** back or
  a bounce flips a `dx` nobody stores (five "bounces" in five frames).
- A shot straight down the centre gives **zero** torque, and the mass aims its own
  shots through the centre.

## Grid, board and rules

- Cell values are `u16` (`colour + 10*pop + 300*explode`, up to ~609); the cart
  deletes a popper only at `v > 600`. Settled means `v < 10`.
- `keys()` skips `v >= 10` and the centre `262` (the anchor).
- `get_dest_cell` can return `nil`; the cart would do `board[nil]=col` and error.
  The port gives the shot up instead.
- `neighbors()` wraps the last column into the next row (`x > 21`) and never bounds
  `y` upward: indexes above 524 read as absent. Both quirks are the cart's.
- Layout: `b2px(i) = (x*8 - 16 (+4 on odd rows), y*7 - 20)`, `B_R = 4`, 21×25,
  centre 262 at (64,64).
- `src/consts.rs` is **generated**: edit `tools/extract_level.py`, then run
  `python3 tools/extract_level.py --check`.
- `check_over` **ceils** the rotated position, so cell centres live in `[8,121]`:
  121 is safe, 122 ends the game (the M5 issue's "124" is wrong).
- `f_bubs = bubs` on game over: the gun's loaded bubble joins the cascade.
- `drop_bubs` divides by `flr(rnd(#f_bubs/20))`, which is 0 under 20 bubbles, and
  PICO-8's `x % 0` is 0 → one bubble a frame. Guard it or Rust panics.

## Save file

- The cart reads the hiscore then `dset(0,0)`, so it survives only if you beat it
  again. **Not ported**: a hiscore that can vanish is a bug.
- `save.dat` sits beside the executable (`hiscore=`/`completed=`); a missing or
  corrupt file is an empty save, never an error.

## Rendering and evidence

- `circfill` is `dx² + dy² <= r²`, matching PICO-8 at r = 0, 1, 4 only: a
  placeholder until the sprites land in M6.
- No display in the container: `cargo run --bin shot -- --scene game --out x.png`.

## Tooling and the repo

- Rust and the system libraries are in the image: **do not install anything.** If
  `cargo fetch` fails, stop and ask.
- CI (`.github/workflows/build.yml`) tests Windows and Linux and uploads both
  binaries as artifacts.
- The `gh` token is **fine-grained**: it pushes and opens PRs, but cannot create or
  delete repositories or change settings (ask the human).
- Issues and PRs **share one number sequence**: the milestones are issues #1–#8.
- `git filter-branch` leaves `refs/original/…`; blobs stay reachable until that ref
  is deleted **and** `git gc --prune=now` runs.
