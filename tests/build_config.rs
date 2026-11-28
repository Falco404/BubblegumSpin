//! A build-config contract, not a behaviour test.
//!
//! `macroquad` 0.4 declares `default = []`, so its **audio backend is opt-in**.
//! Build without `features = ["audio"]` and `play_sound` compiles fine, links the
//! dummy backend, and only prints `warn: macroquad's "audio" feature disabled` at
//! runtime — which is exactly how M7 shipped silent Windows and Linux binaries while
//! every unit test passed. There is no `cfg` for a dependency's features, so the
//! manifest is the only place this can be pinned.

#[test]
fn macroquad_is_built_with_its_audio_feature() {
    let manifest = include_str!("../Cargo.toml");
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("macroquad"))
        .expect("Cargo.toml must depend on macroquad");
    assert!(
        line.contains("features") && line.contains("\"audio\""),
        "macroquad's audio feature is opt-in and this line does not enable it: {line}"
    );
}
