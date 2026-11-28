# BubblegumSpin

A native port of **Bubblegum Spin** (PICO-8, v1.1) by **Bee_Randon**. All game
design and original art, sound and code are his; this is a private,
non-commercial reimplementation. See `docs/decisions.md`.

Work in progress: a full game loop with the cart's own art — sprites, the HUD, the
banners and the fade, all from the cart's sprite sheet. Aim and fire (`←`/`→`/`↑`,
`X`), the bubble bounces, sticks and twists the mass, matches pop and score, four
shots that clear nothing let the mass shoot back, clearing the board starts the next
round with one more colour, and touching the wall ends the game with your hiscore
saved (M1–M6). Sound and the menus are M7–M8. `docs/STATUS.md` says where we are and
`docs/gotchas.md` collects the traps.

## Credits and licences

The game is **Bee_Randon**'s (PICO-8 cart, published "No License"), so this port is
private and non-commercial: the sprite sheet and map in `assets/*.bin` are extracted
from his cart by `tools/p8_import.py` and are not distributed.

Text is drawn with **Tom Thumb**, a public-domain 3x5 pixel font (Brian Swetland,
1999, 3-clause BSD; Robey Pointer's 2010 revisions, released CC0). PICO-8's own
font is not part of the cart — see `assets/FONT-LICENSE.txt` and `docs/gotchas.md`.

## Build and run

Needs a stable Rust toolchain and, on Linux, the usual X11/GL/ALSA development
packages. Windows needs nothing extra.

```
cargo run                                       # 512x512 window, play a shot
cargo run --bin shot -- --scene game            # out/game.png, no display required
cargo test                                      # unit tests, no screen or GPU
```

See `docs/RUNNING.md` for details and `AGENTS.md` for how the project is run.
