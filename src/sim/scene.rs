//! The cart's scene machine: what `_init`/`_upd`/`_drw` currently point at.
//!
//! The cart swaps three function pointers around a single global state: title, how-to,
//! the coin drop, the game, the game-over cascade and the final screen. This module is
//! that swap, plus the `nav` menus and the `reset_scene()` fields the effects borrow.
//!
//! There is **one clock**: the cart's `t` runs in every scene, so it stays where it
//! already lives — [`Game::t`] — and the machine advances it for the scenes that are
//! not the game. Same for the fade, which wraps every screen.

use crate::fixed::{int, mul, Fix, HALF, ONE};
use crate::p8math::{bezier, sin};
use crate::sim::spin::Zoom;
use crate::sim::{Game, Input};

/// Which screen is running (the cart's `_upd`/`_drw` pair).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    /// Title screen with the two-entry menu (cart `init_main`/`update_main`).
    Main,
    /// The two pages of "how to play" (cart `init_how`/`update_how`).
    How,
    /// The coin drop before the game starts (cart `init_coin`/`update_coin`).
    Coin,
    /// Playing (cart `update_game`).
    Game,
    /// The cascade after the mass touches the edge (cart `update_over`).
    Over,
    /// Score, the machine dispensing gumballs and the menu (cart `init_final`).
    Final,
}

/// What a menu entry does when picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// "start game": the coin drop, then the zoom into the board.
    StartGame,
    /// "how to play".
    HowTo,
    /// "retry": a fresh game with no coin drop.
    Retry,
    /// "return to title".
    Title,
}

/// One row of a menu (`nav[i]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavEntry {
    pub text: &'static str,
    pub action: Action,
}

/// The title menu, in the cart's order.
pub const MAIN_NAV: [NavEntry; 2] = [
    NavEntry {
        text: "start game",
        action: Action::StartGame,
    },
    NavEntry {
        text: "how to play",
        action: Action::HowTo,
    },
];

/// The menu on the score screen.
pub const FINAL_NAV: [NavEntry; 2] = [
    NavEntry {
        text: "retry",
        action: Action::Retry,
    },
    NavEntry {
        text: "return to title",
        action: Action::Title,
    },
];

/// PICO-8's `btnp()`: true on the press, then again after 15 frames held, then
/// every 4 frames while it stays down.
///
/// The cart's menus and its fire button all use `btnp`, so holding a key repeats —
/// pulling the trigger once per press is a port-only behaviour.
#[derive(Debug, Clone, Copy, Default)]
pub struct Btnp {
    held: i32,
}

impl Btnp {
    /// Feed one frame of the button and get this frame's `btnp` value.
    pub fn press(&mut self, down: bool) -> bool {
        if !down {
            self.held = 0;
            return false;
        }
        self.held += 1;
        self.held == 1 || (self.held > 15 && (self.held - 16) % 4 == 0)
    }

    /// How long the button has been held, for tests and for the HUD.
    pub fn held(&self) -> i32 {
        self.held
    }
}

/// The cart's `hero` animation state: which sprite frame the kid is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hero {
    #[default]
    Idle,
    /// The starting pose on the coin screen.
    Pose,
    Nom,
    Chew,
    Blow,
}

/// How far the coin moves along its curve each frame (the cart's `0x.02`).
pub const COIN_STEP: Fix = 512;
/// The coin shrinks by this much per frame once it is past halfway (cart `.98`).
pub const COIN_SHRINK: Fix = 64225;
/// cart `.99`: the coin is home.
pub const COIN_LANDED: Fix = 64880;

/// The coin the kid drops into the machine (cart `coin`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coin {
    /// cart `curve`: three control points, start / apex / scoop.
    pub curve: [(Fix, Fix); 3],
    pub frac: Fix,
    /// cart `coin.pos`, re-evaluated from `frac` every frame.
    pub pos: (Fix, Fix),
    /// cart `coin.s`: the sprite's size, which shrinks as it falls.
    pub size: Fix,
}

impl Coin {
    /// Which of the six spin frames to draw (cart `getframe`: `t/10 % #sx`).
    pub fn frame(&self, t: i32) -> usize {
        (t / 10).rem_euclid(6) as usize
    }
}

