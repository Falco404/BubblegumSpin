# BubblegumSpin

A native port of **Bubblegum Spin** (PICO-8, v1.1) by **Bee_Randon**. All game
design and original art, sound and code are his; this is a private,
non-commercial reimplementation. See `docs/decisions.md`.

The whole cart is playable: title screen and menu, the how-to pages, the coin drop,
the game itself (aim and fire with `←`/`→`/`↑` and `X`, hold `Z` to peek at the HUD),
the score screen with the machine dispensing your gumballs, and a hiscore that
survives a restart. It runs on Windows, Linux and Android; on a phone the controls are drawn on screen.
Background music is off by default — `BGS_MUSIC=1 cargo run` turns it on. `docs/STATUS.md` says where we are and `docs/gotchas.md` collects the
traps.

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
