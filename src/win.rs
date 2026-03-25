//! windows.h is ew yucky, i dont want it touching the main program

#![warn(
    clippy::undocumented_unsafe_blocks,
    reason = "do not create soundness holes in communicating with the OS"
)] // TODO: change this to deny once windows.h safety has been cleared up

use std::{ffi::CStr, mem::size_of, thread::sleep, time::Duration};

#[cfg(not(windows))]
mod wrapper_nonwindows;
#[cfg(windows)]
mod wrapper_windows;

use windows::{
    Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{BITMAP, GetObjectA, STRETCH_HALFTONE, SetStretchBltMode, StretchBlt},
        UI::WindowsAndMessaging::{FindWindowA, GetClientRect, SM_CXSCREEN, SM_CYSCREEN},
    },
    core::PCSTR,
};
#[cfg(not(windows))]
use wrapper_nonwindows::*;
#[cfg(windows)]
use wrapper_windows::*;

#[cfg(not(windows))]
pub use wrapper_nonwindows::POINT;
#[cfg(windows)]
pub use wrapper_windows::POINT;

#[derive(Debug)]
struct CommonHdc(HDC);

impl Drop for CommonHdc {
    fn drop(&mut self) {
        // SAFETY: CommonHdc contains a mutable pointer, so it denies Send/Sync,
        // guaranteeing this thread is the one it was created on.
        unsafe {
            _ = ReleaseDC(None, self.0);
        }
    }
}

