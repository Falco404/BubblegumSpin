//! Audio: a PICO-8-style 4-channel synth, and the sfx/music data baked to WAV.
//!
//! PICO-8's own synth is part of its engine, not the cart, so the waveforms here are
//! close approximations of its eight shapes; the **pitch, timing, volume and effect**
//! maths is the real thing (`speed * 183/22050` seconds a note, 64 semitones from
//! C0 = 65.4 Hz). See `docs/gotchas.md`.
//!
//! Nothing in the simulation plays audio: `Game` queues sfx/music ids and `main.rs`
//! drains the queue. This module turns an id into samples and a WAV, which is why it
//! can be tested with no device attached.

pub const SAMPLE_RATE: u32 = 44_100;
/// PICO-8's tick rate; a note's `speed` is a multiple of `183 / 22050` seconds.
pub const TICKS_PER_SECOND: f32 = 22_050.0;
pub const TICKS_PER_NOTE: f32 = 183.0;
/// C0, the frequency of pitch 0 (PICO-8's octave numbering, three below A440's).
pub const C0_HZ: f32 = 65.406_39;

/// Sound effect data: 64 effects of 68 bytes, PICO-8's RAM layout.
pub static SFX: &[u8] = include_bytes!("../../assets/sfx.bin");
/// Music patterns: 64 patterns of four channel bytes (an sfx id per channel).
pub static MUSIC: &[u8] = include_bytes!("../../assets/music.bin");
/// One flag byte per pattern: bit 0 loop start, bit 1 loop end, bit 2 stop.
pub static MUSIC_FLAGS: &[u8] = include_bytes!("../../assets/music_flags.bin");

pub const SFX_COUNT: usize = 64;
pub const SFX_SIZE: usize = 68;
pub const CHANNELS: usize = 4;

/// One tracker step.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Note {
    pub pitch: u8,
    pub wave: u8,
    pub volume: u8,
    pub effect: u8,
}

/// One sound effect / music pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sfx {
    pub notes: [Note; 32],
    pub mode: u8,
    pub speed: u8,
    pub loop_start: u8,
    pub loop_end: u8,
}

impl Sfx {
    /// Decode one 68-byte effect from PICO-8's RAM layout.
    ///
    /// Notes are little-endian 16-bit words: `ww pppppp` / `c eee vvv w`, then the
    /// four header bytes `mode, speed, loop start, loop end`.
    pub fn parse(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), SFX_SIZE, "an sfx is 68 bytes");
        let mut notes = [Note::default(); 32];
        for (i, note) in notes.iter_mut().enumerate() {
            let lsb = bytes[i * 2];
            let msb = bytes[i * 2 + 1];
            note.pitch = lsb & 0x3f;
            note.wave = ((lsb >> 6) & 3) | ((msb & 1) << 2) | (((msb >> 7) & 1) << 3);
            note.volume = (msb >> 1) & 7;
            note.effect = (msb >> 4) & 7;
        }
        Self {
            notes,
            mode: bytes[64],
            speed: bytes[65],
            loop_start: bytes[66],
            loop_end: bytes[67],
        }
    }

    /// The effect with this id, from the compiled-in cart data.
    pub fn from_id(id: u8) -> Self {
        let base = id as usize * SFX_SIZE;
        Self::parse(&SFX[base..base + SFX_SIZE])
    }

    /// Seconds per tracker step. `speed` is a multiple of `183/22050` s and a speed
    /// of 0 is treated as 1.
    pub fn note_seconds(&self) -> f32 {
        let speed = self.speed.max(1) as f32;
        speed * TICKS_PER_NOTE / TICKS_PER_SECOND
    }

    /// How many steps play: up to the last step that makes a sound.
    ///
    /// PICO-8's loop range is parsed but not applied here: a one-shot effect plays
    /// to its last sounding note, and music patterns are sequenced by their flags.
    pub fn length(&self) -> usize {
        self.notes
            .iter()
            .rposition(|n| n.volume > 0)
            .map(|i| i + 1)
            .unwrap_or(1)
    }

    /// Seconds this effect takes to play once.
    pub fn duration(&self) -> f32 {
        self.note_seconds() * self.length() as f32
    }

    /// Is this effect silent?
    pub fn is_silent(&self) -> bool {
        self.notes.iter().all(|n| n.volume == 0)
    }
}

