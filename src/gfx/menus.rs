//! The screens around the game: title, how-to, the coin drop and the score screen.
//!
//! cart: `draw_main`, `draw_how`, `draw_coin`, `draw_final`, `draw_scene`, `nav2`,
//! `draw_sspr`, `getframe`. Pure framebuffer functions driven by [`App`], so each
//! screen can be rendered and compared headlessly.

use crate::fixed::{div, from_f64, int, mul, to_int, Fix};
use crate::gfx::fb::Fb;
use crate::gfx::scenes::{bubcol, draw_banner, draw_map, reset_pal, MAP_COLS};
use crate::gfx::sprites;
use crate::gfx::text::{coprint, cprint};
use crate::p8math::sin;
use crate::sim::scene::{App, Hero, Scene};

/// One animation frame of a drawn object (cart `hero[...]`, `blonde`, `jbox`).
pub struct Frames {
    /// Where the object's left edge sits, in pixels.
    pub x: i32,
    pub sx: &'static [i32],
    pub sy: i32,
    pub sw: i32,
    pub sh: i32,
    pub dw: i32,
    pub dh: i32,
}

impl Frames {
    /// cart `getframe`: `sx[flr(t/10) % #sx]`.
    pub fn sx_at(&self, t: i32) -> i32 {
        let n = self.sx.len() as i32;
        self.sx[(t / 10).rem_euclid(n) as usize]
    }
}

pub static HERO_IDLE: Frames = Frames {
    x: 58,
    sx: &[27],
    sy: 32,
    sw: 9,
    sh: 18,
    dw: 18,
    dh: 36,
};
pub static HERO_POSE: Frames = Frames {
    x: 58,
    sx: &[86],
    sy: 32,
    sw: 10,
    sh: 18,
    dw: 20,
    dh: 36,
};
pub static HERO_NOM: Frames = Frames {
    x: 60,
    sx: &[96],
    sy: 32,
    sw: 12,
    sh: 18,
    dw: 24,
    dh: 36,
};
pub static HERO_CHEW: Frames = Frames {
    x: 58,
    sx: &[0, 9, 18],
    sy: 32,
    sw: 9,
    sh: 18,
    dw: 18,
    dh: 36,
};
pub static HERO_BLOW: Frames = Frames {
    x: 60,
    sx: &[74],
    sy: 32,
    sw: 12,
    sh: 18,
    dw: 24,
    dh: 36,
};
pub static BLONDE: Frames = Frames {
    x: 16,
    sx: &[36, 36, 36, 36, 46, 46, 56, 56, 56, 56, 46, 46],
    sy: 32,
    sw: 10,
    sh: 18,
    dw: 20,
    dh: 36,
};
pub static JBOX: Frames = Frames {
    x: 108,
    sx: &[92],
    sy: 96,
    sw: 18,
    sh: 18,
    dw: 36,
    dh: 36,
};

/// The two pages of the how-to text, verbatim from the cart.
pub static HOW_1: [&str; 12] = [
    "match three or more gumballs",
    "of the same color to remove",
    "them from the sticky mass.",
    "",
    "unmatching gumballs that are",
    "unrooted will also be removed.",
    "",
    "remove all the gumballs",
    "to move to the next round.",
    "",
    "if the sticky mass touches",
    "the edge, it's game over!",
];

pub static HOW_2: [&str; 13] = [
    "after every four misses,",
    "you will be punished.",
    "",
    "don't dawdle! each punishment",
    "hurts more than the last...",
    "",
    "",
    "left/right: aim",
    "up: return aimer to upright",
    "X: fire",
    "O: hold to preview your next",
    "gumballs and view your",
    "remaining misses",
];

/// The hero frame for the current animation state.
pub fn hero_frames(hero: Hero) -> &'static Frames {
    match hero {
        Hero::Idle => &HERO_IDLE,
        Hero::Pose => &HERO_POSE,
        Hero::Nom => &HERO_NOM,
        Hero::Chew => &HERO_CHEW,
        Hero::Blow => &HERO_BLOW,
    }
}

