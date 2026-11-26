# BubblegumSpin

A native port of **Bubblegum Spin** (PICO-8, v1.1) by **Bee_Randon**. All game
design and original art, sound and code are his; this is a private,
non-commercial reimplementation. See `docs/decisions.md`.

Work in progress: right now it builds, draws a 128x128 PICO-8 palette test card
and can write that card to a PNG with no window at all. The game itself does not
exist yet; `docs/STATUS.md` says where we are.

## Build and run

Needs a stable Rust toolchain and, on Linux, the usual X11/GL/ALSA development
packages. Windows needs nothing extra.

```
cargo run                 # 512x512 window, palette test card
cargo run --bin shot      # writes out/m1.png, no display required
cargo test                # unit tests, no screen or GPU
```

See `docs/RUNNING.md` for details and `AGENTS.md` for how the project is run.