/// The frequency of a pitch step. 0 is C0 (65.4 Hz) and each step is a semitone.
pub fn frequency(pitch: u8) -> f32 {
    frequency_f(pitch as f32)
}

/// The frequency of a fractional pitch step (used by slide and vibrato).
pub fn frequency_f(pitch: f32) -> f32 {
    C0_HZ * 2f32.powf(pitch / 12.0)
}

/// PICO-8's noise generator: a deterministic xorshift, so tests are repeatable.
#[derive(Debug, Clone, Copy)]
pub struct Noise(u32);

impl Noise {
    pub fn new(seed: u32) -> Self {
        Self(seed | 1)
    }

    /// One sample in -1..1. Named `sample` rather than `next` so it cannot be
    /// mistaken for an `Iterator`.
    pub fn sample(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// One sample of a waveform at a phase in 0..1.
///
/// The eight shapes are PICO-8's set by number: sine, triangle, tilted saw, saw,
/// square, pulse, organ, noise. The exact sample tables live in PICO-8's engine, so
/// these are shapes, not bit-identical copies.
pub fn wave_value(wave: u8, phase: f32, noise: &mut Noise) -> f32 {
    use std::f32::consts::TAU;
    match wave & 7 {
        0 => (TAU * phase).sin(),
        1 => {
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                3.0 - 4.0 * phase
            }
        }
        // Tilted saw: rises over three quarters of the period, falls over the last.
        2 => {
            if phase < 0.75 {
                phase / 0.75 * 2.0 - 1.0
            } else {
                1.0 - (phase - 0.75) / 0.25 * 2.0
            }
        }
        3 => 2.0 * phase - 1.0,
        4 => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        5 => {
            if phase < 0.25 {
                1.0
            } else {
                -1.0
            }
        }
        // Organ: a sine with a strong octave and a quiet two-octave harmonic.
        6 => {
            let (s1, s2, s4) = (
                (TAU * phase).sin(),
                (2.0 * TAU * phase).sin(),
                (4.0 * TAU * phase).sin(),
            );
            (s1 + 0.5 * s2 + 0.25 * s4) / 1.75
        }
        _ => noise.sample(),
    }
}

/// PICO-8's per-note amplitude envelope for effects 4 and 5.
fn envelope(effect: u8, t: f32) -> f32 {
    match effect {
        4 => t,       // fade in
        5 => 1.0 - t, // fade out
        _ => 1.0,
    }
}

/// The pitch of a note part-way through it: slide, vibrato and drop are pitch
/// effects, the rest are not.
fn bent_pitch(note: &Note, next_pitch: f32, t: f32) -> f32 {
    use std::f32::consts::TAU;
    let base = note.pitch as f32;
    match note.effect {
        1 => base + (next_pitch - base) * t, // slide to the next note
        2 => base + 0.5 * (TAU * 8.0 * t).sin(), // vibrato, half a semitone
        3 => base - 12.0 * t,                // drop an octave
        _ => base,
    }
}

/// Render one sound effect to mono samples.
///
/// A single voice never exceeds 1.0: the amplitude is `volume/7` times the effect
/// envelope, and every waveform is in -1..1.
pub fn render_sfx(sfx: &Sfx, sample_rate: u32) -> Vec<f32> {
    let steps = sfx.length();
    let step_samples = (sfx.note_seconds() * sample_rate as f32).round() as usize;
    let mut out = Vec::with_capacity(steps * step_samples);
    let mut noise = Noise::new(0x9e37_79b9);

    for i in 0..steps {
        let note = sfx.notes[i];
        if note.volume == 0 {
            out.resize(out.len() + step_samples, 0.0);
            continue;
        }
        let next_pitch = sfx
            .notes
            .get(i + 1)
            .map(|n| n.pitch as f32)
            .unwrap_or(note.pitch as f32);
        let mut phase = 0.0f32;
        for s in 0..step_samples {
            let t = if step_samples > 1 {
                s as f32 / (step_samples - 1) as f32
            } else {
                0.0
            };
            let pitch = bent_pitch(&note, next_pitch, t);
            let freq = frequency_f(pitch);
            let amp = (note.volume as f32 / 7.0) * envelope(note.effect, t);
            out.push(wave_value(note.wave, phase, &mut noise) * amp);
            phase += freq / sample_rate as f32;
            phase %= 1.0;
        }
    }
    anti_click(&mut out, sample_rate);
    out
}

/// A 2 ms ramp in and out, so a note that is cut off mid-cycle does not pop.
///
/// PICO-8's mixer applies its own anti-click envelope; this is the port's version
/// of it, and it is why every rendered effect starts and ends on zero.
fn anti_click(samples: &mut [f32], sample_rate: u32) {
    let ramp = ((sample_rate as f32 * 0.002) as usize).min(samples.len() / 2);
    if ramp == 0 {
        return;
    }
    for i in 0..ramp {
        let gain = i as f32 / ramp as f32;
        samples[i] *= gain;
        let last = samples.len() - 1 - i;
        samples[last] *= gain;
    }
}

/// The four channel sfx ids of one music pattern.
pub fn pattern(id: u8) -> [u8; CHANNELS] {
    let base = id as usize * CHANNELS;
    let mut out = [0u8; CHANNELS];
    out.copy_from_slice(&MUSIC[base..base + CHANNELS]);
    // The cart stores 6-bit sfx ids here; the top bits are not documented (pattern
    // 15 holds 0x41..0x44, which only makes sense as sfx 1..4), so mask them off.
    for channel in &mut out {
        *channel &= 0x3f;
    }
    out
}

/// Render a music track, starting at pattern `start`, to mono samples.
///
/// Patterns play in order until one is flagged as a loop end (bit 1) or stop (bit 2),
/// inclusive; `macroquad` then loops the buffer. The four channels are mixed and
/// divided by four so the sum cannot clip. A 2 ms ramp at each end keeps the loop
/// seam click-free, which is why the last sample equals the first.
pub fn render_music(start: u8, sample_rate: u32) -> Vec<f32> {
    const MAX_SECONDS: f32 = 60.0;
    let max_samples = (MAX_SECONDS * sample_rate as f32) as usize;

    let mut out: Vec<f32> = Vec::new();
    let mut id = start as usize;
    while id < SFX_COUNT && out.len() < max_samples {
        let channels = pattern(id as u8);
        let mut rendered: Vec<Vec<f32>> = Vec::with_capacity(CHANNELS);
        for &sfx_id in &channels {
            if sfx_id < SFX_COUNT as u8 {
                let sfx = Sfx::from_id(sfx_id);
                if !sfx.is_silent() {
                    rendered.push(render_sfx(&sfx, sample_rate));
                }
            }
        }
        let longest = rendered.iter().map(|c| c.len()).max().unwrap_or(0);
        if longest == 0 {
            // An empty pattern still takes one step of musical time.
            let step = (Sfx::from_id(channels[0]).note_seconds() * sample_rate as f32) as usize;
            out.resize(out.len() + step, 0.0);
        } else {
            for i in 0..longest {
                let mut sum = 0.0;
                for channel in &rendered {
                    sum += channel.get(i).copied().unwrap_or(0.0);
                }
                out.push(sum / CHANNELS as f32);
            }
        }

        let flags = MUSIC_FLAGS[id];
        if flags & 0b110 != 0 {
            break;
        }
        id += 1;
    }

    // The loop seam: the channels are ramped per effect, and the mix gets one
    // more ramp so the buffer starts and ends on zero.
    anti_click(&mut out, sample_rate);
    out
}

/// Encode mono `f32` samples as a 16-bit PCM WAV, which is what
/// `macroquad::audio::load_sound_from_bytes` wants.
pub fn to_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = samples.len() * 2;
    let mut out = Vec::with_capacity(44 + data_len);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&((36 + data_len) as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
    }