/// A gumball arcing from the machine to the kid on the score screen.
///
/// The cart reuses `bubs` for this, with a curve/frac/pos triple; keeping them apart
/// means the game's bubbles never have to carry animation fields they do not use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreBub {
    pub curve: [(Fix, Fix); 3],
    pub frac: Fix,
    pub pos: (Fix, Fix),
    pub col: u16,
}

/// Everything the cart keeps outside the board: the machine, its menus and the
/// effects' shared fields (`reset_scene`).
#[derive(Debug)]
pub struct App {
    pub game: Game,
    pub scene: Scene,
    /// The current menu (cart `nav`); empty outside the two menu screens.
    pub nav: &'static [NavEntry],
    /// 1-based, exactly like the cart's `focus` and `mid(1, focus, #nav)`.
    pub focus: usize,
    /// 1 or 2: which page of the how-to text (cart `txt`).
    pub how_page: u8,

    /// cart `state`: which hero frame to draw.
    pub hero: Hero,
    /// cart `finalani`: "1" dispense, "2" chew, "3" blow, "4" menu.
    pub finalani: u8,
    /// cart `rumble`: shake the gumball machine this frame.
    pub rumble: bool,
    /// cart `event_t`: frames inside the current `finalani` step.
    pub event_t: i32,
    /// cart `gumballs`: how many the machine still has to spit out on the score screen.
    pub gumballs: i32,
    /// cart `coin`: the coin dropping into the machine, until it lands.
    pub coin: Option<Coin>,
    /// cart `rumble_t`: frames the machine shakes after the coin lands.
    pub rumble_t: i32,

    /// cart `blow_r`/`blow_force`/`blowing`: the gum bubble the kid blows.
    pub blow_r: Fix,
    pub blow_x: Fix,
    pub blow_force: i32,
    pub blowing: bool,
    /// Gumballs the machine spits out on the score screen (cart reuses `bubs`).
    pub score_bubs: Vec<ScoreBub>,
    /// The menu entry picked on the score screen, run once its fade-out finishes.
    pending: Option<NavEntry>,

    fire: Btnp,
    up: Btnp,
    down: Btnp,
}

impl App {
    /// The port's boot state: the cart's `_init` down to `init_main()`.
    pub fn new(seed: u32) -> Self {
        Self::new_with_save(seed, crate::save::Save::default(), None)
    }

    pub fn new_with_save(
        seed: u32,
        save: crate::save::Save,
        save_path: Option<std::path::PathBuf>,
    ) -> Self {
        let mut app = Self {
            game: Game::blank(seed, save, save_path),
            scene: Scene::Main,
            nav: &MAIN_NAV,
            focus: 1,
            how_page: 1,
            hero: Hero::Idle,
            finalani: 1,
            rumble: false,
            event_t: 0,
            gumballs: 0,
            coin: None,
            rumble_t: 0,
            blow_r: int(2),
            blow_x: int(68),
            blow_force: 0,
            blowing: false,
            score_bubs: Vec::new(),
            pending: None,
            fire: Btnp::default(),
            up: Btnp::default(),
            down: Btnp::default(),
        };
        app.enter_main();
        app
    }

    /// One frame at 60 Hz: the cart's `_update60` and the current `_upd`.
    pub fn step(&mut self, input: &Input) {
        // The buttons are read every frame, whatever the scene, so a press that
        // lands during a fade is not replayed later.
        let fire = self.fire.press(input.fire);
        let up = self.up.press(input.up);
        let down = self.down.press(input.down);

        match self.scene {
            Scene::Main => self.step_main(fire, up, down),
            Scene::How => self.step_how(fire),
            Scene::Coin => self.step_coin(),
            Scene::Game => self.step_game(input),
            Scene::Over => self.step_over(input),
            Scene::Final => self.step_final(fire, up, down),
        }
    }

    /// The cart's `t+=1` and `update_fade()` for a scene that is not the game.
    ///
    /// The game does both itself inside [`Game::step`], so it is not repeated here.
    fn tick(&mut self) {
        self.game.t += 1;
        self.game.fade.step();
    }

    /// cart `init_main()`: title music, a fade in, and the two-entry menu.
    fn enter_main(&mut self) {
        self.game.music(0);
        self.reset_scene();
        self.game.fade.start_in();
        self.scene = Scene::Main;
        self.nav = &MAIN_NAV;
        self.focus = 1;
    }

