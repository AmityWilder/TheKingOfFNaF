use crate::{
    CAM_RESP_MS, DECISECS_PER_HOUR, DECISECS_PER_SEC, MS_PER_DECISEC, clr,
    comp_vis::{
        READ_NUMBER_SAMPLE_FLAGS, READ_NUMBER_SAMPLE_OFFSETS, ReadNumberError, ScreenData,
        ScreenDataPair, color::ColorRGB,
    },
    data::{
        draw_graph_slice,
        history::{GameStateDelta, GameStateHistory},
    },
};
use clk::ClockTime;
use raylib::prelude::*;
use std::{sync::Arc, thread::sleep, time::Duration};
use vidivici::{IVec2, VirtualKey};

pub mod clk;

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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OfficeData {
    /// How far left/right we are looking [-1,1]
    pub office_yaw: f64,
    pub is_nmbb_standing: bool,
}

impl OfficeData {
    pub const MASK_POS: IVec2 = IVec2 { x: 682, y: 1006 };
    /// The office version of this constant
    pub const VENT_WARNING_POS: IVec2 = IVec2 { x: 1580, y: 1040 };

    pub const _FOXY: IVec2 = IVec2 { x: 801, y: 710 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CameraData {
    /// Which camera we are looking at
    pub camera: Camera,
}

impl CameraData {
    /// Location for testing vent warning in the cameras
    pub const VENT_WARNING_POS: IVec2 = IVec2 { x: 1563, y: 892 };
    /// Where the reset vent button is for clicking
    pub const RESET_VENT_BTN_POS: IVec2 = IVec2 { x: 1700, y: 915 };

    /// WestHall
    pub const CAM_01_POS: IVec2 = IVec2 { x: 1133, y: 903 };
    /// EastHall
    pub const CAM_02_POS: IVec2 = IVec2 { x: 1382, y: 903 };
    /// Closet
    pub const CAM_03_POS: IVec2 = IVec2 { x: 1067, y: 825 };
    /// Kitchen
    pub const CAM_04_POS: IVec2 = IVec2 { x: 1491, y: 765 };
    /// PirateCove
    pub const CAM_05_POS: IVec2 = IVec2 { x: 1122, y: 670 };
    /// ShowtimeStage
    pub const CAM_06_POS: IVec2 = IVec2 { x: 1422, y: 590 };
    /// PrizeCounter
    pub const CAM_07_POS: IVec2 = IVec2 { x: 1278, y: 503 };
    /// PartsAndServices
    pub const CAM_08_POS: IVec2 = IVec2 { x: 988, y: 495 };

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
pub struct VentData {
    /// Which vent snare is active
    pub vent_snare: Vent,
}

impl VentData {
    /// Left snare
    pub const SNARE_L_POS: IVec2 = IVec2 { x: 548, y: 645 };
    /// Top snare
    pub const SNARE_T_POS: IVec2 = IVec2 { x: 650, y: 536 };
    /// Right snare
    pub const SNARE_R_POS: IVec2 = IVec2 { x: 747, y: 645 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DuctData {
    /// Which duct is currently closed
    pub closed_duct: Duct,
    pub audio_lure: IVec2,
}

impl DuctData {
    /// Check left duct
    pub const _DUCT_L_POS: IVec2 = IVec2 { x: 500, y: 791 };
    /// Check right duct
    pub const _DUCT_R_POS: IVec2 = IVec2 { x: 777, y: 791 };
    /// Left duct button
    pub const DUCT_L_BTN_POS: IVec2 = IVec2 { x: 331, y: 844 };
    /// Right duct button
    pub const DUCT_R_BTN_POS: IVec2 = IVec2 { x: 1016, y: 844 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ClockTimeResult {
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
pub struct GameData {
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
    pub fn mark_ventilation_has_been_reset<const N: usize>(
        &mut self,
        history: &mut GameStateHistory<N>,
    ) {
        self.flags &= !Self::VENTILATION_NEEDS_RESET_FLAG;
        history.push(GameStateDelta::VentilationResetNeeded(false));
    }
    pub fn mark_ventilation_needs_reset<const N: usize>(
        &mut self,
        history: &mut GameStateHistory<N>,
    ) {
        self.flags |= Self::VENTILATION_NEEDS_RESET_FLAG;
        history.push(GameStateDelta::VentilationResetNeeded(true));
    }

    pub const fn is_flashlight_on(&self) -> bool {
        (self.flags & Self::FLASHLIGHT_FLAG) != 0
    }
    pub fn mark_flashlight_off<const N: usize>(&mut self, history: &mut GameStateHistory<N>) {
        self.flags &= !Self::FLASHLIGHT_FLAG;
        history.push(GameStateDelta::FlashlightOn(false));
    }
    pub fn mark_flashlight_on<const N: usize>(&mut self, history: &mut GameStateHistory<N>) {
        self.flags |= Self::FLASHLIGHT_FLAG;
        history.push(GameStateDelta::FlashlightOn(true));
    }

    pub const fn is_door_closed(&self, door: u8) -> bool {
        (self.flags & Self::DOOR0_CLOSED_FLAG << door) != 0
    }
    pub fn mark_door_open<const N: usize>(&mut self, door: u8, history: &mut GameStateHistory<N>) {
        self.flags &= !(Self::DOOR0_CLOSED_FLAG << door);
        history.push(GameStateDelta::DoorClosed(door, false));
    }
    pub fn mark_door_closed<const N: usize>(
        &mut self,
        door: u8,
        history: &mut GameStateHistory<N>,
    ) {
        self.flags |= Self::DOOR0_CLOSED_FLAG << door;
        history.push(GameStateDelta::DoorClosed(door, true));
    }
}

/// Important positions on the screen
impl GameData {
    /// Clock position
    pub const CLK_POS: IVec2 = IVec2 { x: 1807, y: 85 };
    pub const CLK_10SEC_X: i32 = 1832;
    pub const CLK_SEC_X: i32 = 1849;
    pub const CLK_DECISEC_X: i32 = 1873;

    pub const _TEMPERATURE_POS: IVec2 = IVec2 { x: 1818, y: 1012 };

    pub const _COINS: IVec2 = IVec2 { x: 155, y: 75 };

    pub const _PWR_POS: IVec2 = IVec2 { x: 71, y: 910 };
    pub const _PWR_USG_POS: IVec2 = IVec2 { x: 38, y: 969 };

    pub const _NOISE_POS: IVec2 = IVec2 { x: 38, y: 1020 };

    /// Not really needed since the S key exists...
    pub const _OPEN_CAM_POS: IVec2 = IVec2 { x: 1280, y: 1006 };
}

/// What state we are in (office, checking cameras, ducts, vents)
/// The metadata about the state (what part of the office, which camera)
/// Information about the current state that can tell us how to interpret information
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateData {
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

#[derive(Debug)]
pub struct GameState<const BLOCK_CAP: usize> {
    pub state: StateData,
    pub game: GameData,
    hist: GameStateHistory<BLOCK_CAP>,
}

impl<const BLOCK_CAP: usize> GameState<BLOCK_CAP> {
    pub const fn new() -> Self {
        Self {
            state: StateData::Office(OfficeData {
                office_yaw: 0.0,
                is_nmbb_standing: false,
            }),
            game: GameData::new(),
            hist: GameStateHistory::new(),
        }
    }

    pub const fn state(&self) -> State {
        self.state.state()
    }

    #[allow(clippy::too_many_arguments, reason = "dont care")]
    pub fn draw_data(
        &self,
        d: &mut RaylibDrawHandle,
        _thread: &RaylibThread,
        ucn_numbers: &Texture2D,
        font_size: i32,
        color: Color,
        err_color: Color,
    ) {
        use raylib::prelude::*;

        let line_space = font_size + 4;
        let mut y = 0;

        let time_str = &format!("Time: {}", *self.game.time);
        d.draw_text(time_str, 0, y, font_size, color);
        let mut y1 = y;
        let mut x = d.measure_text(time_str, font_size) + font_size;
        if self.game.time.error.iter().any(|e| e.is_some()) {
            const ERR_STR: &str = "error: unrecognized digit(s); time has not been updated";
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
                y += line_space;
            }

            StateData::Office(od) => {
                d.draw_text(&format!("Yaw: {}", od.office_yaw,), 0, y, font_size, color);
                y += line_space;
                d.draw_text(
                    &format!(
                        "Nightmare Balloon Boy: {}",
                        if od.is_nmbb_standing {
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
                y += line_space;
            }

            _ => {
                d.draw_text("TODO", 0, y, font_size, color);

                y += line_space;
            }
        }

        let num_records = self.hist.len();
        d.draw_text(
            &format!(
                "Recorded: {} ({} blocks {} states)",
                num_records,
                num_records / BLOCK_CAP,
                num_records % BLOCK_CAP
            ),
            0,
            y,
            font_size,
            color,
        );
        y += line_space;

        if let Some(first) = self.hist.get(0) {
            let start_time = first.timestamp;

            // monotonic clock
            draw_graph_slice(
                d,
                self.hist.iter().filter_map(|record| match record.change {
                    GameStateDelta::Time(read_time) => Some((record.timestamp, read_time as i32)),
                    _ => None,
                }),
                start_time,
                Duration::from_secs(5),
                0..=300,
                y..=y + 200,
                Color::LIGHTBLUE,
            );

            // state
            draw_graph_slice(
                d,
                self.hist.iter().filter_map(|record| match record.change {
                    GameStateDelta::State(value) => Some((record.timestamp, value as i32)),
                    _ => None,
                }),
                start_time,
                Duration::from_secs(5),
                0..=300,
                y..=y + 200,
                Color::BLUEVIOLET,
            );

            // nmbb status
            draw_graph_slice(
                d,
                self.hist.iter().filter_map(|record| match record.change {
                    GameStateDelta::NMBBStanding(value) => Some((record.timestamp, value as i32)),
                    _ => None,
                }),
                start_time,
                Duration::from_secs(5),
                0..=300,
                y..=y + 200,
                Color::TOMATO,
            );

            // state
            draw_graph_slice(
                d,
                self.hist.iter().filter_map(|record| match record.change {
                    GameStateDelta::FlashlightOn(is_on) => Some((record.timestamp, is_on as i32)),
                    _ => None,
                }),
                start_time,
                Duration::from_secs(5),
                0..=300,
                y..=y + 200,
                Color::YELLOW,
            );
        }
    }
}

impl<const BLOCK_CAP: usize> GameState<BLOCK_CAP> {
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

/// These enable us to put the buttons in an array and choose from them instead of just using the literal names
/// If you're trying to get the position of just the one thing and don't need to do any sort of "switch" thing, please don't use this. It adds additional steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
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
pub const BTN_POSITIONS: [IVec2; 18] = [
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
    IVec2 {
        x: CameraData::SYS_BTN_X,
        y: CameraData::CAM_SYS_BTN_Y,
    },
    IVec2 {
        x: CameraData::SYS_BTN_X,
        y: CameraData::VENT_SYS_BTN_Y,
    },
    IVec2 {
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
    pub const fn pos(self) -> IVec2 {
        BTN_POSITIONS[self as usize]
    }
}

////////////////////////////////////////////////////////////////////////////////////
// This is where the input we've taken from the game gets turned into useful data //
////////////////////////////////////////////////////////////////////////////////////

impl OfficeData {
    /// For finding the yaw of the office
    pub fn locate_office_lamp<const BLOCK_CAP: usize>(
        &mut self,
        screen_data: &ScreenData,
        history: &mut GameStateHistory<BLOCK_CAP>,
    ) {
        const Y: i32 = 66;
        const THRESHOLD: u8 = 200;
        const START: i32 = 723;
        const WIDTH: i32 = 585;
        for x in START..START + WIDTH {
            if screen_data.pixel_color_at(IVec2 { x, y: Y }).gray() > THRESHOLD {
                // 100% of the samples must be 80% matching. Flickering be damned.
                if screen_data.test_samples_gray(IVec2 { x, y: Y }, 255, 20) == 5 {
                    self.office_yaw = (x as f64 - START as f64) / WIDTH as f64;
                    history.push(GameStateDelta::OfficeYaw(self.office_yaw));
                    break;
                }
            }
        }
    }

    /// Assumes we are already in the office
    pub fn look_left<const BLOCK_CAP: usize>(&mut self, history: &mut GameStateHistory<BLOCK_CAP>) {
        history.sim_mouse_goto(IVec2 { x: 8, y: 540 });
        sleep(Duration::from_millis(5 * MS_PER_DECISEC as u64));
    }

    /// Assumes we are already in the office
    fn look_right<const BLOCK_CAP: usize>(&mut self, history: &mut GameStateHistory<BLOCK_CAP>) {
        history.sim_mouse_goto(IVec2 { x: 1910, y: 540 });
        sleep(Duration::from_millis(5 * MS_PER_DECISEC as u64));
    }

    pub fn handle_nmbb<const BLOCK_CAP: usize>(
        &mut self,
        screen_data: &Arc<ScreenDataPair>,
        history: &mut GameStateHistory<BLOCK_CAP>,
    ) {
        // Double check--NMBB will kill us if we flash him wrongfully
        // If he is in fact still up, flash the light on him to put him back down
        self.is_nmbb_standing = is_nmbb_standing(&screen_data.buffer.lock());
        history.push(GameStateDelta::IsNmbbStanding(self.is_nmbb_standing));
        if self.is_nmbb_standing {
            history.sim_key_tap(VirtualKey::Flashlight);
            screen_data.wait_for_update();
            self.is_nmbb_standing = is_nmbb_standing(&screen_data.buffer.lock());
            history.push(GameStateDelta::IsNmbbStanding(self.is_nmbb_standing));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStateError {
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

impl<const BLOCK_CAP: usize> GameState<BLOCK_CAP> {
    pub fn update_state(&mut self) -> Result<(), UpdateStateError> {
        const THRESHOLD: f64 = 0.99;
        let mut new_state = State::Office;
        let screen_data = self.screen_data.buffer.lock();
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

        let state_changing = self.state() != new_state;
        if state_changing {
            self.hist.push(GameStateDelta::State(new_state));

            // Update the global state
            self.state = match new_state {
                State::Office => {
                    let od = OfficeData {
                        office_yaw: 0.0,
                        is_nmbb_standing: false,
                    };
                    StateData::Office(od)
                }

                State::Camera => StateData::Camera(CameraData {
                    camera: Camera::WestHall,
                }),

                State::Vent => StateData::Vent(VentData {
                    vent_snare: Vent::default(),
                }),

                State::Duct => StateData::Duct(DuctData {
                    closed_duct: Duct::West,
                    audio_lure: IVec2::default(),
                }),
            };
        }

        // Update camera
        if let StateData::Camera(cd) = &mut self.state {
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
                if state_changing || cd.camera != camera {
                    self.hist.push(GameStateDelta::Camera(camera));
                    *cd = CameraData { camera };
                }
            } else {
                return Err(UpdateStateError::NoMatchingCameraInCameraState);
            }
        }

        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////
    // This is where basic outputs are combined to make more complex actions //
    ///////////////////////////////////////////////////////////////////////////

    /// Updates all known game information
    pub fn refresh_game_data(&mut self) -> Result<(), UpdateStateError> {
        self.update_state()?;

        match self.screen_data.buffer.lock().read_game_clock() {
            Ok(time) => {
                self.game.time.update_time(time);
                self.game.time.error = [None; 4];
                self.hist.push(GameStateDelta::Time(time));
            }
            Err(e) => self.game.time.error = e,
        }

        if self.does_ventilation_need_reset(&self.screen_data.buffer.lock()) {
            self.game.mark_ventilation_needs_reset(&mut self.hist);
        }

        if let StateData::Office(od) = &mut self.state {
            let new_value = is_nmbb_standing(&self.screen_data.buffer.lock());
            if od.is_nmbb_standing != new_value {
                od.is_nmbb_standing = new_value;
                self.hist.push(GameStateDelta::NMBBStanding(new_value));
            }
        }

        //self.locate_office_lamp(); // Needs work

        Ok(())
    }

    pub fn toggle_monitor(&mut self) -> Result<(), UpdateStateError> {
        self.hist.sim_key_tap(VirtualKey::CameraToggle);
        sleep(Duration::from_millis(CAM_RESP_MS as u64));
        self.update_state()
    }

    pub fn open_monitor_if_closed(&mut self) -> Result<(), UpdateStateError> {
        if self.state() == State::Office {
            self.toggle_monitor()?;
        }
        Ok(())
    }

    // `cam` only used if `state == State::Camera`
    pub fn enter_game_state(
        &mut self,
        new_state: State,
        cam: Camera,
    ) -> Result<(), UpdateStateError> {
        let current_state = self.state();
        if current_state != new_state {
            if (current_state == State::Office) != (new_state == State::Office) {
                self.toggle_monitor()?;
            }
            match new_state {
                State::Office => {}

                State::Camera => {
                    if let StateData::Camera(cd) = &mut self.state
                        && cd.camera != cam
                    {
                        self.hist.sim_mouse_click_at(Button::from(cam).pos());
                    }
                }

                State::Duct => self.hist.sim_mouse_click_at(Button::DuctSystem.pos()),
                State::Vent => self.hist.sim_mouse_click_at(Button::VentSystem.pos()),
            }
            sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    //
    // Playbook of actions
    //

    pub fn handle_funtime_foxy(&mut self) -> Result<(), UpdateStateError> {
        self.open_monitor_if_closed()?;
        if self.state() != State::Camera {
            self.hist
                .sim_mouse_click_at(Button::from(State::Camera).pos());
        }
        sleep(Duration::from_millis(1));
        self.hist.sim_mouse_click_at(Button::Cam06.pos());
        Ok(())
    }

    pub fn reset_vents(&mut self) -> Result<(), UpdateStateError> {
        self.open_monitor_if_closed()?; // We don't need to care which system, only that the monitor is up.
        self.hist.sim_mouse_click_at(Button::ResetVent.pos());
        self.game.mark_ventilation_has_been_reset(&mut self.hist);
        sleep(Duration::from_millis(10));
        Ok(())
    }

    pub fn act_on_game_data(&mut self) -> Result<(), UpdateStateError> {
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

        if let StateData::Office(od) = &mut self.state
            && od.is_nmbb_standing
        {
            od.handle_nmbb(&self.screen_data, &mut self.hist);
        }

        if self.game.is_ventilation_reset_needed() {
            self.reset_vents()?;
        }

        // We have <= 1 seconds before the next hour
        if (DECISECS_PER_HOUR - self.game.time.deciseconds_since_hour())
            <= (DECISECS_PER_SEC as u16 + (CAM_RESP_MS / MS_PER_DECISEC as u16))
        {
            self.handle_funtime_foxy()?;
            sleep(Duration::from_millis(10));
        }

        // Lowest priority actions should go down here //

        Ok(())
    }
}

pub fn is_nmbb_standing(screen_data: &ScreenData) -> bool {
    const PANTS_COLOR: ColorRGB = ColorRGB {
        r: 0,
        g: 28,
        b: 120,
    };
    const SAMPLE_POS: IVec2 = IVec2 { x: 1024, y: 774 };
    const THRESHOLD: f64 = 0.98;
    PANTS_COLOR.similarity(screen_data.pixel_color_at(SAMPLE_POS)) > THRESHOLD
}
