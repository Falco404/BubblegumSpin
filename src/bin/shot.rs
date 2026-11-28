//! Headless screenshot: render one frame to a PNG with no window.
//!
//! ```text
//! cargo run --bin shot                                        # card -> out/m1.png
//! cargo run --bin shot -- --scene game --out out/m3.png
//! cargo run --bin shot -- --scene game --frames 30 --spin 0.25 --out out/m3-spin.png
//! cargo run --bin shot -- --scene game --frames 12 --fire 1 --out out/m4-fire.png
//! ```

use bubblegum_spin::fixed::{from_f64, to_f64, Fix};
use bubblegum_spin::gfx::{fb::Fb, menus, palette_card, scenes::draw_game, scenes::fade_scr};
use bubblegum_spin::sim::scene::{App, Scene, FINAL_NAV};
use bubblegum_spin::sim::{Game, Input};

struct Args {
    scene: String,
    frames: u32,
    out: String,
    spin: Option<Fix>,
    aim: Option<Fix>,
    /// 0-based frame on which the X button is pressed, then released.
    fire: Option<u32>,
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
            fire: None,
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
            "--fire" => args.fire = Some(value()?.parse()?),
            "--seed" => args.seed = value()?.parse()?,
            other => return Err(format!("unknown flag: {other}").into()),
        }
    }
    if args.out.is_empty() {
        args.out = match args.scene.as_str() {
            "game" => "out/game.png".to_string(),
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
            // input script, so a screenshot can show motion (spin, aim, a shot).
            for frame in 0..args.frames {
                let input = Input {
                    fire: args.fire == Some(frame),
                    ..Default::default()
                };
                game.step(&input);
            }
            draw_game(&mut fb, &game);
        }
        "over" => {
            // The game-over cascade: one gumball past the right wall ends it, and
            // the whole mass falls off the screen (M5).
            let mut game = Game::new(args.seed);
            game.board.0.insert(270, 3);
            for _ in 0..args.frames {
                game.step(&Input::default());
            }
            draw_game(&mut fb, &game);
        }
        "main" | "how" | "coin" | "final" => {
            // The scenes around the game, driven by the same machine the window uses.
            let mut app = app_for(&args.scene, args.seed);
            for _ in 0..args.frames {
                app.step(&Input::default());
            }
            fade_scr(&mut fb, to_f64(app.game.fade.value));
            menus::draw(&mut fb, &app);
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

/// Put the scene machine on the screen the caller asked for.
///
/// `how` and `coin` are reached the way a player reaches them (a tap on the menu);
/// `final` is set up directly, because playing a whole game to lose it is not a
/// screenshot.
fn app_for(scene: &str, seed: u32) -> App {
    let mut app = App::new(seed);
    match scene {
        "how" => {
            settle(&mut app);
            app.step(&Input {
                down: true,
                ..Default::default()
            });
            app.step(&Input::default());
            app.step(&Input {
                fire: true,
                ..Default::default()
            });
            app.step(&Input::default());
        }
        "coin" => {
            settle(&mut app);
            app.step(&Input {
                fire: true,
                ..Default::default()
            });
            app.step(&Input::default());
        }
        "final" => {
            app.game.new_game();
            app.game.score = 700;
            app.game.fade.finish_in();
            app.scene = Scene::Final;
            app.nav = &FINAL_NAV;
            app.finalani = 1;
            app.gumballs = 7;
            app.blow_force = 7;
        }
        _ => {}
    }
    app
}

/// Step until the title's fade has finished, as the cart's `faded` flag does.
fn settle(app: &mut App) {
    for _ in 0..200 {
        if app.game.fade.faded {
            return;
        }
        app.step(&Input::default());
    }
}