    /// cart `reset_scene()`: the effect fields the scenes share.
    fn reset_scene(&mut self) {
        self.hero = Hero::Idle;
        self.finalani = 1;
        self.rumble = false;
        self.event_t = 0;
        self.gumballs = 0;
        self.blow_r = int(2);
        self.blow_x = int(68);
        self.blow_force = 0;
        self.blowing = false;
        self.score_bubs.clear();
        self.pending = None;
    }

    /// Run the entry the menu is on (the cart's `nav[focus]` function pointers).
    fn activate(&mut self, entry: NavEntry) {
        match entry.action {
            Action::StartGame => self.enter_coin(),
            Action::HowTo => self.enter_how(),
            Action::Retry => self.enter_retry(),
            Action::Title => self.enter_main(),
        }
    }

    /// cart `init_how()`: the first page of text. Nothing else changes — no fade, no
    /// music — which is why going back keeps the title's state.
    fn enter_how(&mut self) {
        self.scene = Scene::How;
        self.how_page = 1;
    }

    /// cart `update_main()`: X picks the entry, up/down move the focus.
    fn step_main(&mut self, fire: bool, up: bool, down: bool) {
        self.tick();
        if !self.game.fade.faded {
            return;
        }
        if fire {
            let entry = self.nav[self.focus - 1];
            self.activate(entry);
            return;
        }
        if up {
            self.focus -= 1;
        }
        if down {
            self.focus += 1;
        }
        // The cart clamps after moving, so a menu of two cannot land on 0 or 3.
        self.focus = self.focus.clamp(1, self.nav.len());
    }

    /// cart `update_how()`: X turns the page, then leaves — and it leaves by pointing
    /// at `update_main` without re-running `init_main`, so no second fade or music.
    fn step_how(&mut self, fire: bool) {
        self.tick();
        if !fire {
            return;
        }
        if self.how_page == 2 {
            self.scene = Scene::Main;
        } else {
            self.how_page = 2;
        }
    }

    /// cart `init_coin()`: the coin-drop screen, and the clock restarts with it.
    fn enter_coin(&mut self) {
        self.game.music(36);
        self.game.t = 0;
        self.scene = Scene::Coin;
        self.hero = Hero::Pose;
        self.rumble_t = 0;
        self.rumble = false;
        let curve = [
            (crate::fixed::int(68), crate::fixed::int(68)),
            (crate::fixed::int(58), crate::fixed::int(-32)),
            (crate::fixed::int(48), crate::fixed::int(104)),
        ];
        self.coin = Some(Coin {
            curve,
            frac: 0,
            pos: bezier(&curve, 0),
            size: crate::fixed::int(32),
        });
    }

    /// cart `update_coin()`: the coin falls, the machine shakes, then the zoom starts.
    fn step_coin(&mut self) {
        self.tick();

        if let Some(mut coin) = self.coin {
            if self.game.t > 60 {
                coin.frac = (coin.frac + COIN_STEP) % ONE;
                coin.pos = bezier(&coin.curve, coin.frac);
            }
            if coin.frac > HALF {
                coin.size = mul(coin.size, COIN_SHRINK);
            }
            if coin.frac > COIN_LANDED {
                self.rumble_t = 15;
                self.coin = None;
            } else {
                self.coin = Some(coin);
            }
        }

        self.rumble = false;
        if self.rumble_t > 0 {
            self.rumble = true;
            self.rumble_t -= 1;
            self.hero = Hero::Idle;
        }

        if self.coin.is_none() && !self.rumble {
            // cart `init_game()` + `_upd = update_zoom_transition`.
            if self.game.zoom == Zoom::None {
                self.game.start_zoom();
            } else {
                self.game.update_zoom_transition();
                if self.game.zoom == Zoom::Out {
                    // The zoom hit the top: `new_game()` dealt the first round.
                    self.scene = Scene::Game;
                }
            }
        }
    }

    /// cart `update_game()`: the game plays; the first frame with the mass over the
    /// edge moves to the cascade.
    fn step_game(&mut self, input: &Input) {
        self.game.step(input);
        if self.game.over {
            self.enter_over();
        }
    }

    /// cart `check_over()`'s tail: the mass is falling and the screen starts to fade.
    fn enter_over(&mut self) {
        self.scene = Scene::Over;
        self.hero = Hero::Idle;
        self.event_t = 0;
        // cart `check_over()`: one gumball per 100 points for the score screen.
        self.gumballs = self.game.score / 100;
        self.blow_force = self.gumballs;
    }

