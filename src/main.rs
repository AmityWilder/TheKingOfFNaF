#![deny(
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    reason = "the main program should not be doing anything unsafe without good reason. \
    if the unsafety is to interact with windows.h, do it in win.rs"
)]
#![deny(
    clippy::unwrap_used,
    clippy::panic,
    reason = "panics in multithreading get messy"
)]
#![warn(
    clippy::expect_used,
    clippy::missing_panics_doc,
    clippy::missing_assert_message,
    reason = "you had better be certain panics won't fail"
)]
// #![warn(missing_docs, reason = "if I only work on this once every 5 years, there needs to be clear info about how it works")]
#![warn(
    clippy::missing_const_for_fn,
    reason = "non-const blocks others from being const"
)]
#![allow(dead_code)]
#![feature(sync_nonpoison, nonpoison_condvar, nonpoison_mutex)] // Rather than poisoning, I would like the program to simply end when something goes wrong
#![feature(iter_map_windows)]

use comp_vis::{color::*, *};
use game_state::GameState;
use raylib::prelude::*;
use std::{
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    thread::sleep,
    time::Duration,
};
use vidivici::*;

mod comp_vis;
mod data;
mod game_state;

/// Time it takes for the camera to be ready for input
const CAM_RESP_MS: u16 = 300;

/// Real time
const SECS_PER_MIN: u8 = 60;
/// Game time
const SECS_PER_HOUR: u8 = 45;
const DECISECS_PER_SEC: u8 = 10;
const DECISECS_PER_HOUR: u16 = SECS_PER_HOUR as u16 * DECISECS_PER_SEC as u16;
const MS_PER_DECISEC: u8 = 100;

/// Color constants
mod clr {
    use super::*;

    pub(super) const SYS_BTN_COLOR: ColorRGB = ColorRGB {
        r: 40,
        g: 152,
        b: 120,
    };

    pub static SYS_BTN_COLOR_NRM: std::sync::LazyLock<CNorm> =
        std::sync::LazyLock::new(|| CNorm::from(SYS_BTN_COLOR).normalized());

    pub(super) const CAM_BTN_COLOR: ColorRGB = ColorRGB {
        r: 136,
        g: 172,
        b: 0,
    };

    pub static CAM_BTN_COLOR_NRM: std::sync::LazyLock<CNorm> =
        std::sync::LazyLock::new(|| CNorm::from(CAM_BTN_COLOR).normalized());
}

struct Lifeline<'a>(&'a AtomicBool);

impl Drop for Lifeline<'_> {
    fn drop(&mut self) {
        println!("lifeline cut, telling other threads to shutdown");
        self.0.store(false, Relaxed);
    }
}

impl<'a> Lifeline<'a> {
    const fn new(signal: &'a AtomicBool) -> Self {
        Self(signal)
    }

    #[inline]
    fn load(&self) -> bool {
        self.0.load(Relaxed)
    }
}

fn main() {
    let Ok(mut hshared) = vidivici::init().inspect_err(|e| eprintln!("{e}")) else {
        return;
    };

    #[allow(irrefutable_let_patterns, reason = "refutable in some implementions")]
    let Ok(mut vinput) = hshared.init_vinput().inspect_err(|e| eprintln!("{e}")) else {
        return;
    };
    #[allow(irrefutable_let_patterns, reason = "refutable in some implementions")]
    let Ok(mut uinput) = hshared.init_uinput().inspect_err(|e| eprintln!("{e}")) else {
        return;
    };
    #[allow(irrefutable_let_patterns, reason = "refutable in some implementions")]
    let Ok(mut screen) = hshared.init_screen().inspect_err(|e| eprintln!("{e}")) else {
        return;
    };

    let threads_should_loop = AtomicBool::new(true);

    std::thread::scope(|s| {
        let vision_ll = Lifeline::new(&threads_should_loop);

        if let Err(e) = screen.hint_refresh_screencap() {
            eprintln!("failed to update screencap for first time: {e}");
            return;
        }

        // Spawn a thread for acting on that data
        let game_state_thread = std::thread::Builder::new()
            .name("game state".to_string())
            .spawn_scoped(s, || {
                let game_state_ll = Lifeline::new(&threads_should_loop);
                // All the information we have about the state of the game
                let mut game_state = GameState::<1024>::new();
                let (mut rl, thread) = raylib::init()
                    .size(880, 560)
                    .title("TheKingOfFNaF")
                    .fullscreen()
                    .transparent()
                    .undecorated()
                    .build();

                rl.set_window_state(WindowState::default().set_window_topmost(true));

                let ucn_numbers = match rl.load_texture_from_image(
                    &thread,
                    &match Image::load_image_from_mem(".png", include_bytes!("ucn_numbers.png")) {
                        Ok(x) => x,
                        Err(_) => {
                            eprintln!("failed to load image from bytes");
                            threads_should_loop.store(false, Relaxed);
                            return;
                        }
                    },
                ) {
                    Ok(x) => x,
                    Err(_) => {
                        eprintln!("failed to load texture from image");
                        threads_should_loop.store(false, Relaxed);
                        return;
                    }
                };

                rl.set_target_fps(120);

                let mut is_paused = false;

                while game_state_ll.load() {
                    if rl.window_should_close() {
                        println!("User has chosen to reclaim control. Task ended.");
                        return;
                    }

                    if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                        is_paused = !is_paused;
                    }

                    if !is_paused {
                        // Using the screencap generated on the screen_data thread,
                        // update the game state data for decision making
                        if let Err(e) = game_state.refresh_game_data() {
                            eprintln!("failed to update game state: {e}");
                        }
                    }

                    // Output the data for the user to view
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);

                    game_state.draw_data(
                        &mut d,
                        &thread,
                        &ucn_numbers,
                        10,
                        Color::WHITE,
                        Color::RED,
                    );

                    if !is_paused {
                        // Based upon the game data, perform all actions necessary to return the game to a neutral state
                        if let Err(e) = game_state.act_on_game_data() {
                            eprintln!("failed to update game state: {e}");
                        }
                    }

                    // sleep(Duration::from_millis(4)); // Already done by raylib's EndDrawing()
                }
            });

        if let Err(e) = game_state_thread {
            eprintln!("game state thread failed to spawn: {e}");
            return;
        }

        // Read screen pixels on the current thread so that handles don't risk going on the wrong thread
        while vision_ll.load() {
            if let Err(e) = uinput.hint_check_events() {
                // fatal because the user may have entered the exit key and we should not risk
                // continuing unconsentually under any circumstances
                eprintln!("failed to read user inputs: {e}");
                return;
            }
            // Make sure that user control override doesn't disable the user from closing the program
            if uinput.get_key_state(VirtualKey::Exit).unwrap().is_down() {
                println!("User has chosen to reclaim control. Task ended.");
                return;
            }
            // Update our internal copy of what the gamescreen looks like so we can sample its pixels
            if let Err(e) = screen.hint_refresh_screencap() {
                eprintln!("failed to update screencap: {e}");
                return;
            }
            sleep(Duration::from_millis(8));
        }

        println!("\nWaiting on worker threads...");
        // Wait for threads to safely finish their respective functions before destructing them
    });
    println!("\nWorker threads joined.");
}
