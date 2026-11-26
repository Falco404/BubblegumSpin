# Running and testing

Everything is plain `cargo`; there are no scripts and no environment variables.

## Prerequisites

* Rust stable (the container image has 1.100.0; `rust-toolchain.toml` pins the
  channel and the `rustfmt`/`clippy` components).
* Windows: nothing else.
* Linux: `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev libudev-dev
  pkg-config` (already in the project container).
* Debian/WSL with no display: `cargo test` and `cargo run --bin shot` still work,
  because the framebuffer is software. Only `cargo run` needs a screen.

## Commands

| What | Command | Output |
|---|---|---|
| Window (4x scale, 512x512) | `cargo run` | palette test card |
| Headless screenshot | `cargo run --bin shot` | `out/m1.png` (128x128 RGB) |
| Tests | `cargo test` | `test result: ok. ...` |
| Lints / format | `cargo clippy --all-targets`, `cargo fmt --check` | no output = clean |
| Release build | `cargo build --release` | `target/release/bubblegum-spin[.exe]` |

`out/` is gitignored. Screenshots are the primary visual evidence in PRs: render
with `shot`, attach the PNG, and quote the size.

## Windows quick start

```
git clone https://github.com/Falco404/BubblegumSpin
cd BubblegumSpin
cargo run
```

The upstream cart in `assets/upstream/` is gitignored and is not needed to build
or test anything in M1.