/// cart `draw_scene()`: the background map, the kid, the machine and the gum bubble.
///
/// The machine is scaled by `trans_t/trans_acc+1`, which is what "zooms" it out during
/// the first round and on the title screen (`trans_t` is 0 there, so `s` is 1).
pub fn draw_scene(fb: &mut Fb, app: &App) {
    // cart `map(16,0)`: the background art starts sixteen cells into the map, so the
    // logo area (above it, and drawn before this) stays visible.
    let rows = (crate::gfx::scenes::MAP.len() as i32) / MAP_COLS;
    draw_map(fb, 16, 0, 0, 0, MAP_COLS, rows);

    let game = &app.game;
    let y = 120;
    let s = zoom_scale(game.trans_t, game.trans_acc);

    // cart `draw_sspr(jbox,108,y-36)`
    let sheet = sprites::cart();
    fb.sspr(
        &sheet,
        JBOX.sx_at(game.t),
        JBOX.sy,
        JBOX.sw,
        JBOX.sh,
        JBOX.x,
        y - 36,
        JBOX.dw,
        JBOX.dh,
    );

    // cart `draw_sspr(blonde,16,y-18*2,true)` — mirrored.
    fb.sspr_flip(
        &sheet,
        BLONDE.sx_at(game.t),
        BLONDE.sy,
        BLONDE.sw,
        BLONDE.sh,
        BLONDE.x,
        y - 18 * 2,
        BLONDE.dw,
        BLONDE.dh,
    );

    // cart `draw_sspr(state,state.x,y-18*2)`
    let hero = hero_frames(app.hero);
    fb.sspr(
        &sheet,
        hero.sx_at(game.t),
        hero.sy,
        hero.sw,
        hero.sh,
        hero.x,
        y - 18 * 2,
        hero.dw,
        hero.dh,
    );

    // cart: the machine shakes on `t%6>3` while it rumbles.
    let mut o = 0;
    if app.rumble && game.t % 6 > 3 {
        o = 1;
    }
    fb.sspr(
        &sheet,
        110,
        96,
        8,
        18,
        to_int(int(46) - mul(from_f64(7.0), s)) + o,
        to_int(int(99) - mul(from_f64(15.0), s)),
        to_int(mul(int(16), s)),
        to_int(mul(int(36), s)),
    );

    if app.blowing {
        let r = to_int(app.blow_r);
        let x = to_int(app.blow_x);
        let y = 98 - r;
        fb.circfill(x, y, r, 14);
        if r > 4 {
            let r7 = to_int(div(app.blow_r, int(7)));
            let r12 = to_int(div(app.blow_r, int(12)));
            fb.circfill(
                to_int(app.blow_x - mul(from_f64(0.6), app.blow_r)),
                98 - to_int(mul(from_f64(1.4), app.blow_r)),
                r7,
                7,
            );
            fb.circfill(
                to_int(app.blow_x - mul(from_f64(0.5), app.blow_r)),
                98 - to_int(mul(from_f64(1.5), app.blow_r)),
                r7,
                7,
            );
            fb.circfill(
                to_int(app.blow_x - mul(from_f64(0.8), app.blow_r)),
                98 - to_int(mul(from_f64(1.1), app.blow_r)),
                r12,
                7,
            );
        }
    }
}

/// `trans_t/trans_acc+1` as the cart computes it, guarded against a zero divisor.
pub fn zoom_scale(trans_t: i32, trans_acc: Fix) -> Fix {
    if trans_acc == 0 {
        return int(1);
    }
    div(int(trans_t), trans_acc) + int(1)
}

/// cart `nav2()`: the menu, with the focused row bobbing.
pub fn nav2(fb: &mut Fb, app: &App) {
    for (i, entry) in app.nav.iter().enumerate() {
        let focused = i + 1 == app.focus;
        let (fill, outline) = if focused { (7, 1) } else { (6, 1) };
        let mut o = 0;
        if focused {
            o = to_int(mul(sin(int(app.game.t) / 50), int(2)));
        }
        coprint(fb, entry.text, 64, 60 + 12 * i as i32 + o, fill, outline);
    }
}

/// cart `draw_main()`: the title, the menu and the machine.
pub fn draw_main(fb: &mut Fb, app: &App) {
    fb.clear(13);
    let sheet = sprites::cart();
    // The logo and the "press to start" strip, copied straight from the sheet.
    fb.sspr(&sheet, 0, 64, 127, 20, 2, 12, 127, 20);
    fb.sspr(&sheet, 0, 84, 88, 28, 11, 34, 88, 28);

    nav2(fb, app);

    if app.game.hiscore > 0 {
        let text = format!("\u{2605} hiscore : {}0 \u{2605}  ", app.game.hiscore);
        coprint(fb, &text, 64, 4, 10, 4);
    }

    draw_scene(fb, app);
}