impl CommonHdc {
    unsafe fn get() -> Result<Self, ()> {
        // SAFETY: Caller must uphold safety contract
        let hdc = unsafe { GetDC(None) };
        if !hdc.is_invalid() {
            Ok(Self(hdc))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
struct WindowHdc(HWND, HDC);

impl Drop for WindowHdc {
    fn drop(&mut self) {
        // SAFETY: WindowHdc contains a mutable pointer, so it denies Send/Sync,
        // guaranteeing this thread is the one it was created on.
        unsafe {
            _ = ReleaseDC(Some(self.0), self.1);
        }
    }
}

impl WindowHdc {
    unsafe fn get(hwnd: HWND) -> Result<Self, ()> {
        // SAFETY: Caller must uphold safety contract
        let hdc = unsafe { GetDC(None) };
        if !hdc.is_invalid() {
            Ok(Self(hwnd, hdc))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
struct CompatibleHdc(HDC);

impl Drop for CompatibleHdc {
    fn drop(&mut self) {
        // SAFETY: CompatibleHdc contains a mutable pointer, so it denies Send/Sync,
        // guaranteeing this thread is the one it was created on.
        unsafe {
            _ = DeleteDC(self.0);
        }
    }
}

impl CompatibleHdc {
    unsafe fn create(hdc: HDC) -> Result<Self, ()> {
        // SAFETY: Caller must uphold safety contract
        let c_hdc = unsafe { CreateCompatibleDC(Some(hdc)) };
        if !c_hdc.is_invalid() {
            Ok(Self(c_hdc))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
struct HBitmap(HBITMAP);

impl Drop for HBitmap {
    fn drop(&mut self) {
        // SAFETY: HBitmap contains a mutable pointer, so it denies Send/Sync,
        // guaranteeing this thread is the one it was created on.
        unsafe {
            _ = DeleteObject(self.0.into());
        }
    }
}

impl HBitmap {
    unsafe fn create(hdc: HDC, cx: i32, cy: i32) -> Result<Self, ()> {
        // SAFETY: Caller must uphold safety contract
        let hbm = unsafe { CreateCompatibleBitmap(hdc, cx, cy) };
        if !hbm.is_invalid() {
            Ok(Self(hbm))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct WindowsHandles {
    hdc_screen: CommonHdc,
    hdc_window: WindowHdc,
    hdc_mem_dc: CompatibleHdc,
    hbm_screen: HBitmap,
    pub bmp_screen: BITMAP,
    pub lpbitmap: Box<[u8]>,
    pub rc_client: RECT,
    pub bi: BITMAPINFO,
}

impl WindowsHandles {
    pub fn new(windowname: &CStr) -> Result<Self, ()> {
        let h_wnd = unsafe { FindWindowA(None, PCSTR::from_raw(windowname.as_ptr().cast())) }
            .map_err(|_| ())?;
        let hdc_screen = unsafe { CommonHdc::get() }?;
        let hdc_window = unsafe { WindowHdc::get(h_wnd) }?;
        _ = h_wnd; // access through hdc_window.0
        let hdc_mem_dc = unsafe { CompatibleHdc::create(hdc_window.1) }?;

        let mut rc_client = RECT::default();
        // SAFETY: GetClientRect has no safety requirements (sus)
        unsafe { GetClientRect(hdc_window.0, &mut rc_client) }.map_err(|_| ())?;

        unsafe {
            SetStretchBltMode(hdc_window.1, STRETCH_HALFTONE);
        }

        if !unsafe {
            StretchBlt(
                hdc_window.1,
                0,
                0,
                rc_client.right,
                rc_client.bottom,
                Some(hdc_screen.0),
                0,
                0,
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
                SRCCOPY,
            )
        }
        .as_bool()
        {
            return Err(());
        }

        let hbm_screen = unsafe {
            HBitmap::create(
                hdc_window.1,
                rc_client.right - rc_client.left,
                rc_client.bottom - rc_client.top,
            )
        }?;

        unsafe {
            SelectObject(hdc_mem_dc.0, hbm_screen.0.into());
        }

        let mut bmp_screen = BITMAP::default();

        if unsafe {
            BitBlt(
                hdc_mem_dc.0,
                0,
                0,
                rc_client.right - rc_client.left,
                rc_client.bottom - rc_client.top,
                Some(hdc_window.1),
                0,
                0,
                SRCCOPY,
            )
        }
        .is_err()
        {
            return Err(());
        }

        unsafe {
            GetObjectA(
                hbm_screen.0.into(),
                std::mem::size_of::<BITMAP>() as i32,
                Some((&raw mut bmp_screen).cast()),
            );
        }

        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bmp_screen.bmWidth,
                biHeight: bmp_screen.bmHeight,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default(); 1], // windows does this unsoundly
        };

        let dw_bmp_size = ((bmp_screen.bmWidth * bi.bmiHeader.biBitCount as i32 + 31) / 32)
            * 4
            * bmp_screen.bmHeight;

        let mut lpbitmap = vec![0; dw_bmp_size as usize].into_boxed_slice();

        unsafe {
            GetDIBits(
                hdc_window.1,
                hbm_screen.0,
                0,
                bmp_screen.bmHeight as u32,
                Some(lpbitmap.as_mut_ptr().cast()),
                &mut bi,
                DIB_RGB_COLORS,
            );
        }

        Ok(Self {
            hdc_screen,
            hdc_window,
            hdc_mem_dc,
            hbm_screen,
            bmp_screen,
            lpbitmap,
            rc_client,
            bi,
        })
    }

    pub fn screenshot(&mut self) -> Result<(), ()> {
        unsafe {
            SelectObject(self.hdc_mem_dc.0, self.hbm_screen.0.into());
        }

        if unsafe {
            BitBlt(
                self.hdc_mem_dc.0,
                0,
                0,
                self.rc_client.right - self.rc_client.left,
                self.rc_client.bottom - self.rc_client.top,
                Some(self.hdc_window.1),
                0,
                0,
                SRCCOPY,
            )
        }
        .is_err()
        {
            return Err(());
        }

        unsafe {
            GetObjectA(
                self.hbm_screen.0.into(),
                std::mem::size_of::<BITMAP>() as i32,
                Some((&raw mut self.bmp_screen).cast()),
            );
        }

        unsafe {
            GetDIBits(
                self.hdc_window.1,
                self.hbm_screen.0,
                0,
                self.bmp_screen.bmHeight as u32,
                Some(self.lpbitmap.as_mut_ptr().cast()),
                &mut self.bi,
                DIB_RGB_COLORS,
            );
        }

        Ok(())
    }

    #[inline]
    pub const fn swap_buffer(&mut self, buffer: &mut Box<[u8]>) {
        std::mem::swap(&mut self.lpbitmap, buffer);
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
