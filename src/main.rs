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
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
        nonpoison::{Condvar, Mutex},
    },
    thread::sleep,
    time::Duration,
};
use win::*;

use crate::game_state::GameData;

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

/// Tell other threads to stop if this token goes out of scope for any reason
struct Lifeline<'a>(&'a AtomicBool);

impl Drop for Lifeline<'_> {
    fn drop(&mut self) {
        println!("lifeline cut; signalling other threads to join");
        self.0.store(false, Relaxed);
    }
}

fn main() {
    let mut winh = match WindowsHandles::new(c"Ultimate Custom Night") {
        Ok(winh) => winh,
        Err(()) => {
            eprintln!("failed to create Windows handles");
            return;
        }
    };

    let width = winh.rc_client.right.abs_diff(winh.rc_client.left);
    let height = winh.rc_client.top.abs_diff(winh.rc_client.bottom);

    let screen_data = Arc::new(ScreenDataPair {
        buffer: Mutex::new(ScreenData::new(
            winh.lpbitmap.clone(),
            winh.rc_client.right - winh.rc_client.left,
        )),
        counter: Mutex::new(0),
        updated: Condvar::new(),
    });

    let threads_should_loop = AtomicBool::new(true);

    std::thread::scope(|s| {
        let _vision_ll = Lifeline(&threads_should_loop);

        // first screenshot taken in WindowsHandles::new
        winh.swap_buffer(screen_data.buffer.lock().data_mut());
        screen_data.mark_updated();

        // Make sure that user control override doesn't disable the user from closing the program
        let user_guard_thread = std::thread::Builder::new()
            .name("user_guard".to_string())
            .spawn_scoped(s, || {
                let _user_guard_ll = Lifeline(&threads_should_loop);
                // !! SAFETY !!
                while threads_should_loop.load(Relaxed) {
                    sleep(Duration::from_millis(2)); // Give the user time to provide input

                    if is_key_down(VirtualKey::Esc) {
                        // mask to ignore the "toggled" bit
                        println!("User has chosen to reclaim control. Task ended.");
                        break; // _user_guard_ll will tell other threads to stop
                    }
                }
            });

        if let Err(e) = user_guard_thread {
            eprintln!("user guard thread failed to spawn: {e}");
            return;
        }

        // Spawn a thread for acting on that data
        let game_state_thread = std::thread::Builder::new()
            .name("processing".to_string())
            .spawn_scoped(s, || {
                let _processing_ll = Lifeline(&threads_should_loop);

                // All the information we have about the state of the game
                let mut game_state = GameState::<1024>::new(Arc::clone(&screen_data));
                let (mut rl, thread) = init()
                    .title("TheKingOfFNaF")
                    .resizable()
                    .transparent()
                    .undecorated()
                    .build();

                rl.maximize_window();

                rl.set_window_state(
                    WindowState::default()
                        .set_window_topmost(true)
                        .set_window_always_run(true),
                );

                unsafe { ffi::SetWindowState(WindowState::) }

                let mut compensate_for_weird_color_storage = rl.load_shader_from_memory(
                    &thread,
                    None,
                    Some(
                        r"\
#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;

// Output fragment color
out vec4 finalColor;

// NOTE: Add your custom variables here

void main()
{
    // Texel color fetching from texture sampler
    vec4 texelColor = texture(texture0, fragTexCoord);


    // NOTE: Implement here your fragment shader code

    // final color is the color from the texture
    //    times the tint color (colDiffuse)
    //    times the fragment color (interpolated vertex color)
    finalColor = texelColor.bgra*colDiffuse*fragColor;
}
",
                    ),
                );

                let ucn_numbers = match rl.load_texture_from_image(
                    &thread,
                    &match Image::load_image_from_mem(".png", include_bytes!("ucn_numbers.png")) {
                        Ok(x) => x,
                        Err(_) => {
                            eprintln!("failed to load image from bytes");
                            return;
                        }
                    },
                ) {
                    Ok(x) => x,
                    Err(_) => {
                        eprintln!("failed to load texture from image");
                        return;
                    }
                };

                rl.set_target_fps(120);

                let mut rtex = {
                    match rl.load_render_texture(&thread, width, height) {
                        Ok(rtex) => rtex,
                        Err(e) => {
                            eprintln!("failed to load render texture: {e}");
                            return;
                        }
                    }
                };

                let mut is_paused = false;

                while threads_should_loop.load(Relaxed) {
                    if rl.window_should_close() {
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
                    d.clear_background(Color::BLANK);

                    if let Err(e) = rtex.update_texture_rec(
                        rrect(0, 0, width, height),
                        screen_data.buffer.lock().data_mut(),
                    ) {
                        eprintln!("failed to update render texture: {e}");
                        return;
                    }

                    {
                        let mut d = d.begin_shader_mode(&mut compensate_for_weird_color_storage);
                        // d.draw_texture_pro(
                        //     &rtex,
                        //     rrect(0, rtex.height(), rtex.width(), -rtex.height()),
                        //     rrect(0, 0, rtex.width(), rtex.height()),
                        //     Vector2::zero(),
                        //     0.0,
                        //     Color::WHITE,
                        // );
                    }

                    for x in [
                        GameData::CLK_DECISEC_X,
                        GameData::CLK_SEC_X,
                        GameData::CLK_10SEC_X,
                        GameData::CLK_POS.x,
                    ] {
                        d.draw_rectangle_lines(x, GameData::CLK_POS.y, 10, 12, Color::RED);
                    }

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

                    d.draw_fps(0, 600);

                    // sleep(Duration::from_millis(4)); // Already done by raylib's EndDrawing()
                }
            });

        if let Err(e) = game_state_thread {
            eprintln!("game state thread failed to spawn: {e}");
            return;
        }

        // Read screen pixels on the current thread so that handles don't risk going on the wrong thread
        while threads_should_loop.load(Relaxed) {
            // Update our internal copy of what the gamescreen looks like so we can sample its pixels
            if let Err(()) = winh.screenshot() {
                eprintln!("failed to update screencap");
                return;
            }
            winh.swap_buffer(screen_data.buffer.lock().data_mut()); // transfer bits to screen_data so other threads can read them
            screen_data.mark_updated();
            sleep(Duration::from_millis(2));
        }

        println!("Waiting on worker threads...");
        // Wait for threads to safely finish their respective functions before destructing them
    });
    println!("Worker threads joined.");
}
