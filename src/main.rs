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
    reason = "you had better be certain this won't fail"
)]
#![allow(dead_code)]
#![feature(sync_nonpoison, nonpoison_condvar, nonpoison_rwlock)] // Rather than poisoning, I would like the program to simply end when something goes wrong

use raylib::prelude as rl;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        nonpoison::{Condvar, RwLock},
    },
    {thread::sleep, time::Duration},
};
use win::*;

mod win;

//
// Global constants -- These give context to unchanging values
//

/// Time it takes for the camera to be ready for input
const CAM_RESP_MS: u16 = 300;

/// Real time
const SECS_PER_MIN: u8 = 60;
/// Game time
const SECS_PER_HOUR: u8 = 45;
const DECISECS_PER_SEC: u8 = 10;
const DECISECS_PER_HOUR: u16 = SECS_PER_HOUR as u16 * DECISECS_PER_SEC as u16;
const MS_PER_DECISEC: u8 = 100;

////////////////////////////////////////////////////
// Here we declare/define the non-primitive types //
////////////////////////////////////////////////////

/// Bitmap channels, not [`ColorRGB`] channels
const CHANNELS_PER_COLOR: usize = 4;

/// Normalized RGB color
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct CNorm {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl CNorm {
    /// Normalize the color like a vector (necessary for performing dot product properly)
    pub fn normalized(&self) -> Self {
        let inv_len: f64 = 1.0 / (self.r * self.r + self.g * self.g + self.b * self.b).sqrt();
        Self {
            r: self.r * inv_len,
            g: self.g * inv_len,
            b: self.b * inv_len,
        }
    }

    /// Better for determining how close a color is to another, regardless of the scale. (brightness/darkness)
    pub const fn dot(&self, rhs: Self) -> f64 {
        self.r * rhs.r + self.g * rhs.g + self.b * rhs.b
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct ColorHSL {
    /// A degree on the color wheel [0..360]
    pub hue: f64,
    /// Percentage of color [0..100]
    pub sat: f64,
    /// Percentage of brightness [0..100]
    pub lum: f64,
}

/// 24-bit RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct ColorRGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRGB {
    pub const fn gray(&self) -> u8 {
        ((self.r as u16 + self.g as u16 + self.b as u16) / 3) as u8
    }

    pub const fn red_dev(&self) -> i32 {
        let dist_from_mean = self.r as i32 - self.gray() as i32;
        ((dist_from_mean * dist_from_mean) / 3).isqrt()
    }

    pub const fn green_dev(&self) -> i32 {
        let dist_from_mean = self.g as i32 - self.gray() as i32;
        ((dist_from_mean * dist_from_mean) / 3).isqrt()
    }

    pub const fn blue_dev(&self) -> i32 {
        let dist_from_mean = self.b as i32 - self.gray() as i32;
        ((dist_from_mean * dist_from_mean) / 3).isqrt()
    }

    // Convert the color components from 0..=255 to 0.0..=1.0
    pub const fn normalized(&self) -> CNorm {
        const INV_BYTE_MAX: f64 = 1.0 / 255.0;
        CNorm {
            r: self.r as f64 * INV_BYTE_MAX,
            g: self.g as f64 * INV_BYTE_MAX,
            b: self.b as f64 * INV_BYTE_MAX,
        }
    }

    pub fn similarity(&self, other: ColorRGB) -> f64 {
        self.normalized()
            .normalized()
            .dot(other.normalized().normalized())
    }
}

impl From<ColorRGB> for ColorHSL {
    fn from(value: ColorRGB) -> Self {
        let col = value.normalized();

        let cmax: f64 = col.r.max(col.g.max(col.b));
        let cmin: f64 = col.r.max(col.g.min(col.b));
        let cmax_cmpnt: i32 = if col.r > col.g {
            if col.r > col.b { 0 } else { 2 }
        } else if col.g > col.b {
            1
        } else {
            2
        };

        let delta = cmax - cmin;

        // Hue
        let hue = if delta == 0.0 {
            0.0
        } else {
            match cmax_cmpnt {
                0 => 60.0 * ((col.g - col.b) / delta),         // Red
                1 => 60.0 * (((col.b - col.r) / delta) + 2.0), // Green
                2 => 60.0 * (((col.r - col.g) / delta) + 4.0), // Blue
                _ => unreachable!(),
            }
        };

        // Lum
        let lum = 0.5 * (cmax + cmin);

        // Sat
        let sat = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * lum - 1.0).abs())
        };

        // Finished
        ColorHSL { hue, sat, lum }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ClockTime {
    /// One hour is 45 seconds. A night is 4 minutes 30 seconds, or 270 seconds -- 2700 deciseconds.
    /// This can be expressed in 12 bits as 0b101010001100.
    deciseconds: u16,
    pings_since_change: u32,
}

impl PartialOrd for ClockTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClockTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deciseconds.cmp(&other.deciseconds)
    }
}

impl Default for ClockTime {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ClockTime {
    pub const fn new(deciseconds: u16) -> Self {
        Self {
            deciseconds,
            pings_since_change: 0,
        }
    }

    pub const fn deciseconds(&self) -> u16 {
        self.deciseconds
    }

    pub const fn pings_since_change(&self) -> u32 {
        self.pings_since_change
    }

