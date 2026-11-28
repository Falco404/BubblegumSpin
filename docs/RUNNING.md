# Running and testing

Everything is plain `cargo`; there are no scripts and no environment variables.

## Prerequisites

* Rust stable (the container image has 1.100.0; `rust-toolchain.toml` pins the
  channel and the `rustfmt`/`clippy` components).
* Windows: nothing else.
* Linux: `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev libudev-dev
  pkg-config` (already in the project container).
* Background music is **off by default** (see the sources for why). `BGS_MUSIC=1 cargo run`
  on Linux/macOS, or `$env:BGS_MUSIC="1"; cargo run` in PowerShell, turns it back on.
* Debian/WSL with no display: `cargo test` and `cargo run --bin shot` still work,
  because the framebuffer is software. Only `cargo run` needs a screen.
* **Android builds happen on CI only.** This container has no Android SDK/NDK and
  nothing gets installed; the `android` job on the ubuntu runner has both.

## Commands

| What | Command | Output |
|---|---|---|
| Window (4x scale, 512x512) | `cargo run` | the title screen: press X to play |
| Headless screenshot | `cargo run --bin shot` | `out/m1.png` (128x128 RGB) |
| Board scene screenshot | `cargo run --bin shot -- --scene game --out out/game.png` | one frame |
| ... rotated a quarter turn | `cargo run --bin shot -- --scene game --spin 0.25 --out out/spin.png` | one frame |
| ... aimer turned | `cargo run --bin shot -- --scene game --aim 0.30 --out out/aim.png` | one frame |
| ... a bubble in flight | `cargo run --bin shot -- --scene game --frames 8 --fire 1 --out out/fired.png` | one frame, `--fire` is a 0-based frame index |
| ... the game-over cascade | `cargo run --bin shot -- --scene over --frames 14 --out out/over.png` | one frame, one gumball planted past the wall |
| Title screen | `cargo run --bin shot -- --scene main --frames 60 --out out/main.png` | one frame, after the 50-frame fade |
| How to play | `cargo run --bin shot -- --scene how --out out/how.png` | page one |
| Coin drop | `cargo run --bin shot -- --scene coin --frames 100 --out out/coin.png` | one frame, mid-drop |
| Score screen | `cargo run --bin shot -- --scene final --frames 500 --out out/final.png` | a 700-point score, menu up |
| Level as ASCII | `cargo run --bin board_dump` | 25 rows of `.` `W` `8` `9` |
| Golden frames | `cargo test --test screens` | byte-compares 9 frames with `tests/golden/` |
| Re-record goldens | `UPDATE_GOLDENS=1 cargo test --test screens` | rewrites `tests/golden/*.png` after an intended visual change |
| Re-extract the art | `python3 tools/p8_import.py` | `assets/{sprites,map,sfx,music}.bin` from the cart |
| Hear one sound | `cargo run --bin wav_dump -- --sfx 18 --out out/fire.wav` | a WAV you can play |
| Hear one track | `cargo run --bin wav_dump -- --music 16 --out out/round.wav` | ditto, the round theme |
| Tests | `cargo test` | `test result: ok. ...` |
| Lints / format | `cargo clippy --all-targets`, `cargo fmt --check` | no output = clean |
| Release build | `cargo build --release` | `target/release/bubblegum-spin[.exe]` |

`out/` is gitignored. Screenshots are the primary visual evidence in PRs: render
with `shot`, attach the PNG, and quote the size.

## Controls

| Key | Cart button | Does |
|---|---|---|
| `←`/`→` (A/D) | left/right | aim |
| `↑` (W) | up | return the aimer to upright; move the menu up |
| `↓` (S) | down | move the menu down |
| `X` (K) | X | fire — hold it and it repeats like `btnp`; in a menu, pick |
| `Z` (J) | O | hold to peek at the score, the next gumballs and your remaining misses |

The titles screen's menu is `↑`/`↓` then `X`; the score screen works the same way.

## Android

The APK is built by the `android` CI job on every push (arm64, minSdk 26, portrait,
fullscreen, debug-signed) and attached to a release by the `release` job on a `v*` tag.

```
gh run download --name bubblegum-spin-android --dir bgs-apk   # from the latest run
adb install -r bgs-apk/*.apk
adb logcat | grep -i bubblegum                               # the `save:` line shows the path
```

Without `adb`: copy the APK to the phone, tap it, and allow installs from that source.
The phone has the same six buttons on screen — a four-way pad bottom-left, `O` and `X`
on the right — drawn over the bottom strip of the window.

A desktop window can show the same strip, with the mouse driving it, before an APK
exists:

```
set BGS_TOUCH=1 && cargo run        # PowerShell;  BGS_TOUCH=1 cargo run  on Linux
```

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

## Audio

The synth is in `src/audio`: it renders a sound effect or music track to samples and
`to_wav` wraps them in 16-bit mono PCM, which `main.rs` hands to `macroquad` the first
time the game asks for that id. No device is needed for any of it, so `cargo test` and
`wav_dump` work headless.

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
