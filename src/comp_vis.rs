//! Computer Vision

use crate::{DECISECS_PER_SEC, SECS_PER_MIN, game_state::GameData, win::POINT};
use color::{CHANNELS_PER_COLOR, CNorm, ColorRGB};
use std::sync::nonpoison::{Condvar, Mutex};

pub mod color;

#[derive(Debug, Clone)]
pub struct ScreenData {
    data: Box<[u8]>,
    width: i32,
}

impl ScreenData {
    pub const fn new(data: Box<[u8]>, width: i32) -> Self {
        Self { data, width }
    }

    pub fn pixel_color_at(&self, pos: POINT) -> ColorRGB {
        let index: usize = CHANNELS_PER_COLOR * ((pos.y * self.width) + pos.x) as usize;

        ColorRGB {
            r: self.data[index + 2],
            g: self.data[index + 1],
            b: self.data[index],
        }
    }

    pub const fn size(&self) -> (u32, u32) {
        (self.width as u32, self.len() as u32 / self.width as u32)
    }

    pub const fn len(&self) -> usize {
        self.data.len()
    }

    pub const fn data_mut(&mut self) -> &mut Box<[u8]> {
        &mut self.data
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadNumberError {
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

pub const READ_NUMBER_SAMPLE_OFFSETS: [POINT; 8] = [
    POINT { x: 5, y: 0 },  // top middle
    POINT { x: 0, y: 7 },  // upper-middle left
    POINT { x: 10, y: 7 }, // upper-middle right
    POINT { x: 5, y: 8 },  // lower-middle middle
    POINT { x: 0, y: 8 },  // lower-middle left
    POINT { x: 10, y: 8 }, // lower-middle right
    POINT { x: 0, y: 12 }, // bottom left
    POINT { x: 5, y: 12 }, // bottom middle
];

pub const READ_NUMBER_SAMPLE_FLAGS: [u8; 10] = [
    0b10110111, 0b10001001, 0b11001001, 0b10000001, 0b00111010, 0b10100101, 0b10110011, 0b00001001,
    0b10110001, 0b10000101,
];

impl ScreenData {
    /// Input should be top-left corner of the number followed by the size
    pub fn read_number(&self, x: i32, y: i32) -> Result<u8, ReadNumberError> {
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

#[derive(Debug)]
pub struct ScreenDataPair {
    pub buffer: Mutex<ScreenData>,
    pub counter: Mutex<usize>,
    pub updated: Condvar,
}

impl ScreenDataPair {
    pub fn mark_updated(&self) {
        let mut counter = self.counter.lock();
        *counter = counter.wrapping_add(1);
        self.updated.notify_all();
    }

    pub fn wait_for_update(&self) {
        let mut counter = self.counter.lock();
        let current_counter = *counter;
        self.updated
            .wait_while(&mut counter, move |&mut pending| pending != current_counter);
    }
}

pub trait FromPixels: Sized {
    type Err;

    fn from_pixels(sd: &ScreenData) -> Result<Self, Self::Err>;
}

pub trait ParsePx<T: FromPixels> {
    fn parse_px(&self) -> Result<T, T::Err>;
}

impl<T: FromPixels> ParsePx<T> for ScreenData {
    #[inline]
    fn parse_px(&self) -> Result<T, T::Err> {
        T::from_pixels(self)
    }
}
