# Gotchas

**Limit: 80 lines**, one line each; **delete the line when it stops being true** — the code wins over this file.

## The cart and the licence

- **Never commit `assets/upstream/`** — the cart is "No License"; it was committed by
  accident and the repo replaced. Art and audio come from the cart via
  `tools/p8_import.py` into `assets/*.bin`: derived data, like `consts.rs`.

## Fixed point and trig

- Everything is 16.16 (`Fix = i32`) and literals quantise (`0.96` → 62914): compare against the quantised constant.
- `sin`/`atan2` are **y-inverted** and **anticlockwise**: `sin(0.25) = -1`; a quarter
  turn sends `(72,64)` to `(64,56)` and leaves `(64,64)` alone.
- `px2rpx` is **not bit-exact** (`f64`, worst 324/65536 px); `mid` needs sorted bounds.

## Torque and aiming

- Torque `0.05` needs **110** frames to reach 0; aiming is asymmetric (left x=50, right x=77).
- `update_bubble` mutates its own copy: write the **whole struct** back, or a bounce flips a `dx` nobody stores.
- A shot down the centre gives **zero** torque, and the mass aims its own shots through it.

## Grid, board and rules

- Cell values are `u16` (`colour + 10*pop + 300*explode`, ~609): delete at `v > 600`,
  settled is `v < 10`, `keys()` skips `v >= 10` and the centre 262.
- `get_dest_cell` can be `nil` (the cart would index the board with it); we give the shot up.
- `neighbors()` wraps the last column into the next row and never bounds `y` upward — both change which gumballs fall.
- Layout: `b2px(i) = (x*8-16 (+4 odd rows), y*7-20)`, `B_R = 4`, centre 262 at (64,64); `src/consts.rs` comes from `tools/extract_level.py`.
- `check_over` **ceils** the rotated position, so centres live in `[8,121]`; 122 ends the game (the M5 issue's "124" is wrong).
- `f_bubs = bubs` on game over joins the loaded bubble to the cascade; `drop_bubs` divides
  by `flr(rnd(#f_bubs/20))`, which is 0 under 20 bubbles — guard it.

## Drawing

- `pal()`/`palt()` apply **when the pixel is drawn** (a later reset must not repaint); colour
  0 is transparent, which `cls`/`rectfill`/`pset` ignore and `spr`/`sspr` honour.
- `map()` is 128×16 tiles, i.e. the **whole 128×128 screen**: assert on map pixels, not the corner.
- **PICO-8's font is not in the cart** (it is in the ROM at `0x5600`); the port bundles Tom Thumb at PICO-8's 4 px advance.
- Golden frames are byte-compared PNGs in `tests/golden/`; re-record with
  `UPDATE_GOLDENS=1 cargo test --test screens` after an intended visual change.

## Audio

- `.p8` `__sfx__` is **not** the 68-byte RAM layout: a line is 84 bytes of packed five-nibble
  notes, and music channel bytes are **6-bit** sfx ids (`0x41..0x44` → 1..4, so mask `0x3f`).
  A note is `speed * 183/22050` s; pitch 0 is C0 = 65.4 Hz.
- **`macroquad`'s `audio` feature is opt-in** (`default = []`): without it `play_sound` links
  a dummy backend and only prints a warning. `tests/build_config.rs` pins it.
- PICO-8's instruments are **numbered** 0 triangle, 1 tilted saw, 2 saw, 3 square, 4 pulse,
  5 organ, 6 noise, 7 phaser; shifted, the organ is a buzz the tune hides (`..._stays_clear`).
- `sfx(18)` is **two** notes = 66 ms, not the M7 issue's "0.1–0.4 s"; music is baked lazily and looped.

## Save file

- The cart reads the hiscore then `dset(0,0)`, so it survives only if you beat it again.
  **Not ported**; `save.dat` sits beside the executable and corrupt files are ignored.

## Scenes and menus

- `draw_scene()` draws the map from **cell (16,0)**: from cell 0 the background covers the logo.
- PICO-8 draws a **negative `sspr` size mirrored** — the score screen's gumballs rely on it
  (`dw=(32/y)*3+2` goes negative above the machine), so clamp the size, not the sign.
- `btnp` is press, then every 4 frames after 15 held: firing and both menus use it, so a held `X` repeats.
- The how-to text's `y` is 1-based (`i*8+1-2`); its pink pass is a drop shadow one pixel *below*.

## Releases

- `gh release create` has no `--clobber` (that is an `upload` flag); a `workflow_dispatch` run needs the tag as an input.

## Android

- **The bin, the `.so` and `package_name`'s last segment must be one word.** cargo-quad-apk loads the
  package's last dot-part while the APK carries `lib<bin>.so`, so a hyphen ships an APK that dies at
  launch with `UnsatisfiedLinkError`; CI compares the two.
- `cargo quad-apk` 0.1.4 cannot read a `Cargo.lock` v4, shells out to the `dx` that build-tools 31+
  removed, and wants JDK 8's `rt.jar`; CI pins master by rev. The runner's NDK r27 ships only
  `llvm-*`, so `ar`/`ld`/`readelf` are symlinked and `-lunwind` comes from clang's dir. The APK is
  arm64, debug-signed: a reinstall needs an uninstall first.
