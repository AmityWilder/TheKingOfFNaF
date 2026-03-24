//! windows.h is ew yucky, i dont want it touching the main program

#![warn(
    clippy::undocumented_unsafe_blocks,
    reason = "do not create soundness holes in communicating with the OS"
)] // TODO: change this to deny once windows.h safety has been cleared up

use std::{mem::size_of, num::NonZeroU32, thread::sleep, time::Duration};

mod wrapper;

pub use wrapper::Point;
use wrapper::*;

#[derive(Debug)]
pub struct WindowsHandles {
    /// get the desktop device context
    desktop_hdc: CommonHdc,
    /// create a device context to use ourselves
    internal_hdc: CompatibilityHdc,
    /// create a bitmap
    bitmap: HBitmap,

    /// Should always be compatible with `as i32`
    screen_width: NonZeroU32,
    /// Should always be compatible with `as i32`
    screen_height: NonZeroU32,
}

impl Drop for WindowsHandles {
    fn drop(&mut self) {
        unsafe {
            self.ucn_hdc.release_dc();
            self.bitmap.delete_object();
            _ = DeleteDC(self.internal_hdc); // Destroy our internal display handle
            _ = ReleaseDC(None, self.desktop_hdc); // Free the desktop handle
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowsHandlesError {
    SystemMetrics,
    SystemMetricsTryFromInt(std::num::TryFromIntError),
    Windows(windows::core::Error),
}

impl std::fmt::Display for WindowsHandlesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SystemMetrics => write!(f, "could not obtain screen size"),
            Self::SystemMetricsTryFromInt(_) => {
                write!(f, "screen cannot be negative")
            }
            Self::Windows(_) => write!(f, "windows error during initialization"),
        }
    }
}

impl std::error::Error for WindowsHandlesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SystemMetricsTryFromInt(e) => Some(e),
            Self::Windows(e) => Some(e),
            _ => None,
        }
    }
}

impl WindowsHandles {
    pub fn new() -> std::result::Result<Self, WindowsHandlesError> {
        let screen_width = NonZeroU32::new(
            // SAFETY: GetSystemMetrics has no safety requirements.
            unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) }
                .try_into()
                .map_err(WindowsHandlesError::SystemMetricsTryFromInt)?,
        )
        .ok_or(WindowsHandlesError::SystemMetrics)?;
        let screen_height = NonZeroU32::new(
            // SAFETY: GetSystemMetrics has no safety requirements.
            unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) }
                .try_into()
                .map_err(WindowsHandlesError::SystemMetricsTryFromInt)?,
        )
        .ok_or(WindowsHandlesError::SystemMetrics)?;

        let ucn_hdc = unsafe { GetWindowDC(Some(ucn_hwnd)) };
        let desktop_hdc = unsafe { GetDC(None) }; // get the desktop device context
        let internal_hdc = unsafe { CreateCompatibleDC(Some(desktop_hdc)) }; // create a device context to use ourselves

        let bitmap = unsafe {
            CreateCompatibleBitmap(
                desktop_hdc,
                screen_width.get() as i32,
                screen_height.get() as i32,
            )
        };

        unsafe {
            SelectObject(internal_hdc, HGDIOBJ(bitmap.0)); // Get a handle to our bitmap
            // why are we ignoring the return?
        }

        Ok(Self {
            ucn_hdc,
            ucn_hwnd,
            desktop_hdc,
            internal_hdc,
            bitmap,
            screen_width,
            screen_height,
        })
    }

    #[inline]
    pub const fn screen_width(&self) -> NonZeroU32 {
        self.screen_width
    }

    #[inline]
    pub const fn screen_height(&self) -> NonZeroU32 {
        self.screen_height
    }

    #[inline]
    pub const fn screen_width_u32(&self) -> u32 {
        self.screen_width.get()
    }

    #[inline]
    pub const fn screen_height_u32(&self) -> u32 {
        self.screen_height.get()
    }

    #[inline]
    pub const fn screen_width_i32(&self) -> i32 {
        self.screen_width.get() as i32
    }

    #[inline]
    pub const fn screen_height_i32(&self) -> i32 {
        self.screen_height.get() as i32
    }

    pub fn bitblt(&self, buffer: &mut [u8]) -> WindowsResult<()> {
        unsafe {
            BitBlt(
                self.internal_hdc,
                0,
                0,
                self.screen_width_i32(),
                self.screen_height_i32(),
                Some(self.desktop_hdc),
                0,
                0,
                SRCCOPY,
            )?;

            GetDIBits(
                self.desktop_hdc,
                self.bitmap,
                0,
                self.screen_height_u32(),
                Some(buffer.as_mut_ptr().cast()),
                &mut bitmap_info(self.screen_width_i32(), self.screen_height_i32()),
                DIB_RGB_COLORS,
            );
        }
        Ok(())
    }
}

