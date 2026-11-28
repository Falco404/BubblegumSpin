//! Dump one sound effect or music track to a WAV, so the synth can be checked by
//! ear (or by any decoder) without opening the game.
//!
//! ```text
//! cargo run --bin wav_dump -- --sfx 18 --out out/sfx18.wav
//! cargo run --bin wav_dump -- --music 16 --out out/music16.wav
//! ```

use bubblegum_spin::audio::{render_music, render_sfx, to_wav, Sfx, SAMPLE_RATE};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sfx: Option<u8> = None;
    let mut music: Option<u8> = None;
    let mut out = String::new();

    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--sfx" => sfx = Some(value()?.parse()?),
            "--music" => music = Some(value()?.parse()?),
            "--out" => out = value()?,
            other => return Err(format!("unknown flag: {other}").into()),
        }
    }

    let samples = match (sfx, music) {
        (Some(id), None) => {
            if out.is_empty() {
                out = format!("out/sfx{id}.wav");
            }
            render_sfx(&Sfx::from_id(id), SAMPLE_RATE)
        }
        (None, Some(id)) => {
            if out.is_empty() {
                out = format!("out/music{id}.wav");
            }
            render_music(id, SAMPLE_RATE)
        }
        _ => return Err("give exactly one of --sfx <id> or --music <id>".into()),
    };

    let seconds = samples.len() as f32 / SAMPLE_RATE as f32;
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let wav = to_wav(&samples, SAMPLE_RATE);

    if let Some(dir) = std::path::Path::new(&out).parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&out, &wav)?;
    println!(
        "wrote {out}: {:.3} s, {} samples, peak {:.3}, {} bytes",
        seconds,
        samples.len(),
        peak,
        wav.len()
    );
    Ok(())
}
