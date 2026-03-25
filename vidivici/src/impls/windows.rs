//! Ensures the program can compile whether we're on windows or not, even if it can't operate outside of windows

use super::*;
use std::ffi::c_char;
use windows::{
    Win32::{
        Foundation::POINT,
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap,
            CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP,
            HDC, HGDIOBJ, RGBQUAD, ReleaseDC, SRCCOPY, SelectObject,
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS,
                KEYBDINPUT, KEYEVENTF_KEYUP, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE,
                MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT, SendInput,
                VIRTUAL_KEY,
            },
            WindowsAndMessaging::{
                GetCursorPos, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
            },
        },
    },
    core::Result,
};

#[derive(Debug)]
pub struct WindowsHandles {
    /// get the desktop device context
    desktop_hdc: HDC,
    /// create a device context to use ourselves
    internal_hdc: HDC,
    /// create a bitmap
    bitmap: HBITMAP,

    pub screen_width: i32,
    pub screen_height: i32,
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
        let screen_height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

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
            screen_width,
            screen_height,
        }
    }

    pub fn bitblt(&self, buffer: &mut [u8]) -> IoResult<()> {
        unsafe {
            BitBlt(
                self.internal_hdc,
                0,
                0,
                self.screen_width,
                self.screen_height,
                Some(self.desktop_hdc),
                0,
                0,
                SRCCOPY,
            )?;

            GetDIBits(
                self.desktop_hdc,
                self.bitmap,
                0,
                self.screen_height as u32,
                Some(buffer.as_mut_ptr().cast()),
                &mut bitmap_info(self.screen_width, self.screen_height),
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

pub const VK_W: c_char = b'W' as c_char;
pub const VK_A: c_char = b'A' as c_char;
pub const VK_S: c_char = b'S' as c_char;
pub const VK_D: c_char = b'D' as c_char;
pub const VK_F: c_char = b'F' as c_char;
pub const VK_C: c_char = b'C' as c_char;
pub const VK_ENTER: c_char = b'\n' as c_char;
pub const VK_SPACE: c_char = b' ' as c_char;
pub const VK_1: c_char = b'1' as c_char;
pub const VK_2: c_char = b'2' as c_char;
pub const VK_3: c_char = b'3' as c_char;
pub const VK_4: c_char = b'4' as c_char;
pub const VK_5: c_char = b'5' as c_char;
pub const VK_6: c_char = b'6' as c_char;
pub const VK_X: c_char = b'X' as c_char;
pub const VK_Z: c_char = b'Z' as c_char;
pub const VK_ESC: c_char = b'\x1b' as c_char;

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

pub fn mouse_input(movement: Option<MouseMovement>, m1: M1State) -> INPUT {
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

pub fn simulate_key_tap(key: VirtualKey) {
    simulate_key_down(key);
    simulate_key_up(key);
}

pub fn get_mouse_pos() -> IVec2 {
    let mut p = IVec2::default();
    match unsafe { GetCursorPos(&mut p) } {
        Ok(()) => p,
        Err(_) => IVec2::default(),
    }
}

pub fn simulate_mouse_move(p: IVec2) {
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

pub fn simulate_mouse_goto(p: IVec2) {
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

pub fn simulate_mouse_click_at(p: IVec2) {
    simulate_mouse_goto(p);
    simulate_mouse_click();
}

pub fn is_key_down(key: VirtualKey) -> bool {
    (unsafe { GetAsyncKeyState(key as i32) } & !1) != 0
}