    /// It takes 1 bit more than a char to describe the number of seconds in a night.
    pub const fn seconds(&self) -> u16 {
        self.deciseconds / DECISECS_PER_SEC as u16
    }

    /// Not sure what we'd need this for, but just in case.
    pub const fn minutes(&self) -> u16 {
        self.seconds() / SECS_PER_MIN as u16 // realtime
    }

    /// What hour of the night we are at
    pub const fn hour(&self) -> u16 {
        self.seconds() / SECS_PER_HOUR as u16 // gametime
    }

    /// Converts hours to deciseconds, for finding how many deciseconds we are through the current hour.
    pub const fn whole_hour_deciseconds(&self) -> u16 {
        self.hour() * DECISECS_PER_HOUR
    }

    /// Finds how many deciseconds into the current hour we are.
    pub const fn deciseconds_since_hour(&self) -> u16 {
        self.deciseconds() - self.whole_hour_deciseconds()
    }

    /// Updates [`Self::pings_since_change`] if the time is a duplicate or erroneous
    pub const fn update_time(&mut self, new_time: u16) {
        if new_time > self.deciseconds
            && new_time < 6000
            && new_time > 0
            && ((new_time - self.deciseconds) < 10 || self.pings_since_change > 10)
        {
            self.deciseconds = new_time;
            self.pings_since_change = 0;
        } else {
            self.pings_since_change += 1;
        }
    }
}

impl std::fmt::Display for ClockTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{:02}.{}",
            self.minutes(),
            self.seconds() % SECS_PER_MIN as u16,
            self.deciseconds() % DECISECS_PER_SEC as u16
        )
    }
}

