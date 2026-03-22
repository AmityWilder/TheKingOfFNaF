//! windows.h is ew yucky, i dont want it touching the main program
use std::{mem::size_of, thread::sleep, time::Duration};
pub use windows::Win32::Foundation::POINT;
use windows::Win32::{
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC,
        DeleteObject, GetDC, HBITMAP, HDC, HGDIOBJ, RGBQUAD, ReleaseDC, SelectObject,
    },
    UI::{
        Input::KeyboardAndMouse::{
            GetKeyState, INPUT, INPUT_KEYBOARD, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
            MOUSEEVENTF_LEFTUP, SendInput,
        },
        WindowsAndMessaging::{
            GetCursorPos, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
        },
    },
};

#[derive(Debug)]
pub struct WindowsHandles {
    /// get the desktop device context
    pub desktop_hdc: HDC,
    /// create a device context to use ourselves
    pub internal_hdc: HDC,
    /// create a bitmap
    pub bitmap: HBITMAP,
}

impl Drop for WindowsHandles {
    fn drop(&mut self) {
        unsafe {
            _ = DeleteObject(HGDIOBJ(self.bitmap.0)); // Free the bitmap memory to the OS
            _ = DeleteDC(self.internal_hdc); // Destroy our internal display handle
            ReleaseDC(None, self.desktop_hdc); // Free the desktop handle
        }
    }
}

impl WindowsHandles {
    pub fn new() -> Self {
        let screen_width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        super::SCREEN_WIDTH.set(screen_width).unwrap();
        let screen_height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
        super::SCREEN_HEIGHT.set(screen_height).unwrap();

        let desktop_hdc = unsafe { GetDC(None) }; // get the desktop device context
        let internal_hdc = unsafe { CreateCompatibleDC(Some(desktop_hdc)) }; // create a device context to use ourselves

        let bitmap = unsafe { CreateCompatibleBitmap(desktop_hdc, screen_width, screen_height) };
        unsafe {
            SelectObject(internal_hdc, HGDIOBJ(bitmap.0)); // Get a handle to our bitmap
            // why are we ignoring the return?
        }

        Self {
            desktop_hdc,
            internal_hdc,
            bitmap,
        }
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
    VkW = 'W' as isize,
    VkA = 'A' as isize,
    VkS = 'S' as isize,
    VkD = 'D' as isize,
    VkF = 'F' as isize,
    VkC = 'C' as isize,
    VkEnter = '\n' as isize,
    VkSpace = ' ' as isize,
    Vk1 = '1' as isize,
    Vk2 = '2' as isize,
    Vk3 = '3' as isize,
    Vk4 = '4' as isize,
    Vk5 = '5' as isize,
    Vk6 = '6' as isize,
    VkX = 'X' as isize,
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
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT_0, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    };
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

pub fn mouse_input(movement: Option<MouseMovement>, m1: M1State) -> INPUT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_MOVE, MOUSEINPUT,
    };
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: movement.map_or(0, |m| m.x),
                dy: movement.map_or(0, |m| m.y),
                mouseData: 0,
                dwFlags: MOUSE_EVENT_FLAGS(
                    movement.map_or(0, |m| MOUSEEVENTF_MOVE.0 | m.translation as u32) | m1 as u32,
                ),
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

pub fn simulate_keypress(key: VirtualKey) {
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
    (unsafe { GetKeyState(key as i32) } & !1) != 0
}
