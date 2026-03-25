#![deny(
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    reason = "the main program should not be doing anything unsafe without good reason. \
    if the unsafety is to interact with windows.h, do it in win.rs"
)]
#![cfg_attr(
    not(debug_assertions),
    deny(clippy::unwrap_used, reason = "prefer to give more information")
)]
#![warn(clippy::missing_panics_doc, clippy::missing_assert_message)]
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
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
        nonpoison::{Condvar, Mutex},
    },
    thread::sleep,
    time::Duration,
};

mod input;
mod output;

mod comp_vis;
mod data;
mod game_state;
mod win;

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
        std::sync::LazyLock::new(|| SYS_BTN_COLOR.normalized().normalized());

    pub(super) const CAM_BTN_COLOR: ColorRGB = ColorRGB {
        r: 136,
        g: 172,
        b: 0,
    };

    pub static CAM_BTN_COLOR_NRM: std::sync::LazyLock<CNorm> =
        std::sync::LazyLock::new(|| CAM_BTN_COLOR.normalized().normalized());
}

fn main() {
    let winh = WindowsHandles::new();

    let screen_data = Arc::new(ScreenDataPair {
        buffer: Mutex::new(ScreenData::new(
            vec![0; CHANNELS_PER_COLOR * winh.screen_width as usize * winh.screen_height as usize],
            winh.screen_width,
        )),
        counter: Mutex::new(0),
        updated: Condvar::new(),
    });

    let threads_should_loop = AtomicBool::new(true);

    std::thread::scope(|s| {
        if let Err(e) = winh.bitblt(screen_data.buffer.lock().data_mut()) {
            eprintln!("failed to update screencap for first time: {e}");
            threads_should_loop.store(false, Relaxed);
            return;
        }

        // Make sure that user control override doesn't disable the user from closing the program
        let user_guard_thread = std::thread::Builder::new()
            .name("user guard".to_string())
            .spawn_scoped(s, || {
                // !! SAFETY !!
                while threads_should_loop.load(Relaxed) {
                    sleep(Duration::from_millis(2)); // Give the user time to provide input

                    if is_key_down(VirtualKey::Esc) {
                        // mask to ignore the "toggled" bit
                        println!("User has chosen to reclaim control. Task ended.");
                        threads_should_loop.store(false, Relaxed); // This tells the worker threads to stop
                    }
                }
            });

        if let Err(e) = user_guard_thread {
            eprintln!("user guard thread failed to spawn: {e}");
            threads_should_loop.store(false, Relaxed);
            return;
        }

        // Spawn a thread for acting on that data
        let game_state_thread = std::thread::Builder::new()
            .name("game state".to_string())
            .spawn_scoped(s, || {
                // All the information we have about the state of the game
                let mut game_state = GameState::<1024>::new(Arc::clone(&screen_data));
                let (mut rl, thread) = init()
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

                while threads_should_loop.load(Relaxed) {
                    if rl.window_should_close() {
                        threads_should_loop.store(false, Relaxed);
                        println!("User has chosen to reclaim control. Task ended.");
                        break;
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
            threads_should_loop.store(false, Relaxed);
            return;
        }

        let mut buffer = vec![0; screen_data.buffer.lock().len()];

        // Read screen pixels on the current thread so that handles don't risk going on the wrong thread
        while threads_should_loop.load(Relaxed) {
            // Update our internal copy of what the gamescreen looks like so we can sample its pixels
            if let Err(e) = winh.bitblt(&mut buffer) {
                eprintln!("failed to update screencap: {e}");
                threads_should_loop.store(false, Relaxed);
                return;
            }
            std::mem::swap(screen_data.buffer.lock().data_mut(), &mut buffer);
            screen_data.mark_updated();
            sleep(Duration::from_millis(8));
        }

        println!("\nWaiting on worker threads...");
        // Wait for threads to safely finish their respective functions before destructing them
    });
    println!("\nWorker threads joined.");
}
