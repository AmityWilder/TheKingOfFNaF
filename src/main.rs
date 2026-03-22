//
// Global constants -- These give context to unchanging values
//
#![allow(dead_code)]
mod win;
use std::{
    sync::{
        Arc, OnceLock, RwLock,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
    {thread::sleep, time::Duration},
};

use win::*;

const CLEAR_CONSOLE: &str = "\x1b[2J";
const RESET_CURSOR: &str = "\x1b[0;0H";

/// Important positions on the screen
mod pnt {
    use super::*;

    /// Clock position
    pub const CLK_POS: POINT = POINT { x: 1807, y: 85 };
    pub const CLK_10SEC_X: i32 = 1832;
    pub const CLK_SEC_X: i32 = 1849;
    pub const CLK_DECISEC_X: i32 = 1873;

    pub const TEMPERATURE_POS: POINT = POINT { x: 1818, y: 1012 };

    pub const COINS: POINT = POINT { x: 155, y: 75 };

    pub const PWR_POS: POINT = POINT { x: 71, y: 910 };
    pub const PWR_USG_POS: POINT = POINT { x: 38, y: 969 };

    pub const NOISE_POS: POINT = POINT { x: 38, y: 1020 };

    /// Not really needed since the S key exists...
    pub const OPEN_CAM_POS: POINT = POINT { x: 1280, y: 1006 };

    /// Office
    pub mod ofc {
        use super::*;

        pub const MASK_POS: POINT = POINT { x: 682, y: 1006 };
        /// The office version of this constant
        pub const VENT_WARNING_POS: POINT = POINT { x: 1580, y: 1040 };

        pub const FOXY: POINT = POINT { x: 801, y: 710 };
    }

    /// Camera
    pub mod cam {
        use super::*;

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

    /// Vents
    pub mod vnt {
        use super::*;

        /// Left snare
        pub const SNARE_L_POS: POINT = POINT { x: 548, y: 645 };
        /// Top snare
        pub const SNARE_T_POS: POINT = POINT { x: 650, y: 536 };
        /// Right snare
        pub const SNARE_R_POS: POINT = POINT { x: 747, y: 645 };
    }

    /// Ducts
    pub mod dct {
        use super::*;

        /// Check left duct
        pub const DUCT_L_POS: POINT = POINT { x: 500, y: 791 };
        /// Check right duct
        pub const DUCT_R_POS: POINT = POINT { x: 777, y: 791 };
        /// Left duct button
        pub const DUCT_L_BTN_POS: POINT = POINT { x: 331, y: 844 };
        /// Right duct button
        pub const DUCT_R_BTN_POS: POINT = POINT { x: 1016, y: 844 };
    }
}

/// Time it takes for the camera to be ready for input
pub const CAM_RESP_MS: u16 = 300;

/// Real time
pub const SECS_PER_MIN: u8 = 60;
/// Game time
pub const SECS_PER_HOUR: u8 = 45;
pub const DECISECS_PER_SEC: u8 = 10;
pub const DECISECS_PER_HOUR: u16 = SECS_PER_HOUR as u16 * DECISECS_PER_SEC as u16;
pub const MS_PER_DECISEC: u8 = 100;

////////////////////////////////////////////////////
// Here we declare/define the non-primitive types //
////////////////////////////////////////////////////

/// Bitmap channels, not `Color` channels
pub const CHANNELS_PER_COLOR: usize = 4;

/// Normalized RGB color
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CNorm {
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
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
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

    pub fn similarity(&self, other: Color) -> f64 {
        self.normalized()
            .normalized()
            .dot(other.normalized().normalized())
    }
}

impl From<Color> for ColorHSL {
    fn from(value: Color) -> Self {
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
pub mod clr {
    use super::*;

    pub(super) const SYS_BTN_COLOR: Color = Color {
        r: 40,
        g: 152,
        b: 120,
    };

    pub static SYS_BTN_COLOR_NRM: std::sync::LazyLock<CNorm> =
        std::sync::LazyLock::new(|| SYS_BTN_COLOR.normalized().normalized());

    pub(super) const CAM_BTN_COLOR: Color = Color {
        r: 136,
        g: 172,
        b: 0,
    };

    pub static CAM_BTN_COLOR_NRM: std::sync::LazyLock<CNorm> =
        std::sync::LazyLock::new(|| CAM_BTN_COLOR.normalized().normalized());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockTime {
    /// One hour is 45 seconds. A night is 4 minutes 30 seconds, or 270 seconds -- 2700 deciseconds.
    /// This can be expressed in 12 bits as 0b101010001100.
    deciseconds: u16,
    pings_since_change: i32,
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

    pub const fn pings_since_change(&self) -> i32 {
        self.pings_since_change
    }

    // It takes 1 bit more than a char to describe the number of seconds in a night.
    pub const fn seconds(&self) -> u16 {
        self.deciseconds / DECISECS_PER_SEC as u16
    }

    // Not sure what we'd need this for, but just in case.
    pub const fn minutes(&self) -> u16 {
        self.seconds() / SECS_PER_MIN as u16 // realtime
    }

    // What hour of the night we are at
    pub const fn hour(&self) -> u16 {
        self.seconds() / SECS_PER_HOUR as u16 // gametime
    }

    // Converts hours to deciseconds, for finding how many deciseconds we are through the current hour.
    pub const fn whole_hour_deciseconds(&self) -> u16 {
        self.hour() * DECISECS_PER_HOUR
    }

    // Finds how many deciseconds into the current hour we are.
    pub const fn deciseconds_since_hour(&self) -> u16 {
        self.deciseconds() - self.whole_hour_deciseconds()
    }

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
            "{}:{}.{}",
            self.minutes(),
            self.seconds() % SECS_PER_MIN as u16,
            self.deciseconds() % DECISECS_PER_SEC as u16
        )
    }
}

// What gamestate we are in (what we can see on the screen)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
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
pub enum Camera {
    WestHall,
    EastHall,
    Closet,
    Kitchen,
    PirateCove,
    ShowtimeStage,
    PrizeCounter,
    PartsAndServices,
}

impl TryFrom<u8> for Camera {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::WestHall),
            1 => Ok(Self::EastHall),
            2 => Ok(Self::Closet),
            3 => Ok(Self::Kitchen),
            4 => Ok(Self::PirateCove),
            5 => Ok(Self::ShowtimeStage),
            6 => Ok(Self::PrizeCounter),
            7 => Ok(Self::PartsAndServices),
            _ => Err(()),
        }
    }
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
pub enum Vent {
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
pub enum Duct {
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

#[derive(Debug)]
struct OfficeData {
    /// How far left/right we are looking [-1,1]
    pub office_yaw: f64,
}

#[derive(Debug)]
struct CamData {
    /// Which camera we are looking at
    pub camera: Camera,
}

#[derive(Debug)]
struct VentData {
    /// Which vent snare is active
    pub vent_snare: Vent,
}

#[derive(Debug)]
struct DuctData {
    /// Which duct is currently closed
    pub closed_duct: Duct,
    pub audio_lure: POINT,
}

/// This is the type which actually stores the data we have about the gamestate
#[derive(Debug)]
struct GameData {
    flags: u8,
    pub time: ClockTime,
    pub next_ff_show: ClockTime,
}

impl GameData {
    const VENTILATION_NEEDS_RESET_FLAG: u8 = 1;
    const FLASHLIGHT_FLAG: u8 = Self::VENTILATION_NEEDS_RESET_FLAG << 1;
    /// in order from left to right
    const DOOR0_CLOSED_FLAG: u8 = Self::FLASHLIGHT_FLAG << 1;
    const DOOR1_CLOSED_FLAG: u8 = Self::DOOR0_CLOSED_FLAG << 1;
    const DOOR2_CLOSED_FLAG: u8 = Self::DOOR1_CLOSED_FLAG << 1;
    const DOOR3_CLOSED_FLAG: u8 = Self::DOOR2_CLOSED_FLAG << 1;

    const fn new() -> Self {
        Self {
            flags: 0,
            time: ClockTime::new(0),
            next_ff_show: ClockTime::new(0),
        }
    }

    pub const fn does_ventilation_need_reset(&self) -> bool {
        (self.flags & Self::VENTILATION_NEEDS_RESET_FLAG) != 0
    }
    pub const fn ventilation_has_been_reset(&mut self) {
        self.flags &= !Self::VENTILATION_NEEDS_RESET_FLAG;
    }
    pub const fn ventilation_needs_reset(&mut self) {
        self.flags |= Self::VENTILATION_NEEDS_RESET_FLAG;
    }
    pub const fn toggle_ventilation_reset(&mut self) {
        self.flags ^= Self::VENTILATION_NEEDS_RESET_FLAG;
    }

    pub const fn is_flashlight_on(&self) -> bool {
        (self.flags & Self::FLASHLIGHT_FLAG) != 0
    }
    pub const fn turn_flashlight_off(&mut self) {
        self.flags &= !Self::FLASHLIGHT_FLAG;
    }
    pub const fn turn_flashlight_on(&mut self) {
        self.flags |= Self::FLASHLIGHT_FLAG;
    }
    pub const fn toggle_flashlight(&mut self) {
        self.flags ^= Self::FLASHLIGHT_FLAG;
    }

    pub const fn is_door_closed(&self, door: i32) -> bool {
        (self.flags & Self::DOOR0_CLOSED_FLAG << door) != 0
    }
    pub const fn open_door(&mut self, door: i32) {
        self.flags &= !(Self::DOOR0_CLOSED_FLAG << door);
    }
    pub const fn close_door(&mut self, door: i32) {
        self.flags |= Self::DOOR0_CLOSED_FLAG << door;
    }
    pub const fn toggle_door(&mut self, door: i32) {
        self.flags ^= Self::DOOR0_CLOSED_FLAG << door;
    }
}

/// What state we are in (office, checking cameras, ducts, vents)
/// The metadata about the state (what part of the office, which camera)
/// Information about the current state that can tell us how to interpret information
#[derive(Debug)]
enum StateData {
    Office(OfficeData),
    Camera(CamData),
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

#[derive(Debug)]
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

    pub fn display_data(&self) {
        println!(
            r"{RESET_CURSOR}Time: {}

Ventilation: {}
  Left door: {}
 Front vent: {}
 Right door: {}
 Right vent: {}
 Flashlight: {}
Next Funtime Foxy show: {}
",
            self.game.time,
            if self.game.does_ventilation_need_reset() {
                "WARNING"
            } else {
                "good   "
            },
            if self.game.is_door_closed(0) {
                "closed"
            } else {
                "open  "
            },
            if self.game.is_door_closed(1) {
                "closed"
            } else {
                "open  "
            },
            if self.game.is_door_closed(2) {
                "closed"
            } else {
                "open  "
            },
            if self.game.is_door_closed(3) {
                "closed"
            } else {
                "open  "
            },
            if self.game.is_flashlight_on() {
                "on "
            } else {
                "off"
            },
            self.game.next_ff_show
        );

        print!("<");
        for s in [State::Camera, State::Vent, State::Duct, State::Office] {
            let [open, close] = match (&s, &self.state) {
                (State::Camera, StateData::Camera(_))
                | (State::Vent, StateData::Vent(_))
                | (State::Duct, StateData::Duct(_))
                | (State::Office, StateData::Office(_)) => *b"[]",
                _ => *b"  ",
            }
            .map(char::from);
            print!("{open}{s}{close}");
        }
        println!(">");

        match &self.state {
            StateData::Camera(cd) => {
                println!(
                    "Looking at: CAM 0{} | {:<18}",
                    (cd.camera as i32 + 1),
                    cd.camera
                );
            }

            StateData::Office(od) => {
                println!(
                    "Yaw: {}\nNightmare Balloon Boy: {}",
                    od.office_yaw,
                    if is_nmbb_standing() {
                        "standing"
                    } else {
                        "sitting "
                    }
                );
            }

            _ => println!("TODO"),
        }

        println!();
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

static SCREEN_WIDTH: OnceLock<i32> = OnceLock::new();
static SCREEN_HEIGHT: OnceLock<i32> = OnceLock::new();

#[derive(Debug)]
struct ScreenData {
    data: Vec<u8>,
}

impl ScreenData {
    pub const fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn update_screencap(&mut self, winh: &mut WindowsHandles) {
        use windows::Win32::Graphics::Gdi::{BitBlt, DIB_RGB_COLORS, GetDIBits, SRCCOPY};
        unsafe {
            BitBlt(
                winh.internal_hdc,
                0,
                0,
                *SCREEN_WIDTH.get().unwrap(),
                *SCREEN_HEIGHT.get().unwrap(),
                Some(winh.desktop_hdc),
                0,
                0,
                SRCCOPY,
            )
            .unwrap();

            GetDIBits(
                winh.desktop_hdc,
                winh.bitmap,
                0,
                *SCREEN_HEIGHT.get().unwrap() as u32,
                Some(self.data.as_mut_ptr().cast()),
                &mut win::bitmap_info(*SCREEN_WIDTH.get().unwrap(), *SCREEN_HEIGHT.get().unwrap()),
                DIB_RGB_COLORS,
            );
        }
    }

    pub fn pixel_color_at(&self, pos: POINT) -> Color {
        let index: usize =
            CHANNELS_PER_COLOR * ((pos.y * SCREEN_WIDTH.get().unwrap()) + pos.x) as usize;

        Color {
            r: self.data[index + 2],
            g: self.data[index + 1],
            b: self.data[index],
        }
    }
}

static SCREEN_DATA: std::sync::RwLock<ScreenData> =
    std::sync::RwLock::new(ScreenData::new(Vec::new()));

/// All the information we have about the state of the game
static GAME_STATE: RwLock<GameState> = RwLock::new(GameState::new());

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
    pnt::ofc::MASK_POS,
    pnt::cam::RESET_VENT_BTN_POS,
    pnt::cam::CAM_01_POS,
    pnt::cam::CAM_02_POS,
    pnt::cam::CAM_03_POS,
    pnt::cam::CAM_04_POS,
    pnt::cam::CAM_05_POS,
    pnt::cam::CAM_06_POS,
    pnt::cam::CAM_07_POS,
    pnt::cam::CAM_08_POS,
    POINT {
        x: pnt::cam::SYS_BTN_X,
        y: pnt::cam::CAM_SYS_BTN_Y,
    },
    POINT {
        x: pnt::cam::SYS_BTN_X,
        y: pnt::cam::VENT_SYS_BTN_Y,
    },
    POINT {
        x: pnt::cam::SYS_BTN_X,
        y: pnt::cam::DUCT_SYS_BTN_Y,
    },
    pnt::vnt::SNARE_L_POS,
    pnt::vnt::SNARE_T_POS,
    pnt::vnt::SNARE_R_POS,
    pnt::dct::DUCT_L_BTN_POS,
    pnt::dct::DUCT_R_BTN_POS,
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

fn is_nmbb_standing() -> bool {
    const PANTS_COLOR: Color = Color {
        r: 0,
        g: 28,
        b: 120,
    };
    const SAMPLE_POS: POINT = POINT { x: 1024, y: 774 };
    const THRESHOLD: f64 = 0.98;
    PANTS_COLOR.similarity(SCREEN_DATA.read().unwrap().pixel_color_at(SAMPLE_POS)) > THRESHOLD
}

/// Input should be top-left corner of the number followed by the size
fn read_number(x: i32, y: i32) -> u8 {
    const SAMPLE_OFFSETS: [POINT; 9] = [
        POINT { x: 5, y: 0 },
        POINT { x: 5, y: 8 },
        POINT { x: 0, y: 8 },
        POINT { x: 10, y: 8 },
        POINT { x: 0, y: 12 },
        POINT { x: 5, y: 12 },
        POINT { x: 10, y: 12 },
        POINT { x: 0, y: 7 },
        POINT { x: 10, y: 7 },
    ];
    const THRESHOLD: u8 = 100; // Minimum brightness value of the pixel

    let mut guess_bitflags: i32 = 0;
    let screen = SCREEN_DATA.read().unwrap();
    for (sample, offset) in SAMPLE_OFFSETS.iter().enumerate() {
        let sample_pos = POINT {
            x: x + offset.x,
            y: y + offset.y,
        };
        if screen.pixel_color_at(sample_pos).gray() > THRESHOLD {
            guess_bitflags |= 1 << sample;
        }
    }

    match guess_bitflags {
        0b110101101 => 0,
        0b000100011 => 1,
        0b001110011 => 2,
        0b000100001 => 3,
        0b010001110 => 4,
        0b100101001 => 5,
        0b010101101 => 6,
        0b000000011 => 7,
        0b000101101 => 8,
        0b100100001 => 9,
        _ => 0, // 0 on Error
    }
}

/// Run this about once every frame
fn read_game_clock() {
    let deciseconds = read_number(pnt::CLK_DECISEC_X, pnt::CLK_POS.y) as u16;
    let seconds = read_number(pnt::CLK_SEC_X, pnt::CLK_POS.y) as u16;
    let tens_of_seconds = read_number(pnt::CLK_10SEC_X, pnt::CLK_POS.y) as u16;
    let minute = read_number(pnt::CLK_POS.x, pnt::CLK_POS.y) as u16;

    let time = deciseconds
        + DECISECS_PER_SEC as u16 * (seconds + 10 * tens_of_seconds + SECS_PER_MIN as u16 * minute);

    GAME_STATE.write().unwrap().game.time.update_time(time);
}

fn does_ventilation_need_reset() -> bool {
    SCREEN_DATA
        .read()
        .unwrap()
        .pixel_color_at(match &GAME_STATE.read().unwrap().state {
            StateData::Office(_) => pnt::ofc::VENT_WARNING_POS,
            _ => pnt::cam::VENT_WARNING_POS,
        })
        .red_dev()
        > 35
}

fn generate_sample_points(start: POINT, scale: i32) -> [POINT; 5] {
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

/// - `center`: Point around which to generate the sample points
/// - `compare`: Normalized color against which to compare the color at the sample points
/// - `threshold`: 0..1 double value for the minimum similarity required to consider a sample point a "match"
///
/// returns: Total number of sample points which exceeded the threshold
fn test_samples(center: POINT, compare: CNorm, threshold: f64) -> i32 {
    let mut match_count = 0;
    let screen = SCREEN_DATA.read().unwrap();
    for sample_point in generate_sample_points(center, 4) {
        let sample = screen.pixel_color_at(sample_point).normalized();
        if sample.normalized().dot(compare) > threshold {
            match_count += 1;
        }
    }
    match_count
}

fn test_samples_gray(center: POINT, compare: u8, max_difference: u8) -> i32 {
    let mut match_count: i32 = 0;
    let screen = SCREEN_DATA.read().unwrap();
    for sample_point in generate_sample_points(center, 4) {
        let sample = screen.pixel_color_at(sample_point).gray();
        if sample.abs_diff(compare) > max_difference {
            match_count += 1;
        }
    }
    match_count
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

/// For finding the yaw of the office
fn locate_office_lamp() {
    const Y: i32 = 66;
    const THRESHOLD: u8 = 200;
    const START: i32 = 723;
    const WIDTH: i32 = 585;
    let screen = SCREEN_DATA.read().unwrap();
    for x in START..START + WIDTH {
        if screen.pixel_color_at(POINT { x, y: Y }).gray() > THRESHOLD {
            // 100% of the samples must be 80% matching. Flickering be damned.
            if test_samples_gray(POINT { x, y: Y }, 255, 20) == 5 {
                match &mut GAME_STATE.write().unwrap().state {
                    StateData::Office(od) => {
                        od.office_yaw = (x as f64 - START as f64) / WIDTH as f64;
                    }
                    _ => panic!(),
                }
                break;
            }
        }
    }
}

fn update_state() {
    const THRESHOLD: f64 = 0.99;
    let mut new_state = State::Office;
    // List of how many samples returned as matches for each of the buttons being tested
    let states_to_test = [State::Camera, State::Vent, State::Duct].map(|sys_btn| {
        test_samples(
            Button::from(sys_btn).pos(),
            *clr::SYS_BTN_COLOR_NRM,
            THRESHOLD,
        )
    });
    let index_of_max = max_in_array(states_to_test).unwrap();
    // We must have over 50% of the samples returning as matches
    if states_to_test[index_of_max] == 5 {
        new_state = match index_of_max {
            0 => State::Camera,
            1 => State::Vent,
            2 => State::Duct,
            3 => State::Office,
            _ => todo!(),
        }
    }
    // Update the global state
    GAME_STATE.write().unwrap().state = match new_state {
        State::Office => StateData::Office(OfficeData { office_yaw: 0.0 }),

        State::Camera => {
            let cams_to_test: [i32; 8] = std::array::from_fn(|camera| {
                test_samples(
                    Button::from(Camera::try_from(camera as u8).unwrap()).pos(),
                    *clr::CAM_BTN_COLOR_NRM,
                    THRESHOLD,
                )
            });
            // If we've confirmed the state then there's no doubt we can identify the camera
            StateData::Camera(CamData {
                camera: Camera::try_from(max_in_array(cams_to_test).unwrap() as u8).unwrap(),
            })
        }

        State::Vent => StateData::Vent(VentData {
            vent_snare: Vent::default(),
        }),

        State::Duct => StateData::Duct(DuctData {
            closed_duct: Duct::West,
            audio_lure: POINT::default(),
        }),
    }
}

/// Assumes we are already in the office
fn office_look_left() {
    assert!(
        matches!(GAME_STATE.read().unwrap().state, StateData::Office(_)),
        "cannot look left/right in cameras"
    );
    simulate_mouse_goto(POINT { x: 8, y: 540 });
    sleep(Duration::from_millis(5 * MS_PER_DECISEC as u64));
}

/// Assumes we are already in the office
fn office_look_right() {
    assert!(
        matches!(GAME_STATE.read().unwrap().state, StateData::Office(_)),
        "cannot look left/right in cameras"
    );
    simulate_mouse_goto(POINT { x: 1910, y: 540 });
    sleep(Duration::from_millis(5 * MS_PER_DECISEC as u64));
}

///////////////////////////////////////////////////////////////////////////
// This is where basic outputs are combined to make more complex actions //
///////////////////////////////////////////////////////////////////////////

/// Updates all known game information
fn refresh_game_data() {
    update_state();
    read_game_clock();
    if does_ventilation_need_reset() {
        GAME_STATE.write().unwrap().game.ventilation_needs_reset();
    }
    //LocateOfficeLamp(); // Needs work
}

fn toggle_monitor() {
    simulate_keypress(VirtualKey::CameraToggle);
    sleep(Duration::from_millis(CAM_RESP_MS as u64));
    update_state();
}

fn open_monitor_if_closed() {
    if matches!(GAME_STATE.read().unwrap().state, StateData::Office(_)) {
        toggle_monitor();
    }
}

fn close_monitor_if_open() {
    if matches!(GAME_STATE.read().unwrap().state, StateData::Office(_)) {
        toggle_monitor();
    }
}

fn ensure_system(system: State) {
    open_monitor_if_closed();
    if GAME_STATE.read().unwrap().state.state() != system {
        simulate_mouse_click_at(Button::from(system).pos());
    }
}

fn open_camera_if_closed() {
    ensure_system(State::Camera);
    sleep(Duration::from_millis(1)); // In case the next step is another button press elsewhere
}

// `cam` only used if `state == State::Camera`
fn enter_game_state(state: State, cam: Camera) {
    if GAME_STATE.read().unwrap().state.state() != state {
        if matches!(state, State::Office) {
            close_monitor_if_open();
        } else {
            open_monitor_if_closed();
        }
        match state {
            State::Office => {}

            State::Camera => {
                if let StateData::Camera(cd) = &mut GAME_STATE.write().unwrap().state
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
}

/// Playbook of actions
mod action {
    use super::*;

    pub fn handle_funtime_foxy() {
        open_camera_if_closed();
        simulate_mouse_click_at(Button::Cam06.pos());
    }

    pub fn reset_vents() {
        open_monitor_if_closed(); // We don't need to care which system, only that the monitor is up.
        simulate_mouse_click_at(Button::ResetVent.pos());
        GAME_STATE
            .write()
            .unwrap()
            .game
            .ventilation_has_been_reset();
        sleep(Duration::from_millis(10));
    }

    pub fn handle_nmbb(winh: &mut WindowsHandles) {
        sleep(Duration::from_millis(17)); // Wait a little bit to make sure we have time for the screen to change
        SCREEN_DATA.write().unwrap().update_screencap(winh);
        if is_nmbb_standing() {
            // Double check--NMBB will kill us if we flash him wrongfully
            // If he is in fact still up, flash the light on him to put him back down
            simulate_keypress(VirtualKey::Flashlight);
        }
    }
}

fn act_on_game_data(winh: &mut WindowsHandles) {
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
    if matches!(GAME_STATE.read().unwrap().state, StateData::Office(_)) && is_nmbb_standing() {
        action::handle_nmbb(winh);
    }

    if GAME_STATE
        .read()
        .unwrap()
        .game
        .does_ventilation_need_reset()
    {
        action::reset_vents();
    }

    // We have <= 1 seconds before the next hour
    if (DECISECS_PER_HOUR
        - GAME_STATE
            .read()
            .unwrap()
            .game
            .time
            .deciseconds_since_hour())
        <= (DECISECS_PER_SEC as u16 + (CAM_RESP_MS / MS_PER_DECISEC as u16))
    {
        action::handle_funtime_foxy();
        sleep(Duration::from_millis(10));
    }

    // Lowest priority actions should go down here //
}

fn main() {
    let winh = Arc::new(RwLock::new(WindowsHandles::new()));

    SCREEN_DATA.write().unwrap().data.resize(
        CHANNELS_PER_COLOR
            * *SCREEN_WIDTH.get().unwrap() as usize
            * *SCREEN_HEIGHT.get().unwrap() as usize,
        0,
    );

    SCREEN_DATA
        .write()
        .unwrap()
        .update_screencap(&mut winh.write().unwrap()); // first time screen update

    let threads_should_loop: AtomicBool = AtomicBool::new(true);
    print!("{CLEAR_CONSOLE}");
    std::thread::scope(|s| {
        // Spawn a thread for reading the screen pixels
        s.spawn(|| {
            while threads_should_loop.load(Relaxed) {
                SCREEN_DATA
                    .write()
                    .unwrap()
                    .update_screencap(&mut winh.write().unwrap()); // Update our internal copy of what the gamescreen looks like so we can sample its pixels
                sleep(Duration::from_millis(2));
            }
        });

        // Spawn a thread for acting on that data
        s.spawn(|| {
            while threads_should_loop.load(Relaxed) {
                refresh_game_data(); // Using the screencap we just generated, update the game data statuses for decision making
                GAME_STATE.read().unwrap().display_data(); // Output the data for the user to view
                act_on_game_data(&mut winh.write().unwrap()); // Based upon the game data, perform all actions necessary to return the game to a neutral state
                sleep(Duration::from_millis(4));
            }
        });

        // !! SAFETY !!
        // Make sure that user control override doesn't disable the user from closing the program
        while threads_should_loop.load(Relaxed) {
            sleep(Duration::from_millis(2)); // Give the user time to provide input

            if is_key_down(VirtualKey::Esc) {
                // mask to ignore the "toggled" bit
                println!("{CLEAR_CONSOLE}\nUser has chosen to reclaim control. Task ended.");
                threads_should_loop.store(false, Relaxed); // This tells the worker threads to stop
                break;
            }
        }

        println!("\nWaiting on worker threads...");
        // Wait for threads to safely finish their respective functions before destructing them
    });
    println!("\nWorker threads joined.");
}
