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
| Window (4x scale, 512x512) | `cargo run` | the board scene |
| Headless screenshot | `cargo run --bin shot` | `out/m1.png` (128x128 RGB) |
| Board scene screenshot | `cargo run --bin shot -- --scene game --out out/game.png` | one frame |
| ... rotated a quarter turn | `cargo run --bin shot -- --scene game --spin 0.25 --out out/spin.png` | one frame |
| ... aimer turned | `cargo run --bin shot -- --scene game --aim 0.30 --out out/aim.png` | one frame |
| ... a bubble in flight | `cargo run --bin shot -- --scene game --frames 8 --fire 1 --out out/fired.png` | one frame, `--fire` is a 0-based frame index |
| ... the game-over cascade | `cargo run --bin shot -- --scene over --frames 14 --out out/over.png` | one frame, one gumball planted past the wall |
| Level as ASCII | `cargo run --bin board_dump` | 25 rows of `.` `W` `8` `9` |
| Golden frames | `cargo test --test screens` | byte-compares 5 frames with `tests/golden/` |
| Re-record goldens | `UPDATE_GOLDENS=1 cargo test --test screens` | rewrites `tests/golden/*.png` after an intended visual change |
| Re-extract the art | `python3 tools/p8_import.py` | `assets/sprites.bin`, `assets/map.bin` from the cart |
| Tests | `cargo test` | `test result: ok. ...` |
| Lints / format | `cargo clippy --all-targets`, `cargo fmt --check` | no output = clean |
| Release build | `cargo build --release` | `target/release/bubblegum-spin[.exe]` |

`out/` is gitignored. Screenshots are the primary visual evidence in PRs: render
with `shot`, attach the PNG, and quote the size.

## Generated code

`src/consts.rs` is generated from the cart, not hand-written:

```
python3 tools/extract_level.py           # rewrite src/consts.rs from the cart
python3 tools/extract_level.py --check    # exit 1 if it is stale
```

It needs `assets/upstream/bubblegum_spin.p8` (or the `_code.txt` dump). The
generated file is rustfmt-clean as produced, so `cargo fmt --check` stays happy
without reformatting it.

## Windows quick start

```
git clone https://github.com/Falco404/BubblegumSpin
cd BubblegumSpin
cargo run
```

## Controls (M4)

| Key | What it does |
|---|---|
| `←` / `A` | aim left |
| `→` / `D` | aim right |
| `↑` / `W` | snap the aimer back to vertical |
| `X` / `K` | fire (the cart's X button). One bubble per press; the press edge is read once per 60 Hz step. |

The debug spin key from M3 (`S` / space) is gone: the mass now spins because you
hit it.

## Save file

`save.dat` sits next to the executable (`target/debug/` under `cargo run`) and holds
`hiscore=` and `completed=`. Delete the file to reset the hiscore. A missing or
corrupt file is treated as an empty save, never an error.

The upstream cart in `assets/upstream/` is gitignored and is not needed to build
or test anything in M1.

## Downloading a build instead of compiling

`.github/workflows/build.yml` runs on every push to `main` and on every pull
request, on `windows-latest` and `ubuntu-latest`. To get a binary:

1. Open the repo's **Actions** tab and pick the newest `build` run.
2. At the bottom of the run page, download the **Artifacts**:
   `bubblegum-spin-windows` (a zip containing `bubblegum-spin.exe`) or
   `bubblegum-spin-linux` (`bubblegum-spin`).
3. Unzip and run the binary. Until M6 the game loads no data files, so the bare
   executable is enough; after that a packaged release (M8) is the way.

Artifacts need a GitHub login to download and expire after 90 days.