// What gamestate we are in (what we can see on the screen)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    Camera,
    Vent,
    Duct,
    Office,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Camera => "Camera",
            State::Vent => "Vent",
            State::Duct => "Duct",
            State::Office => "Office",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Camera {
    WestHall,
    EastHall,
    Closet,
    Kitchen,
    PirateCove,
    ShowtimeStage,
    PrizeCounter,
    PartsAndServices,
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Camera::WestHall => "West hall",
            Camera::EastHall => "East hall",
            Camera::Closet => "Closet",
            Camera::Kitchen => "Kitchen",
            Camera::PirateCove => "Pirate cove",
            Camera::ShowtimeStage => "Showtime stage",
            Camera::PrizeCounter => "Prize counter",
            Camera::PartsAndServices => "Parts and services",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Vent {
    /// Snares reset after being tripped
    #[default]
    Inactive,
    WestSnare,
    NorthSnare,
    EastSnare,
}

impl std::fmt::Display for Vent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vent::Inactive => "Inactive",
            Vent::WestSnare => "West snare",
            Vent::NorthSnare => "North snare",
            Vent::EastSnare => "East snare",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Duct {
    West,
    East,
}

impl std::fmt::Display for Duct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Duct::West => "West",
            Duct::East => "East",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct OfficeData {
    /// How far left/right we are looking [-1,1]
    pub office_yaw: f64,
}

impl OfficeData {
    pub const MASK_POS: POINT = POINT { x: 682, y: 1006 };
    /// The office version of this constant
    pub const VENT_WARNING_POS: POINT = POINT { x: 1580, y: 1040 };

    pub const _FOXY: POINT = POINT { x: 801, y: 710 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CameraData {
    /// Which camera we are looking at
    pub camera: Camera,
}

impl CameraData {
    /// Location for testing vent warning in the cameras
    pub const VENT_WARNING_POS: POINT = POINT { x: 1563, y: 892 };
    /// Where the reset vent button is for clicking
    pub const RESET_VENT_BTN_POS: POINT = POINT { x: 1700, y: 915 };

    /// WestHall
    pub const CAM_01_POS: POINT = POINT { x: 1133, y: 903 };
    /// EastHall
    pub const CAM_02_POS: POINT = POINT { x: 1382, y: 903 };
    /// Closet
    pub const CAM_03_POS: POINT = POINT { x: 1067, y: 825 };
    /// Kitchen
    pub const CAM_04_POS: POINT = POINT { x: 1491, y: 765 };
    /// PirateCove
    pub const CAM_05_POS: POINT = POINT { x: 1122, y: 670 };
    /// ShowtimeStage
    pub const CAM_06_POS: POINT = POINT { x: 1422, y: 590 };
    /// PrizeCounter
    pub const CAM_07_POS: POINT = POINT { x: 1278, y: 503 };
    /// PartsAndServices
    pub const CAM_08_POS: POINT = POINT { x: 988, y: 495 };

    /// System buttons X position
    pub const SYS_BTN_X: i32 = 1331;
    /// Cam system button Y position
    pub const CAM_SYS_BTN_Y: i32 = 153;
    /// Vent system button Y position
    pub const VENT_SYS_BTN_Y: i32 = 263;
    /// Duct system button Y position
    pub const DUCT_SYS_BTN_Y: i32 = 373;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct VentData {
    /// Which vent snare is active
    pub vent_snare: Vent,
}

impl VentData {
    /// Left snare
    pub const SNARE_L_POS: POINT = POINT { x: 548, y: 645 };
    /// Top snare
    pub const SNARE_T_POS: POINT = POINT { x: 650, y: 536 };
    /// Right snare
    pub const SNARE_R_POS: POINT = POINT { x: 747, y: 645 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DuctData {
    /// Which duct is currently closed
    pub closed_duct: Duct,
    pub audio_lure: POINT,
}

impl DuctData {
    /// Check left duct
    pub const _DUCT_L_POS: POINT = POINT { x: 500, y: 791 };
    /// Check right duct
    pub const _DUCT_R_POS: POINT = POINT { x: 777, y: 791 };
    /// Left duct button
    pub const DUCT_L_BTN_POS: POINT = POINT { x: 331, y: 844 };
    /// Right duct button
    pub const DUCT_R_BTN_POS: POINT = POINT { x: 1016, y: 844 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct ClockTimeResult {
    pub last_trusted: ClockTime,
    pub error: [Option<ReadNumberError>; 4],
}

impl std::ops::Deref for ClockTimeResult {
    type Target = ClockTime;

    fn deref(&self) -> &Self::Target {
        &self.last_trusted
    }
}

impl std::ops::DerefMut for ClockTimeResult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.last_trusted
    }
}

impl From<ClockTime> for ClockTimeResult {
    fn from(value: ClockTime) -> Self {
        ClockTimeResult {
            last_trusted: value,
            error: [None; 4],
        }
    }
}

/// This is the type which actually stores the data we have about the gamestate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct GameData {
    flags: u8,
    pub time: ClockTimeResult,
    pub next_ff_show: ClockTime, // needs a different read_number function
}

impl GameData {
    const VENTILATION_NEEDS_RESET_FLAG: u8 = 1;
    const FLASHLIGHT_FLAG: u8 = Self::VENTILATION_NEEDS_RESET_FLAG << 1;
    /// in order from left to right
    const DOOR0_CLOSED_FLAG: u8 = Self::FLASHLIGHT_FLAG << 1;

    const fn new() -> Self {
        Self {
            flags: 0,
            time: ClockTimeResult {
                last_trusted: ClockTime::new(0),
                error: [None; 4],
            },
            next_ff_show: ClockTime::new(0),
        }
    }

    pub const fn is_ventilation_reset_needed(&self) -> bool {
        (self.flags & Self::VENTILATION_NEEDS_RESET_FLAG) != 0
    }
    pub const fn mark_ventilation_has_been_reset(&mut self) {
        self.flags &= !Self::VENTILATION_NEEDS_RESET_FLAG;
    }
    pub const fn mark_ventilation_needs_reset(&mut self) {
        self.flags |= Self::VENTILATION_NEEDS_RESET_FLAG;
    }

    pub const fn is_flashlight_on(&self) -> bool {
        (self.flags & Self::FLASHLIGHT_FLAG) != 0
    }
    pub const fn mark_flashlight_off(&mut self) {
        self.flags &= !Self::FLASHLIGHT_FLAG;
    }
    pub const fn mark_flashlight_on(&mut self) {
        self.flags |= Self::FLASHLIGHT_FLAG;
    }

    pub const fn is_door_closed(&self, door: i32) -> bool {
        (self.flags & Self::DOOR0_CLOSED_FLAG << door) != 0
    }
    pub const fn mark_door_open(&mut self, door: i32) {
        self.flags &= !(Self::DOOR0_CLOSED_FLAG << door);
    }
    pub const fn mark_door_closed(&mut self, door: i32) {
        self.flags |= Self::DOOR0_CLOSED_FLAG << door;
    }
}

/// Important positions on the screen
impl GameData {
    /// Clock position
    pub const CLK_POS: POINT = POINT { x: 1807, y: 85 };
    pub const CLK_10SEC_X: i32 = 1832;
    pub const CLK_SEC_X: i32 = 1849;
    pub const CLK_DECISEC_X: i32 = 1873;

    pub const _TEMPERATURE_POS: POINT = POINT { x: 1818, y: 1012 };

    pub const _COINS: POINT = POINT { x: 155, y: 75 };

    pub const _PWR_POS: POINT = POINT { x: 71, y: 910 };
    pub const _PWR_USG_POS: POINT = POINT { x: 38, y: 969 };

    pub const _NOISE_POS: POINT = POINT { x: 38, y: 1020 };

    /// Not really needed since the S key exists...
    pub const _OPEN_CAM_POS: POINT = POINT { x: 1280, y: 1006 };
}

/// What state we are in (office, checking cameras, ducts, vents)
/// The metadata about the state (what part of the office, which camera)
/// Information about the current state that can tell us how to interpret information
#[derive(Debug, Clone, Copy, PartialEq)]
enum StateData {
    Office(OfficeData),
    Camera(CameraData),
    Vent(VentData),
    Duct(DuctData),
}

impl StateData {
    pub const fn state(&self) -> State {
        match self {
            StateData::Office(_) => State::Office,
            StateData::Camera(_) => State::Camera,
            StateData::Vent(_) => State::Vent,
            StateData::Duct(_) => State::Duct,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GameState {
    pub state: StateData,
    pub game: GameData,
}

impl GameState {
    pub const fn new() -> Self {
        Self {
            state: StateData::Office(OfficeData { office_yaw: 0.0 }),
            game: GameData::new(),
        }
    }

    pub const fn state(&self) -> State {
        self.state.state()
    }

    #[allow(clippy::too_many_arguments, reason = "dont care")]
    pub fn draw_data(
        &self,
        d: &mut rl::RaylibDrawHandle,
        _thread: &rl::RaylibThread,
        screen_data: &RwLock<ScreenData>,
        ucn_numbers: &rl::Texture2D,
        font_size: i32,
        color: rl::Color,
        err_color: rl::Color,
    ) {
        use raylib::prelude::*;

        let line_space = font_size + 4;
        let mut y = 0;

        let time_str = &format!("Time: {}", *self.game.time);
        d.draw_text(time_str, 0, y, font_size, color);
        let mut y1 = y;
        let mut x = d.measure_text(time_str, font_size) + font_size;
        if self.game.time.error.iter().any(|e| e.is_some()) {
            const ERR_STR: &str = "error: unrecognized digit(s)";
            d.draw_text(ERR_STR, x, y, font_size, err_color);
            x += d.measure_text(ERR_STR, font_size);
        }
        for e in self.game.time.error.iter().flatten() {
            const SCALE: i32 = 4;
            const DIGIT_WIDTH: i32 = 11 + 1;
            const DIGIT_HEIGHT: i32 = 14 + 1;
            match e {
                ReadNumberError::UnknownSequence { flags } => {
                    d.draw_texture_ex(
                        ucn_numbers,
                        rvec2(x + DIGIT_WIDTH * SCALE, y1),
                        0.0,
                        SCALE as f32,
                        Color::GRAY,
                    );
                    for (i, expect_flags) in std::iter::once(u8::MAX)
                        .chain(READ_NUMBER_SAMPLE_FLAGS)
                        .enumerate()
                    {
                        let digit_x = i as i32 * DIGIT_WIDTH * SCALE;
                        for (flag_matches, offset) in READ_NUMBER_SAMPLE_OFFSETS
                            .into_iter()
                            .enumerate()
                            .map(|(j, offset)| {
                                (((flags >> j) & 1) == ((expect_flags >> j) & 1), offset)
                            })
                        {
                            d.draw_rectangle(
                                x + SCALE * offset.x + digit_x,
                                y1 + SCALE * offset.y,
                                SCALE,
                                SCALE,
                                if expect_flags == u8::MAX {
                                    if flag_matches {
                                        Color::WHITE
                                    } else {
                                        Color::BLUEVIOLET
                                    }
                                } else if flag_matches {
                                    Color::GREEN
                                } else {
                                    Color::RED
                                },
                            );
                        }
                    }
                }
            }
            y1 += (DIGIT_HEIGHT + 1) * SCALE;
        }
        y += line_space;
        d.draw_text(
            &format!(
                "Ventilation: {}",
                if self.game.is_ventilation_reset_needed() {
                    "WARNING"
                } else {
                    "good"
                }
            ),
            0,
            y,
            font_size,
            color,
        );
        y += line_space;
        for door in ["Left door", "Front vent", "Right door", "Right vent"] {
            d.draw_text(
                &format!(
                    "{door}: {}",
                    if self.game.is_door_closed(0) {
                        "closed"
                    } else {
                        "open"
                    }
                ),
                0,
                y,
                font_size,
                color,
            );
            y += line_space;
        }
        d.draw_text(
            &format!(
                "Flashlight: {}",
                if self.game.is_flashlight_on() {
                    "on"
                } else {
                    "off"
                }
            ),
            0,
            y,
            font_size,
            color,
        );
        y += line_space;
        d.draw_text(
            &format!("Next Funtime Foxy show: {}", self.game.next_ff_show),
            0,
            y,
            font_size,
            color,
        );
        y += line_space;

        const STATES: [State; 4] = [State::Camera, State::Vent, State::Duct, State::Office];

        let mut x = font_size;
        for s in STATES {
            let name = &s.to_string();
            let width = d.measure_text(name, font_size);
            d.draw_rectangle(
                x - font_size,
                y - 1,
                width + 2 * font_size,
                font_size + 2,
                Color::BLUEVIOLET,
            );
            if s == self.state() {
                d.draw_rectangle(
                    x - font_size / 2,
                    y - 1,
                    width + font_size,
                    font_size + 2,
                    Color::BLUE,
                );
            }
            d.draw_text(name, x, y, font_size, color);
            x += width + font_size;
        }
        y += line_space;

        match &self.state {
            StateData::Camera(cd) => {
                d.draw_text(
                    &format!(
                        "Looking at: CAM 0{} | {}",
                        (cd.camera as i32 + 1),
                        cd.camera
                    ),
                    0,
                    y,
                    font_size,
                    color,
                );
            }

            StateData::Office(od) => {
                d.draw_text(
                    &format!(
                        "Yaw: {}\nNightmare Balloon Boy: {}",
                        od.office_yaw,
                        if is_nmbb_standing(&screen_data.read()) {
                            "standing"
                        } else {
                            "sitting"
                        }
                    ),
                    0,
                    y,
                    font_size,
                    color,
                );
            }

            _ => d.draw_text("TODO", 0, y, font_size, color),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////
// Here, all of the variables & constants used throughout the project are declared/defined //
/////////////////////////////////////////////////////////////////////////////////////////////

//
// Global variables -- used by everyone, but changeable.
//

///////////////////////////////////////////////
// This is where we take input from the game //
// e.g.                                      //
// - Test pixel color at { 253, 1004 }       //
///////////////////////////////////////////////

#[derive(Debug)]
struct ScreenData {
    data: Vec<u8>,
    width: i32,
}

impl ScreenData {
    pub const fn new(data: Vec<u8>, width: i32) -> Self {
        Self { data, width }
    }

    pub fn update_screencap(&mut self, winh: &mut WindowsHandles) -> WindowsResult<()> {
        winh.bitblt(&mut self.data)
    }

    pub fn pixel_color_at(&self, pos: POINT) -> ColorRGB {
        let index: usize = CHANNELS_PER_COLOR * ((pos.y * self.width) + pos.x) as usize;

        ColorRGB {
            r: self.data[index + 2],
            g: self.data[index + 1],
            b: self.data[index],
        }
    }
}

/// These enable us to put the buttons in an array and choose from them instead of just using the literal names
/// If you're trying to get the position of just the one thing and don't need to do any sort of "switch" thing, please don't use this. It adds additional steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Button {
    Mask,
    ResetVent,

    /// WestHall
    Cam01,
    /// EastHall
    Cam02,
    /// Closet
    Cam03,
    /// Kitchen
    Cam04,
    /// PirateCove
    Cam05,
    /// ShowtimeStage
    Cam06,
    /// PrizeCounter
    Cam07,
    /// PartsAndServices
    Cam08,

    CameraSystem,
    VentSystem,
    DuctSystem,

    SnareLeft,
    SnareTop,
    SnareRight,

    DuctLeft,
    DuctRight,
}

impl From<Camera> for Button {
    fn from(value: Camera) -> Self {
        match value {
            Camera::WestHall => Button::Cam01,
            Camera::EastHall => Button::Cam02,
            Camera::Closet => Button::Cam03,
            Camera::Kitchen => Button::Cam04,
            Camera::PirateCove => Button::Cam05,
            Camera::ShowtimeStage => Button::Cam06,
            Camera::PrizeCounter => Button::Cam07,
            Camera::PartsAndServices => Button::Cam08,
        }
    }
}

impl From<State> for Button {
    fn from(value: State) -> Self {
        match value {
            State::Camera => Button::CameraSystem,
            State::Vent => Button::VentSystem,
            State::Duct => Button::DuctSystem,
            State::Office => todo!("there is no \"office system\" button"),
        }
    }
}

// This is the list the above enum was referring to
const BTN_POSITIONS: [POINT; 18] = [
    OfficeData::MASK_POS,
    CameraData::RESET_VENT_BTN_POS,
    CameraData::CAM_01_POS,
    CameraData::CAM_02_POS,
    CameraData::CAM_03_POS,
    CameraData::CAM_04_POS,
    CameraData::CAM_05_POS,
    CameraData::CAM_06_POS,
    CameraData::CAM_07_POS,
    CameraData::CAM_08_POS,
    POINT {
        x: CameraData::SYS_BTN_X,
        y: CameraData::CAM_SYS_BTN_Y,
    },
    POINT {
        x: CameraData::SYS_BTN_X,
        y: CameraData::VENT_SYS_BTN_Y,
    },
    POINT {
        x: CameraData::SYS_BTN_X,
        y: CameraData::DUCT_SYS_BTN_Y,
    },
    VentData::SNARE_L_POS,
    VentData::SNARE_T_POS,
    VentData::SNARE_R_POS,
    DuctData::DUCT_L_BTN_POS,
    DuctData::DUCT_R_BTN_POS,
];

impl Button {
    /// Pick the button position from the list of button positions
    fn pos(self) -> POINT {
        BTN_POSITIONS[self as usize]
    }
}

////////////////////////////////////////////////////////////////////////////////////
// This is where the input we've taken from the game gets turned into useful data //
////////////////////////////////////////////////////////////////////////////////////

fn is_nmbb_standing(screen_data: &ScreenData) -> bool {
    const PANTS_COLOR: ColorRGB = ColorRGB {
        r: 0,
        g: 28,
        b: 120,
    };
    const SAMPLE_POS: POINT = POINT { x: 1024, y: 774 };
    const THRESHOLD: f64 = 0.98;
    PANTS_COLOR.similarity(screen_data.pixel_color_at(SAMPLE_POS)) > THRESHOLD
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ReadNumberError {
    UnknownSequence { flags: u8 },
}

impl std::fmt::Display for ReadNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadNumberError::UnknownSequence { .. } => {
                write!(f, "unrecognized combination of pixels")
            }
        }
    }
}

impl std::error::Error for ReadNumberError {}

const READ_NUMBER_SAMPLE_OFFSETS: [POINT; 8] = [
    POINT { x: 5, y: 0 },  // top middle
    POINT { x: 0, y: 7 },  // upper-middle left
    POINT { x: 10, y: 7 }, // upper-middle right
    POINT { x: 5, y: 8 },  // lower-middle middle
    POINT { x: 0, y: 8 },  // lower-middle left
    POINT { x: 10, y: 8 }, // lower-middle right
    POINT { x: 0, y: 12 }, // bottom left
    POINT { x: 5, y: 12 }, // bottom middle
];

const READ_NUMBER_SAMPLE_FLAGS: [u8; 10] = [
    0b10110111, 0b10001001, 0b11001001, 0b10000001, 0b00111010, 0b10100101, 0b10110011, 0b00001001,
    0b10110001, 0b10000101,
];

impl ScreenData {
    /// Input should be top-left corner of the number followed by the size
    fn read_number(&self, x: i32, y: i32) -> Result<u8, ReadNumberError> {
        const THRESHOLD: u8 = 100; // Minimum brightness value of the pixel

        let mut guess_bitflags: u8 = 0;
        for (sample, offset) in READ_NUMBER_SAMPLE_OFFSETS.iter().enumerate() {
            let sample_pos = POINT {
                x: x + offset.x,
                y: y + offset.y,
            };
            if self.pixel_color_at(sample_pos).gray() > THRESHOLD {
                guess_bitflags |= 1 << sample;
            }
        }

        READ_NUMBER_SAMPLE_FLAGS
            .into_iter()
            .position(|flags| flags == guess_bitflags)
            .map(|pos| pos as u8)
            .ok_or(ReadNumberError::UnknownSequence {
                flags: guess_bitflags,
            })
    }

    /// Run this about once every frame
    ///
    /// Returns time in deciseconds on success
    pub fn read_game_clock(&self) -> Result<u16, [Option<ReadNumberError>; 4]> {
        let deciseconds = self.read_number(GameData::CLK_DECISEC_X, GameData::CLK_POS.y);

        let seconds = self.read_number(GameData::CLK_SEC_X, GameData::CLK_POS.y);

        let tens_of_seconds = self.read_number(GameData::CLK_10SEC_X, GameData::CLK_POS.y);

        let minute = self.read_number(GameData::CLK_POS.x, GameData::CLK_POS.y);

        if let (Ok(deciseconds), Ok(seconds), Ok(tens_of_seconds), Ok(minute)) =
            (deciseconds, seconds, tens_of_seconds, minute)
        {
            Ok(u16::from(deciseconds)
                + u16::from(DECISECS_PER_SEC)
                    * (u16::from(seconds)
                        + 10 * u16::from(tens_of_seconds)
                        + u16::from(SECS_PER_MIN) * u16::from(minute)))
        } else {
            Err([minute, tens_of_seconds, seconds, deciseconds].map(|res| res.err()))
        }
    }
}

impl GameState {
    pub fn does_ventilation_need_reset(&self, screen_data: &ScreenData) -> bool {
        screen_data
            .pixel_color_at(match self.state() {
                State::Office => OfficeData::VENT_WARNING_POS,
                _ => CameraData::VENT_WARNING_POS,
            })
            .red_dev()
            > 35
    }
}

const fn generate_sample_points(start: POINT, scale: i32) -> [POINT; 5] {
    [
        start,
        POINT {
            x: start.x,
            y: start.y + scale,
        },
        POINT {
            x: start.x + scale,
            y: start.y,
        },
        POINT {
            x: start.x,
            y: start.y - scale,
        },
        POINT {
            x: start.x - scale,
            y: start.y,
        },
    ]
}

impl ScreenData {
    /// - `center`: Point around which to generate the sample points
    /// - `compare`: Normalized color against which to compare the color at the sample points
    /// - `threshold`: 0..1 double value for the minimum similarity required to consider a sample point a "match"
    ///
    /// returns: Total number of sample points which exceeded the threshold
    pub fn test_samples(&self, center: POINT, compare: CNorm, threshold: f64) -> i32 {
        let mut match_count = 0;
        for sample_point in generate_sample_points(center, 4) {
            let sample = self.pixel_color_at(sample_point).normalized();
            if sample.normalized().dot(compare) > threshold {
                match_count += 1;
            }
        }
        match_count
    }

    pub fn test_samples_gray(&self, center: POINT, compare: u8, max_difference: u8) -> i32 {
        let mut match_count: i32 = 0;
        for sample_point in generate_sample_points(center, 4) {
            let sample = self.pixel_color_at(sample_point).gray();
            if sample.abs_diff(compare) > max_difference {
                match_count += 1;
            }
        }
        match_count
    }
}

/// Returns the position of the maximum value
fn max_in_array<I: IntoIterator<Item: PartialOrd>>(it: I) -> Option<usize> {
    let mut it = it.into_iter().enumerate();
    let (mut max_pos, mut max_val) = it.next()?;
    for (pos, item) in it {
        if max_val < item {
            (max_pos, max_val) = (pos, item);
        }
    }
    Some(max_pos)
}

impl OfficeData {
    /// For finding the yaw of the office
    pub fn locate_office_lamp(&mut self, screen_data: &ScreenData) {
        const Y: i32 = 66;
        const THRESHOLD: u8 = 200;
        const START: i32 = 723;
        const WIDTH: i32 = 585;
        for x in START..START + WIDTH {
            if screen_data.pixel_color_at(POINT { x, y: Y }).gray() > THRESHOLD {
                // 100% of the samples must be 80% matching. Flickering be damned.
                if screen_data.test_samples_gray(POINT { x, y: Y }, 255, 20) == 5 {
                    self.office_yaw = (x as f64 - START as f64) / WIDTH as f64;
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateStateError {
    NoMatchingCameraInCameraState,
}

impl std::fmt::Display for UpdateStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateStateError::NoMatchingCameraInCameraState => {
                write!(f, "camera state identified, but no active camera found")
            }
        }
    }
}

impl std::error::Error for UpdateStateError {}

impl GameState {
    pub fn update_state(&mut self, screen_data: &ScreenData) -> Result<(), UpdateStateError> {
        const THRESHOLD: f64 = 0.99;
        let mut new_state = State::Office;
        // List of how many samples returned as matches for each of the buttons being tested
        if let Some((tested_state, _)) = [State::Camera, State::Vent, State::Duct]
            .into_iter()
            .map(|sys_btn| {
                (
                    sys_btn,
                    screen_data.test_samples(
                        Button::from(sys_btn).pos(),
                        *clr::SYS_BTN_COLOR_NRM,
                        THRESHOLD,
                    ),
                )
            })
            .max_by_key(|(_, x)| *x)
            // We must have every sample returning as a match to avoid false positives
            .filter(|(_, matching_samples)| *matching_samples >= 5)
        {
            new_state = tested_state;
        }

        // Update the global state
        self.state = match new_state {
            State::Office => StateData::Office(OfficeData { office_yaw: 0.0 }),

            State::Camera => {
                const CAMERAS: [Camera; 8] = [
                    Camera::WestHall,
                    Camera::EastHall,
                    Camera::Closet,
                    Camera::Kitchen,
                    Camera::PirateCove,
                    Camera::ShowtimeStage,
                    Camera::PrizeCounter,
                    Camera::PartsAndServices,
                ];
                // If we've confirmed the state then there should be no doubt we can identify the camera
                if let Some((camera, _)) = CAMERAS
                    .into_iter()
                    .map(|camera| {
                        (
                            camera,
                            screen_data.test_samples(
                                Button::from(camera).pos(),
                                *clr::CAM_BTN_COLOR_NRM,
                                THRESHOLD,
                            ),
                        )
                    })
                    .max_by_key(|(_, x)| *x)
                    // We must have every sample returning as a match to avoid false positives
                    .filter(|(_, matching_samples)| *matching_samples >= 5)
                {
                    StateData::Camera(CameraData { camera })
                } else {
                    return Err(UpdateStateError::NoMatchingCameraInCameraState);
                }
            }

            State::Vent => StateData::Vent(VentData {
                vent_snare: Vent::default(),
            }),

            State::Duct => StateData::Duct(DuctData {
                closed_duct: Duct::West,
                audio_lure: POINT::default(),
            }),
        };
        Ok(())
    }

    /// Assumes we are already in the office
    pub fn office_look_left(&mut self) {
        assert_eq!(
            self.state(),
            State::Office,
            "cannot look left/right in cameras"
        );
        simulate_mouse_goto(POINT { x: 8, y: 540 });
        sleep(Duration::from_millis(5 * MS_PER_DECISEC as u64));
    }

    /// Assumes we are already in the office
    fn office_look_right(&mut self) {
        assert_eq!(
            self.state(),
            State::Office,
            "cannot look left/right in cameras"
        );
        simulate_mouse_goto(POINT { x: 1910, y: 540 });
        sleep(Duration::from_millis(5 * MS_PER_DECISEC as u64));
    }

    ///////////////////////////////////////////////////////////////////////////
    // This is where basic outputs are combined to make more complex actions //
    ///////////////////////////////////////////////////////////////////////////

    /// Updates all known game information
    pub fn refresh_game_data(
        &mut self,
        screen_data: &RwLock<ScreenData>,
    ) -> Result<(), UpdateStateError> {
        self.update_state(&screen_data.read())?;

        match screen_data.read().read_game_clock() {
            Ok(time) => {
                self.game.time.update_time(time);
                self.game.time.error = [None; 4];
            }
            Err(e) => self.game.time.error = e,
        }

        if self.does_ventilation_need_reset(&screen_data.read()) {
            self.game.mark_ventilation_needs_reset();
        }

        //self.locate_office_lamp(); // Needs work

        Ok(())
    }

    pub fn toggle_monitor(
        &mut self,
        screen_data: &RwLock<ScreenData>,
    ) -> Result<(), UpdateStateError> {
        simulate_keypress(VirtualKey::CameraToggle);
        sleep(Duration::from_millis(CAM_RESP_MS as u64));
        self.update_state(&screen_data.read())
    }

    pub fn open_monitor_if_closed(
        &mut self,
        screen_data: &RwLock<ScreenData>,
    ) -> Result<(), UpdateStateError> {
        if self.state() == State::Office {
            self.toggle_monitor(screen_data)?;
        }
        Ok(())
    }

    // `cam` only used if `state == State::Camera`
    pub fn enter_game_state(
        &mut self,
        new_state: State,
        cam: Camera,
        screen_data: &RwLock<ScreenData>,
    ) -> Result<(), UpdateStateError> {
        let current_state = self.state();
        if current_state != new_state {
            if (current_state == State::Office) != (new_state == State::Office) {
                self.toggle_monitor(screen_data)?;
            }
            match new_state {
                State::Office => {}

                State::Camera => {
                    if let StateData::Camera(cd) = &mut self.state
                        && cd.camera != cam
                    {
                        simulate_mouse_click_at(Button::from(cam).pos());
                    }
                }

                State::Duct => simulate_mouse_click_at(Button::DuctSystem.pos()),
                State::Vent => simulate_mouse_click_at(Button::VentSystem.pos()),
            }
            sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    //
    // Playbook of actions
    //

    pub fn handle_funtime_foxy(
        &mut self,
        screen_data: &RwLock<ScreenData>,
    ) -> Result<(), UpdateStateError> {
        self.open_monitor_if_closed(screen_data)?;
        if self.state() != State::Camera {
            simulate_mouse_click_at(Button::from(State::Camera).pos());
        }
        sleep(Duration::from_millis(1));
        simulate_mouse_click_at(Button::Cam06.pos());
        Ok(())
    }

    pub fn reset_vents(
        &mut self,
        screen_data: &RwLock<ScreenData>,
    ) -> Result<(), UpdateStateError> {
        self.open_monitor_if_closed(screen_data)?; // We don't need to care which system, only that the monitor is up.
        simulate_mouse_click_at(Button::ResetVent.pos());
        self.game.mark_ventilation_has_been_reset();
        sleep(Duration::from_millis(10));
        Ok(())
    }

    pub fn handle_nmbb(&self, screen_data: &RwLock<ScreenData>) {
        sleep(Duration::from_millis(17)); // Wait a little bit to make sure we have time for the screen to change
        // TODO: Wait for next screencap update
        if is_nmbb_standing(&screen_data.read()) {
            // Double check--NMBB will kill us if we flash him wrongfully
            // If he is in fact still up, flash the light on him to put him back down
            simulate_keypress(VirtualKey::Flashlight);
        }
    }

    pub fn act_on_game_data(
        &mut self,
        screen_data: &RwLock<ScreenData>,
    ) -> Result<(), UpdateStateError> {
        /**************************************************************************************************
         * Definitions
         * ===========
         * "Behavioral events" - Some things are not events so much as hints that our current behavior is
         *   unsustainable due to external data. When one of these occurs, we cannot simply 'handle' it
         *   and be done, and must change our behavioral pattern to better suit the needs of the event
         *   until the state has returned to neutral. Thankfully, the behavioral changes are usually
         *   transient and only require temporarily disabling certain systems.
         *
         * "Inturruption events" - Events which give us abrupt notice which we have only a short window to
         *   react to. We don't know ahead of time when these events will occur, and they can trigger
         *   automatically without intervention.
         *
         * "Transition events" - Events triggered by a change in gamestate (like opening or closing the
         *   monitor). These events usually aren't timed and can be done at leisure, but they limit the
         *   actions we can perform without handling them.
         *
         * "Timed events" - Events which are time-sensitive relative to the in-game clock or a countdown.
         *   These are usually long-term and while high-priority, can be done when convenient.
         *
         * "Transient events" - These are events which can be detected & reacted to without any dependence
         *   upon or changes to the current game state.
         *
         * "Distractor events" - Depending on the event, these events can be quick difficult to react to.
         *   They make it much harder to react to other events, and may even take away our control.
         *   Thankfully these events are usually either very short in duration or can be handled by rote.
         **************************************************************************************************/

        if self.state() == State::Office && is_nmbb_standing(&screen_data.read()) {
            self.handle_nmbb(screen_data);
        }

        if self.game.is_ventilation_reset_needed() {
            self.reset_vents(screen_data)?;
        }

        // We have <= 1 seconds before the next hour
        if (DECISECS_PER_HOUR - self.game.time.deciseconds_since_hour())
            <= (DECISECS_PER_SEC as u16 + (CAM_RESP_MS / MS_PER_DECISEC as u16))
        {
            self.handle_funtime_foxy(screen_data)?;
            sleep(Duration::from_millis(10));
        }

        // Lowest priority actions should go down here //

        Ok(())
    }
}

fn main() {
    let mut winh = WindowsHandles::new();

    let screencap_updated: Condvar = Condvar::new();
    let screen_data: RwLock<ScreenData> =
        RwLock::new(ScreenData::new(Vec::new(), winh.screen_width));

    screen_data.write().data.resize(
        CHANNELS_PER_COLOR * winh.screen_width as usize * winh.screen_height as usize,
        0,
    );

    let threads_should_loop: AtomicBool = AtomicBool::new(true);

    std::thread::scope(|s| {
        if let Err(e) = screen_data.write().update_screencap(&mut winh) {
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
                use raylib::prelude::*;

                // All the information we have about the state of the game
                let mut game_state = GameState::new();
                let (mut rl, thread) = init()
                    .size(740, 260)
                    .title("TheKingOfFNaF")
                    .resizable()
                    .build();

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

                rl.set_target_fps(60);

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
                        if let Err(e) = game_state.refresh_game_data(&screen_data) {
                            eprintln!("failed to update game state: {e}");
                        }
                    }

                    // Output the data for the user to view
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);

                    game_state.draw_data(
                        &mut d,
                        &thread,
                        &screen_data,
                        &ucn_numbers,
                        10,
                        Color::WHITE,
                        Color::RED,
                    );

                    if !is_paused {
                        // Based upon the game data, perform all actions necessary to return the game to a neutral state
                        if let Err(e) = game_state.act_on_game_data(&screen_data) {
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

        // Read screen pixels on the current thread so that handles don't risk going on the wrong thread
        while threads_should_loop.load(Relaxed) {
            // Update our internal copy of what the gamescreen looks like so we can sample its pixels
            if let Err(e) = screen_data.write().update_screencap(&mut winh) {
                eprintln!("failed to update screencap: {e}");
                threads_should_loop.store(false, Relaxed);
                return;
            }
            screencap_updated.notify_one();
            sleep(Duration::from_millis(16));
        }

        println!("\nWaiting on worker threads...");
        // Wait for threads to safely finish their respective functions before destructing them
    });
    println!("\nWorker threads joined.");
}