    /// cart `update_over()`: the cascade runs, the screen fades to black, and then the
    /// score screen fades back in.
    fn step_over(&mut self, input: &Input) {
        self.game.step(input);
        if self.game.fade.faded && self.game.fade.value >= crate::fixed::ONE {
            self.enter_final();
        } else if self.game.bubs.is_empty() && self.game.fade.faded {
            self.game.fade.start_out();
        }
    }

    /// cart `update_over()`'s hand-off: the score screen, with the score or hiscore
    /// track queued.
    fn enter_final(&mut self) {
        self.scene = Scene::Final;
        self.nav = &FINAL_NAV;
        self.focus = 1;
        self.game.fade.start_in();
        if self.game.new_hiscore {
            self.game.music(27);
        } else {
            self.game.music(24);
        }
    }

    /// cart `init_final()` and `update_final()`: the machine dispenses one gumball per
    /// 100 points, the kid chews, blows a bubble, and then the menu appears.
    fn step_final(&mut self, fire: bool, up: bool, down: bool) {
        self.tick();
        self.update_blow();

        if !self.game.fade.faded {
            self.event_t += 1;
            return;
        }

        match self.finalani {
            1 => {
                if self.game.score == 0 {
                    // Nothing to show: straight to the menu.
                    self.finalani = 4;
                    return;
                }
                self.rumble = true;
                self.hero = Hero::Nom;
                self.dispense();
                self.update_score_bubs();
                if self.score_bubs.is_empty() {
                    self.finalani = 2;
                    self.event_t = 0;
                }
            }
            2 => {
                self.rumble = false;
                self.hero = Hero::Chew;
                if self.event_t > 120 {
                    self.finalani = 3;
                    self.event_t = 0;
                }
            }
            3 => {
                self.hero = Hero::Blow;
                self.blow_bubble();
            }
            _ => {
                // The fade-out that follows a choice finally lands here.
                if self.game.fade.value >= ONE && self.game.fade.faded {
                    if let Some(entry) = self.pending.take() {
                        self.activate(entry);
                        return;
                    }
                }
                if fire {
                    let entry = self.nav[self.focus - 1];
                    // cart `init_fadeout()` then `_init=nav[focus].init`: the switch
                    // waits for the screen to go black, and `completed` is saved.
                    self.game.fade.start_out();
                    self.game.save.completed = true;
                    self.pending = Some(entry);
                }
                if up {
                    self.focus -= 1;
                }
                if down {
                    self.focus += 1;
                }
                self.focus = self.focus.clamp(1, self.nav.len());
            }
        }
        self.event_t += 1;
    }

    /// cart `dispense()`: one gumball leaves the machine, on a bezier to the kid.
    fn dispense(&mut self) {
        if self.gumballs <= 0 {
            return;
        }
        self.gumballs -= 1;
        let (mx, my) = (int(46), int(99));
        let curve = [
            (mx + int(2), my + int(13)),
            (
                mx + self.game.rng.rnd_fix(int(32)),
                self.game.rng.rnd_fix(int(96)) - int(96),
            ),
            (int(68), int(19) + int(120) - int(18) * 2),
        ];
        let col = self.game.rnd_col();
        self.score_bubs.push(ScoreBub {
            curve,
            frac: 0,
            pos: bezier(&curve, 0),
            col,
        });
    }

    /// cart `update_bub_ani()`: the dispensed gumballs follow their curves and go.
    fn update_score_bubs(&mut self) {
        for bub in &mut self.score_bubs {
            bub.frac = (bub.frac + COIN_STEP) % ONE;
            bub.pos = bezier(&bub.curve, bub.frac);
        }
        self.score_bubs.retain(|bub| bub.frac <= COIN_LANDED);
    }

    /// cart `update_blow()`: the bubble wobbles once it is wide enough.
    fn update_blow(&mut self) {
        self.blow_x = int(68);
        if self.blow_r > int(10) {
            self.blow_x += mul(
                sin(int(self.game.t) / 200),
                crate::fixed::div(self.blow_r, int(10)),
            );
        }
    }

