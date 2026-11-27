# BubblegumSpin

A native port of **Bubblegum Spin** (PICO-8, v1.1) by **Bee_Randon**. All game
design and original art, sound and code are his; this is a private,
non-commercial reimplementation. See `docs/decisions.md`.

Work in progress: a full game loop. Aim and fire (`←`/`→`/`↑`, `X`), the bubble
bounces, sticks and twists the mass, matches pop and score, four shots that clear
nothing let the mass shoot back, clearing the board starts the next round with one
more colour, and touching the wall ends the game with your hiscore saved (M1–M5).
The HUD, sprites, sound and menus are M6–M8. `docs/STATUS.md` says where we are and
`docs/gotchas.md` collects the traps.

## Build and run

Needs a stable Rust toolchain and, on Linux, the usual X11/GL/ALSA development
packages. Windows needs nothing extra.

```
cargo run                                       # 512x512 window, play a shot
cargo run --bin shot -- --scene game            # out/game.png, no display required
cargo test                                      # unit tests, no screen or GPU
```

See `docs/RUNNING.md` for details and `AGENTS.md` for how the project is run.