/// cart `draw_how()`: the two pages of text, centred, with a bobbing prompt.
pub fn draw_how(fb: &mut Fb, app: &App) {
    fb.rectfill(0, 0, 127, 127, 13);
    let lines: &[&str] = if app.how_page == 2 { &HOW_2 } else { &HOW_1 };
    for (i, line) in lines.iter().enumerate() {
        // cart: `cprint(txt[i],64,i*8+1-2,14)` then `cprint(txt[i],64,i*8-2,7)` — the
        // pink is a drop shadow one pixel below the white text. `i` is 1-based there.
        let y = (i as i32 + 1) * 8 - 1;
        cprint(fb, line, 64, y, 14);
        cprint(fb, line, 64, y - 1, 7);
    }
    let o = to_int(mul(sin(int(app.game.t) / 50), int(2)));
    coprint(fb, "press X to continue", 64, 120 + o, 7, 1);
}

/// cart `draw_coin()`: the machine, the kid and the coin dropping in.
pub fn draw_coin(fb: &mut Fb, app: &App) {
    fb.clear(0);
    let game = &app.game;
    let y = 120;
    let s = zoom_scale(game.trans_t, game.trans_acc);
    let sheet = sprites::cart();

    // cart `draw_sspr(state,state.x,y-18*2)` — the kid in the starting pose.
    let hero = hero_frames(app.hero);
    fb.sspr(
        &sheet,
        hero.sx_at(game.t),
        hero.sy,
        hero.sw,
        hero.sh,
        hero.x,
        y - 18 * 2,
        hero.dw,
        hero.dh,
    );

    let mut o = 0;
    if app.rumble && game.t % 6 > 3 {
        o = 1;
    }
    fb.sspr(
        &sheet,
        110,
        96,
        8,
        18,
        to_int(int(46) - mul(from_f64(7.0), s)) + o,
        to_int(int(99) - mul(from_f64(15.0), s)),
        to_int(mul(int(16), s)),
        to_int(mul(int(36), s)),
    );

    if let Some(coin) = app.coin {
        let sx = coin.frame(game.t) as i32 * 16;
        let half = div(coin.size, int(2));
        fb.sspr(
            &sheet,
            sx,
            112,
            16,
            16,
            to_int(coin.pos.0 - half),
            to_int(coin.pos.1 - half),
            to_int(coin.size),
            to_int(coin.size),
        );
    }
}

/// cart `draw_final()`: the score screen with the machine dispensing its gumballs.
pub fn draw_final(fb: &mut Fb, app: &App) {
    fb.clear(13);
    draw_scene(fb, app);

    let sheet = sprites::cart();
    for bub in &app.score_bubs {
        bubcol(fb, bub.col);
        // cart: `dw=(32/b.pos[2])*3+2`, then `sspr(...,dw,dh)`. Above the machine the
        // quotient goes negative, and PICO-8 draws a negative size mirrored, so the
        // sprite flips instead of vanishing. The clamp is the port's: a y just above
        // zero would otherwise ask for a sprite thousands of pixels wide.
        let y = bub.pos.1;
        let raw = if y == 0 {
            2
        } else {
            to_int(mul(div(int(32), y), int(3))) + 2
        };
        let dw = raw.abs().clamp(1, 128);
        let (dx, dy) = (to_int(bub.pos.0) - dw / 2, to_int(bub.pos.1) - dw / 2);
        if raw < 0 {
            fb.sspr_flip(&sheet, 16, 0, 8, 8, dx, dy, dw, dw);
        } else {
            fb.sspr(&sheet, 16, 0, 8, 8, dx, dy, dw, dw);
        }
        reset_pal(fb);
    }

    if app.finalani == 4 {
        if app.game.new_hiscore {
            coprint(fb, "\u{2605} new hiscore \u{2605}  ", 64, 8, 10, 4);
        }
        draw_banner(fb, -8, 29, 136, 39, 12, 7);
        let text = format!("you scored {}0 gumballs!", app.game.score);
        cprint(fb, &text, 64, 32, 7);
        nav2(fb, app);
    }
}

/// Which screen to draw for a scene: the cart's `_drw` swap.
pub fn draw(fb: &mut Fb, app: &App) {
    match app.scene {
        Scene::Main => draw_main(fb, app),
        Scene::How => draw_how(fb, app),
        Scene::Coin => draw_coin(fb, app),
        Scene::Game | Scene::Over => crate::gfx::scenes::draw_game(fb, &app.game),
        Scene::Final => draw_final(fb, app),
    }
}
