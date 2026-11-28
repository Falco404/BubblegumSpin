//! Native port of *Bubblegum Spin* (PICO-8, v1.1, by Bee_Randon).
//!
//! The crate is split so the rules can be tested without a screen. Game logic
//! never touches a screen; `main.rs` is the only place that talks to the window
//! and the audio device.

pub mod audio;
pub mod board;
pub mod consts;
pub mod fixed;
pub mod gfx;
pub mod grid;
pub mod p8math;
pub mod save;
pub mod sim;
pub mod ui;