pub const fn bitmap_info(width: i32, height: i32) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0, // 3 * ScreenX * ScreenY; (position, not size)
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    }
}

////////////////////////////////////////////////////
// This is where we send basic output to the game //
// e.g.                                           //
// - Press "d" key                                //
// - Move mouse to { 24, 36 }                     //
////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    /// Front vent
    VkW = 'W' as isize,
    /// Left door
    VkA = 'A' as isize,
    /// Camera toggle
    VkS = 'S' as isize,
    /// Right door
    VkD = 'D' as isize,
    /// Right vent
    VkF = 'F' as isize,
    /// Catch fish
    VkC = 'C' as isize,
    /// Close ad
    VkEnter = '\n' as isize,
    /// Desk fan
    VkSpace = ' ' as isize,
    Vk1 = '1' as isize,
    Vk2 = '2' as isize,
    Vk3 = '3' as isize,
    Vk4 = '4' as isize,
    Vk5 = '5' as isize,
    Vk6 = '6' as isize,
    VkX = 'X' as isize,
    /// Flashlight
    VkZ = 'Z' as isize,
    Esc = '\x1b' as isize,
}

#[allow(non_upper_case_globals)]
impl VirtualKey {
    pub const FrontVent: Self = Self::VkW;
    pub const LeftDoor: Self = Self::VkA;
    pub const CameraToggle: Self = Self::VkS;
    pub const RightDoor: Self = Self::VkD;
    pub const RightVent: Self = Self::VkF;
    pub const CatchFish: Self = Self::VkC;
    pub const CloseAd: Self = Self::VkEnter;
    pub const DeskFan: Self = Self::VkSpace;
    pub const Flashlight: Self = Self::VkZ;
}

pub const fn key_input(key: VirtualKey, key_up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(key as u16),
                wScan: 0,
                dwFlags: if key_up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranslateType {
    Relative = 0,
    Absolute = MOUSEEVENTF_ABSOLUTE.0 as isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum M1State {
    None = 0,
    Press = MOUSEEVENTF_LEFTDOWN.0 as isize,
    Release = MOUSEEVENTF_LEFTUP.0 as isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MouseMovement {
    pub x: i32,
    pub y: i32,
    pub translation: TranslateType,
}

pub const fn mouse_input(movement: Option<MouseMovement>, m1: M1State) -> INPUT {
    let (dx, dy, movement_flags) = match movement {
        Some(m) => (m.x, m.y, MOUSEEVENTF_MOVE.0 | m.translation as u32),
        None => (0, 0, 0),
    };
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSE_EVENT_FLAGS(movement_flags | m1 as u32),
                time: 0, // Pleaseeeee don't mess with this... it makes the monitor go funky...
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn simulate_key_down(key: VirtualKey) {
    let input: INPUT = key_input(key, false);
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
    sleep(Duration::from_millis(2));
}

pub fn simulate_key_up(key: VirtualKey) {
    let input: INPUT = key_input(key, true);
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
    sleep(Duration::from_millis(2));
}

pub fn simulate_key_tap(key: VirtualKey) {
    simulate_key_down(key);
    simulate_key_up(key);
}

pub fn get_mouse_pos() -> POINT {
    let mut p = POINT::default();
    match unsafe { GetCursorPos(&mut p) } {
        Ok(()) => p,
        Err(_) => POINT::default(),
    }
}

pub fn simulate_mouse_move(p: POINT) {
    let input = mouse_input(
        Some(MouseMovement {
            x: p.x,
            y: p.y,
            translation: TranslateType::Relative,
        }),
        M1State::None,
    );
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn simulate_mouse_goto(p: POINT) {
    let input = mouse_input(
        Some(MouseMovement {
            x: p.x * 34,
            y: p.y * 61,
            translation: TranslateType::Absolute,
        }),
        M1State::None,
    );
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn simulate_mouse_down() {
    let input: INPUT = mouse_input(None, M1State::Press);
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
    sleep(Duration::from_millis(2));
}

pub fn simulate_mouse_up() {
    let input: INPUT = mouse_input(None, M1State::Release);
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
    sleep(Duration::from_millis(2));
}

pub fn simulate_mouse_click() {
    simulate_mouse_down();
    simulate_mouse_up();
}

pub fn simulate_mouse_click_at(p: POINT) {
    simulate_mouse_goto(p);
    simulate_mouse_click();
}

pub fn is_key_down(key: VirtualKey) -> bool {
    (unsafe { GetAsyncKeyState(key as i32) } & !1) != 0
}