    fn peak(samples: &[f32]) -> f32 {
        samples.iter().fold(0.0f32, |m, s| m.max(s.abs()))
    }

    fn zero_crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
            .count()
    }

    #[test]
    fn the_pitch_table_runs_from_c0_to_d_sharp_5() {
        assert!((frequency(0) - 65.406).abs() < 0.01, "C0");
        assert!((frequency(12) - 130.813).abs() < 0.02, "C1");
        assert!((frequency(24) - 261.626).abs() < 0.05, "C2");
        assert!(
            (frequency(60) - 2093.0).abs() < 2.0,
            "C5 in PICO-8's octaves is 2093 Hz"
        );
        assert!((frequency(63) - 2489.0).abs() < 3.0, "D#5");
    }

    #[test]
    fn a_fractional_pitch_interpolates_the_table() {
        assert!((frequency_f(12.0) - 130.813).abs() < 0.02);
        // A quarter tone is the twelfth root of two squared.
        assert!((frequency_f(0.5) - C0_HZ * 1.029_302).abs() < 0.01);
    }

    #[test]
    fn every_waveform_sounds_and_stays_in_range() {
        for wave in 0..8u8 {
            let mut noise = Noise::new(1);
            let samples: Vec<f32> = (0..4_410)
                .map(|i| {
                    let phase = (i as f32 * 441.0 / 44_100.0) % 1.0;
                    wave_value(wave, phase, &mut noise)
                })
                .collect();
            assert!(rms(&samples) > 0.2, "wave {wave} is too quiet");
            assert!(peak(&samples) <= 1.0, "wave {wave} peaks above 1.0");
        }
    }

    #[test]
    fn a_sine_at_441_hz_crosses_zero_882_times_a_second() {
        let mut noise = Noise::new(1);
        let samples: Vec<f32> = (0..44_100)
            .map(|i| {
                let phase = (i as f32 * 441.0 / 44_100.0) % 1.0;
                wave_value(0, phase, &mut noise)
            })
            .collect();
        let crossings = zero_crossings(&samples);
        assert!(
            (crossings as i32 - 882).abs() <= 2,
            "sine crosses {crossings} times"
        );
    }

    #[test]
    fn noise_is_not_periodic() {
        let mut noise = Noise::new(7);
        let samples: Vec<f32> = (0..1_000).map(|_| wave_value(7, 0.0, &mut noise)).collect();
        // A tone repeats; noise does not: consecutive samples are unrelated.
        let same_sign = samples
            .windows(2)
            .filter(|w| (w[0] < 0.0) == (w[1] < 0.0))
            .count();
        assert!(same_sign < 600, "noise looks periodic: {same_sign}/999");
        assert!(rms(&samples) > 0.3);
    }

    #[test]
    fn the_sfx_data_parses_into_the_fire_effect() {
        // sfx(18) is the fire sound: two sine notes, speed 4, the second sliding.
        let sfx = Sfx::from_id(18);
        assert_eq!(sfx.speed, 4);
        assert_eq!(sfx.mode, 1);
        assert_eq!(sfx.length(), 2);
        assert_eq!(
            sfx.notes[0],
            Note {
                pitch: 24,
                wave: 0,
                volume: 1,
                effect: 0
            }
        );
        assert_eq!(
            sfx.notes[1],
            Note {
                pitch: 36,
                wave: 0,
                volume: 2,
                effect: 1
            }
        );
    }

    #[test]
    fn every_effect_the_game_uses_has_notes() {
        for id in [8u8, 12, 18, 19, 21] {
            let sfx = Sfx::from_id(id);
            assert!(!sfx.is_silent(), "sfx {id} is silent");
            assert!(sfx.length() > 0);
        }
    }

    #[test]
    fn a_note_lasts_speed_times_183_ticks() {
        let sfx = Sfx::from_id(18);
        // 4 * 183 / 22050 = 0.0332 s a note, two notes = 0.0664 s.
        assert!((sfx.note_seconds() - 0.033_197).abs() < 0.000_1);
        assert!((sfx.duration() - 0.066_394).abs() < 0.000_2);
    }

    #[test]
    fn the_fire_effect_renders_a_short_audible_blip() {
        let sfx = Sfx::from_id(18);
        let samples = render_sfx(&sfx, SAMPLE_RATE);
        let seconds = samples.len() as f32 / SAMPLE_RATE as f32;
        // Two notes at speed 4 is ~66 ms, not the 0.1–0.4 s the issue guessed.
        assert!((0.05..0.08).contains(&seconds), "fire is {seconds} s long");
        assert!(rms(&samples) > 0.01, "the fire sound is audible");
        assert!(peak(&samples) <= 1.0, "and does not clip");
    }

    #[test]
    fn a_silent_note_renders_silence() {
        let mut sfx = Sfx::from_id(18);
        sfx.notes[1].volume = 0;
        sfx.notes[0].volume = 0;
        let samples = render_sfx(&sfx, SAMPLE_RATE);
        assert!(peak(&samples) == 0.0);
    }

    #[test]
    fn fade_out_gets_quieter_and_fade_in_gets_louder() {
        let mut sfx = Sfx::from_id(8);
        for note in sfx.notes.iter_mut() {
            note.volume = 7;
        }
        sfx.notes[0].effect = 5; // fade out
        let out = render_sfx(&sfx, SAMPLE_RATE);
        let step = (sfx.note_seconds() * SAMPLE_RATE as f32) as usize;
        let first = rms(&out[..step / 4]);
        let last = rms(&out[step - step / 4..step]);
        assert!(last < first / 2.0, "fade out: {first} then {last}");

        sfx.notes[0].effect = 4; // fade in
        let inn = render_sfx(&sfx, SAMPLE_RATE);
        assert!(rms(&inn[..step / 4]) < rms(&inn[step - step / 4..step]) / 2.0);
    }

    #[test]
    fn drop_lowers_the_pitch_and_slide_moves_towards_the_next_note() {
        // A long note, so a pitch change is measurable in zero crossings.
        let mut sfx = Sfx::from_id(8);
        sfx.speed = 20;
        for note in sfx.notes.iter_mut() {
            note.volume = 0;
        }
        sfx.notes[0].pitch = 24; // C2
        sfx.notes[0].volume = 7;
        sfx.notes[1].pitch = 48; // C4
        let step = (sfx.note_seconds() * SAMPLE_RATE as f32) as usize;

        sfx.notes[0].effect = 0;
        let flat = render_sfx(&sfx, SAMPLE_RATE);
        sfx.notes[0].effect = 3; // drop
        let dropped = render_sfx(&sfx, SAMPLE_RATE);
        let flat_crossings = zero_crossings(&flat[..step]);
        let drop_crossings = zero_crossings(&dropped[..step]);
        // The drop walks C2 down an octave, so the average pitch (and the crossing
        // rate) is well below the flat note without being half of it.
        assert!(
            drop_crossings * 100 < flat_crossings * 85,
            "drop {drop_crossings} vs flat {flat_crossings}"
        );

        // Slide: the note climbs from C2 towards C4, so the end is much brighter.
        sfx.notes[0].effect = 1;
        let slid = render_sfx(&sfx, SAMPLE_RATE);
        let first = zero_crossings(&slid[..step / 4]);
        let last = zero_crossings(&slid[step * 3 / 4..step]);
        assert!(last > first * 2, "slide: {first} then {last} crossings");
    }

    #[test]
    fn every_effect_the_game_plays_starts_and_ends_on_zero() {
        // The anti-click ramp is what stops a cut-off note from popping.
        for id in [8u8, 12, 18, 19, 21] {
            let samples = render_sfx(&Sfx::from_id(id), SAMPLE_RATE);
            assert_eq!(samples[0], 0.0, "sfx {id} starts on zero");
            assert_eq!(*samples.last().unwrap(), 0.0, "sfx {id} ends on zero");
        }
    }

    #[test]
    fn a_wav_is_a_sixteen_bit_mono_riff_file() {
        let samples = vec![0.0, 0.5, -0.5, 1.0];
        let wav = to_wav(&samples, SAMPLE_RATE);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1, "PCM");
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1, "mono");
        assert_eq!(
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
            SAMPLE_RATE
        );
        assert_eq!(u16::from_le_bytes([wav[34], wav[35]]), 16, "16-bit");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]), 8);
        assert_eq!(wav.len(), 44 + 8);
        // 0.5 becomes 16383 and -0.5 becomes -16383.
        assert_eq!(i16::from_le_bytes([wav[46], wav[47]]), 16_383);
    }

    #[test]
    fn a_music_track_renders_and_loops_without_a_click() {
        let samples = render_music(16, SAMPLE_RATE);
        assert!(
            samples.len() > SAMPLE_RATE as usize / 2,
            "half a second at least"
        );
        assert!(peak(&samples) <= 1.0, "the mix does not clip");
        assert!(rms(&samples) > 0.005, "and is audible");
        assert_eq!(samples[0], 0.0);
        assert_eq!(*samples.last().unwrap(), 0.0);
    }

    #[test]
    fn every_music_the_game_plays_renders() {
        for id in [0u8, 16, 24, 27, 32, 34, 36] {
            let samples = render_music(id, 22_050);
            assert!(!samples.is_empty(), "music {id} rendered nothing");
            assert!(samples.iter().all(|s| s.is_finite()));
        }
    }

    #[test]
    fn a_pattern_has_four_channel_ids() {
        let channels = pattern(16);
        assert_eq!(channels.len(), 4);
        assert!(channels.iter().all(|&c| c < 64), "{channels:?}");
    }
}
