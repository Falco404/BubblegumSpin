//! Headless screenshot: render one frame to a PNG with no window.
//!
//! ```text
//! cargo run --bin shot                                        # card -> out/m1.png
//! cargo run --bin shot -- --scene game --out out/m3.png
//! cargo run --bin shot -- --scene game --frames 30 --spin 0.25 --out out/m3-spin.png
//! ```

use bubblegum_spin::fixed::{from_f64, Fix};
use bubblegum_spin::gfx::{fb::Fb, palette_card, scenes::draw_game};
use bubblegum_spin::sim::{Game, Input};

struct Args {
    scene: String,
    frames: u32,
    out: String,
    spin: Option<Fix>,
    aim: Option<Fix>,
    seed: u32,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            scene: "card".to_string(),
            frames: 1,
            out: String::new(),
            spin: None,
            aim: None,
            seed: 1,
        }
    }
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = Args::default();
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--scene" => args.scene = value()?,
            "--frames" => args.frames = value()?.parse()?,
            "--out" => args.out = value()?,
            "--spin" => args.spin = Some(from_f64(value()?.parse::<f64>()?)),
            "--aim" => args.aim = Some(from_f64(value()?.parse::<f64>()?)),
            "--seed" => args.seed = value()?.parse()?,
            other => return Err(format!("unknown flag: {other}").into()),
        }
    }
    if args.out.is_empty() {
        args.out = match args.scene.as_str() {
            "game" => "out/m3.png".to_string(),
            _ => "out/m1.png".to_string(),
        };
    }
    Ok(args)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let mut fb = Fb::new();

    match args.scene.as_str() {
        "card" => palette_card(&mut fb),
        "game" => {
            let mut game = Game::new(args.seed);
            if let Some(var) = args.spin {
                game.spin.var = var;
            }
            if let Some(a) = args.aim {
                game.gun.a = a;
            }
            // Step the simulation the requested number of frames with a fixed
            // input script, so a screenshot can show motion (spin, aim).
            for _ in 0..args.frames {
                let input = Input::default();
                game.step(&input);
            }
            draw_game(&mut fb, &game);
        }
        other => return Err(format!("unknown scene: {other}").into()),
    }

    if let Some(dir) = std::path::Path::new(&args.out).parent() {
        std::fs::create_dir_all(dir)?;
    }
    fb.to_png(&args.out)?;
    println!("wrote {}", args.out);
    Ok(())
}