    /// cart `blow_bubble()`: the bubble grows one step per frame, then the menu opens.
    fn blow_bubble(&mut self) {
        self.blowing = true;
        if self.blow_force > 0 {
            self.blow_force -= 1;
            self.blow_r += 13_107; // 0.2, quantised
        } else {
            self.finalani = 4;
        }
    }

    /// cart `init_retry()`: no fade at all (`fade=0; faded=true`), a fresh game and
    /// no coin drop — the retry is instant.
    fn enter_retry(&mut self) {
        self.game.fade.finish_in();
        self.game.new_game();
        self.reset_scene();
        self.scene = Scene::Game;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn still() -> Input {
        Input::default()
    }
    fn pressed() -> Input {
        Input {
            fire: true,
            ..Default::default()
        }
    }
    fn up() -> Input {
        Input {
            up: true,
            ..Default::default()
        }
    }
    fn down() -> Input {
        Input {
            down: true,
            ..Default::default()
        }
    }

    /// Let the title's fade run out, as the cart's `faded` flag does.
    fn settle(app: &mut App) {
        for _ in 0..200 {
            if app.game.fade.faded {
                return;
            }
            app.step(&still());
        }
        panic!("the fade never finished");
    }

    /// A fresh press of X after releasing it: the cart's `btnp`, not a held button.
    fn tap(app: &mut App, input: Input) {
        app.step(&input);
        app.step(&still());
    }

    #[test]
    fn the_app_opens_on_the_title_screen() {
        let mut app = App::new(7);
        assert_eq!(app.scene, Scene::Main);
        assert_eq!(app.nav, &MAIN_NAV[..]);
        assert_eq!(app.focus, 1);
        // cart `init_main()` starts a fade in and asks for track 0.
        assert_eq!(app.game.fade.value, crate::fixed::ONE);
        assert!(!app.game.fade.faded);
        assert_eq!(app.game.take_music(), vec![0]);
        assert_eq!(app.hero, Hero::Idle);
        assert!(!app.rumble);
    }

    #[test]
    fn the_title_takes_no_input_until_the_fade_finishes() {
        let mut app = App::new(7);
        app.step(&pressed());
        assert_eq!(app.scene, Scene::Main, "the fade has not finished yet");
        settle(&mut app);
        tap(&mut app, pressed());
        assert_eq!(app.scene, Scene::Coin);
    }

    #[test]
    fn the_focus_moves_with_up_and_down_and_clamps_at_the_ends() {
        let mut app = App::new(7);
        settle(&mut app);
        tap(&mut app, down());
        assert_eq!(app.focus, 2);
        tap(&mut app, down());
        assert_eq!(app.focus, 2, "two entries, so it clamps at 2");
        tap(&mut app, up());
        assert_eq!(app.focus, 1);
        tap(&mut app, up());
        assert_eq!(app.focus, 1, "and at 1");
    }

    #[test]
    fn the_second_entry_opens_the_how_to_play_page() {
        let mut app = App::new(7);
        settle(&mut app);
        tap(&mut app, down());
        tap(&mut app, pressed());
        assert_eq!(app.scene, Scene::How);
        assert_eq!(app.how_page, 1);
    }

    #[test]
    fn the_how_to_page_turns_once_and_returns_without_another_fade() {
        let mut app = App::new(7);
        settle(&mut app);
        tap(&mut app, down());
        tap(&mut app, pressed());
        assert_eq!(app.scene, Scene::How);

        // The first X turns the page; the how-to screen is not gated by the fade.
        tap(&mut app, pressed());
        assert_eq!(app.how_page, 2);
        assert_eq!(app.scene, Scene::How);

        // The second X points back at `update_main` without `init_main`: the menu is
        // where it was and there is no second fade or title track.
        app.game.take_music();
        tap(&mut app, pressed());
        assert_eq!(app.scene, Scene::Main);
        assert_eq!(app.focus, 2);
        assert!(app.game.fade.faded);
        assert!(app.game.take_music().is_empty());
    }

    #[test]
    fn holding_a_button_repeats_like_btnp() {
        // PICO-8: press, then after 15 frames held, then every 4 frames.
        let mut btn = Btnp::default();
        let frames: Vec<i32> = (1..=30).filter(|_| btn.press(true)).collect();
        assert_eq!(frames, vec![1, 16, 20, 24, 28]);
        assert!(!btn.press(false), "a released button is not pressed");
        assert_eq!(btn.held(), 0);
        assert!(btn.press(true), "and the next press is fresh again");
    }

    /// Walk from the title into the coin drop.
    fn to_coin(app: &mut App) {
        settle(app);
        tap(app, pressed());
        assert_eq!(app.scene, Scene::Coin);
    }

    #[test]
    fn the_coin_waits_sixty_frames_then_follows_its_curve() {
        let mut app = App::new(7);
        to_coin(&mut app);
        let start = app.coin.unwrap().pos;
        while app.game.t < 60 {
            app.step(&still());
        }
        assert_eq!(app.coin.unwrap().frac, 0, "it is still held still");
        app.step(&still());
        let coin = app.coin.unwrap();
        assert_eq!(app.game.t, 61);
        assert_eq!(coin.frac, COIN_STEP);
        assert_ne!(coin.pos, start, "the bezier has moved it");
    }

    #[test]
    fn the_coin_shrinks_by_two_percent_a_frame_past_halfway() {
        let mut app = App::new(7);
        to_coin(&mut app);
        let mut before = None;
        while let Some(coin) = app.coin {
            if coin.frac > HALF {
                before = Some(coin.size);
                break;
            }
            app.step(&still());
        }
        let before = before.expect("the coin gets past halfway");
        app.step(&still());
        let now = app.coin.expect("still falling").size;
        assert_eq!(now, mul(before, COIN_SHRINK));
    }

    #[test]
    fn the_coin_lands_then_the_machine_rumbles_for_fifteen_frames() {
        let mut app = App::new(7);
        to_coin(&mut app);
        let mut frames = 0;
        while app.coin.is_some() {
            app.step(&still());
            frames += 1;
            assert!(frames < 400, "the coin never landed");
        }
        // The landing frame already spent one of the fifteen (the cart decrements
        // in the same frame it arms the rumble).
        assert_eq!(app.rumble_t, 14);
        let mut rumbles = 0;
        while app.rumble {
            assert_eq!(app.hero, Hero::Idle, "the kid stops posing while it shakes");
            app.step(&still());
            rumbles += 1;
            assert!(rumbles <= 15, "the rumble never stopped");
        }
        assert_eq!(rumbles, 15);
        app.step(&still());
        assert_eq!(app.game.zoom, Zoom::In, "and then the zoom begins");
    }

    #[test]
    fn the_zoom_takes_120_frames_and_deals_the_first_round() {
        let mut app = App::new(7);
        to_coin(&mut app);
        let mut frames = 0;
        while app.scene == Scene::Coin {
            app.step(&still());
            frames += 1;
            assert!(frames < 900, "the zoom never finished");
        }
        assert_eq!(app.scene, Scene::Game);
        assert_eq!(app.game.zoom, Zoom::Out);
        assert_eq!(app.game.trans_t, 120);
        assert_eq!(app.game.board.0.len(), 61, "the first level is dealt");
        assert_eq!(app.game.bubs.len(), 1, "and the gun is loaded");
    }

    #[test]
    fn the_zoom_unwinds_while_the_first_round_plays() {
        let mut app = App::new(7);
        to_coin(&mut app);
        while app.scene == Scene::Coin {
            app.step(&still());
        }
        assert!(app.game.trans_acc <= HALF, "the zoom is at its limit");
        for _ in 0..400 {
            app.step(&still());
        }
        assert_eq!(app.game.trans_t, 0, "back to the normal view");
        assert_eq!(app.game.trans_acc, crate::fixed::int(50));
    }

    #[test]
    fn a_game_over_fades_the_cascade_out_then_shows_the_score_screen() {
        let mut app = App::new(7);
        // Straight into the game, as "retry" does.
        app.game.new_game();
        app.game.fade.finish_in();
        app.scene = Scene::Game;
        app.game.take_music();

        app.game.board.0.insert(270, 3);
        let mut frames = 0;
        while app.scene != Scene::Over {
            app.step(&still());
            frames += 1;
            assert!(frames < 200, "the game never ended");
        }
        assert!(app.game.over);

        while app.scene != Scene::Final {
            app.step(&still());
            frames += 1;
            assert!(frames < 1_000, "the score screen never came");
        }
        assert_eq!(app.nav, &FINAL_NAV[..]);
        assert_eq!(app.focus, 1);
        assert!(
            app.game.take_music().contains(&24),
            "the score track is queued"
        );
        assert!(app.gumballs >= 0);
    }

    /// Drop the app on the score screen with a given score, without playing for it.
    fn to_final(app: &mut App, score: i32) {
        app.game.new_game();
        app.game.score = score;
        app.game.fade.finish_in();
        app.scene = Scene::Final;
        app.nav = &FINAL_NAV;
        app.focus = 1;
        app.finalani = 1;
        app.gumballs = score / 100;
        app.blow_force = app.gumballs;
    }

    #[test]
    fn a_zero_score_goes_straight_to_the_menu() {
        let mut app = App::new(7);
        to_final(&mut app, 0);
        app.step(&still());
        assert_eq!(app.finalani, 4, "nothing to dispense, so no animation");
        assert!(app.score_bubs.is_empty());
    }

    #[test]
    fn the_machine_dispenses_one_gumball_per_hundred_points() {
        let mut app = App::new(7);
        to_final(&mut app, 500);
        for _ in 0..5 {
            app.step(&still());
        }
        assert_eq!(app.gumballs, 0, "five gumballs for 500 points");
        assert_eq!(app.score_bubs.len(), 5);
        assert_eq!(app.hero, Hero::Nom);
        assert!(app.rumble);

        for _ in 0..200 {
            if app.finalani == 2 {
                break;
            }
            app.step(&still());
        }
        assert_eq!(app.finalani, 2, "the arcs finish and the kid chews");
        assert!(app.score_bubs.is_empty());
    }

    #[test]
    fn the_kid_chews_for_120_frames_then_blows_the_bubble() {
        let mut app = App::new(7);
        to_final(&mut app, 100);
        for _ in 0..300 {
            if app.finalani == 2 {
                break;
            }
            app.step(&still());
        }
        assert_eq!(app.finalani, 2);
        app.step(&still());
        assert_eq!(app.hero, Hero::Chew, "the next frame switches the pose");
        let mut chewed = 0;
        while app.finalani == 2 {
            app.step(&still());
            chewed += 1;
            assert!(chewed < 200);
        }
        // `event_t` is already 1 on the frame the state changed, so 120 more
        // frames pass before the cart's `event_t > 120` fires.
        assert_eq!(chewed, 120);
        assert_eq!(app.finalani, 3);
    }

    #[test]
    fn the_blown_bubble_grows_once_per_punishment_then_the_menu_opens() {
        let mut app = App::new(7);
        to_final(&mut app, 300);
        app.finalani = 3;
        let before = app.blow_r;
        app.step(&still());
        assert_eq!(app.blow_r, before + 13_107);
        assert!(app.blowing);
        let mut frames = 0;
        while app.finalani == 3 {
            app.step(&still());
            frames += 1;
            assert!(frames < 10);
        }
        assert_eq!(app.finalani, 4, "and then the score menu appears");
        assert_eq!(app.blow_force, 0);
    }

    #[test]
    fn retry_fades_the_score_screen_out_then_deals_a_fresh_game() {
        let mut app = App::new(7);
        to_final(&mut app, 0);
        app.step(&still());
        assert_eq!(app.finalani, 4);

        tap(&mut app, pressed());
        assert_eq!(app.game.fade.state, crate::sim::spin::FadeState::Out);
        let mut frames = 0;
        while app.scene == Scene::Final {
            app.step(&still());
            frames += 1;
            assert!(frames < 200, "the fade-out never finished");
        }
        assert_eq!(app.scene, Scene::Game);
        assert_eq!(app.game.board.0.len(), 61, "a fresh board");
        assert_eq!(app.game.score, 0);
        assert!(
            app.game.save.completed,
            "the score screen marks the cart done"
        );
    }

    #[test]
    fn return_to_title_fades_out_and_rebuilds_the_title_screen() {
        let mut app = App::new(7);
        to_final(&mut app, 0);
        app.step(&still());
        tap(&mut app, down());
        assert_eq!(app.focus, 2);
        app.game.take_music();
        tap(&mut app, pressed());
        let mut frames = 0;
        while app.scene == Scene::Final {
            app.step(&still());
            frames += 1;
            assert!(frames < 200);
        }
        assert_eq!(app.scene, Scene::Main);
        assert_eq!(app.nav, &MAIN_NAV[..]);
        assert_eq!(app.focus, 1);
        assert!(app.game.take_music().contains(&0), "the title track again");
    }
}
