//! Windows implementation

#![warn(clippy::undocumented_unsafe_blocks, reason = "audit in-progress")]

use super::*;
use windows::{
    Win32::{
        Foundation::{ERROR_INVALID_PARAMETER, GetLastError, POINT},
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
    core::HRESULT,
};

/// Some impls may require all subsystem handles to belong to a singular shared handle.
/// In that case, they will all reference this one through a [`SharedHandleRef`].
///
/// - May not implement [`Send`] or [`Sync`] because some impls hold mutable pointers.
/// - May not implement [`Clone`] because some impls may risk double-free if duplicated.
/// - Must implement [`Drop`] because most imples with a shared handle will need be cleaned.
#[derive(Debug)]
pub struct SharedHandle {
    /// the desktop device context
    desktop_hdc: HDC,
    /// device context to use ourselves
    internal_hdc: HDC,
}

impl Drop for SharedHandle {
    fn drop(&mut self) {
        unsafe {
            _ = DeleteDC(self.internal_hdc); // Destroy our internal display handle
            ReleaseDC(None, self.desktop_hdc); // Free the desktop handle
        }
    }
}

/// See [`SHandle::InitError`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedHandleInitError {
    /// `GetDC(None)` error
    DesktopHdc,
    /// `CreateCompatibleDC(Some(desktop_hdc))` error
    InternalHdc,
}

impl std::fmt::Display for SharedHandleInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DesktopHdc => "failed to get desktop device context handle",
            Self::InternalHdc => "failed to create device context compatible with desktop hdc",
        }
        .fmt(f)
    }
}

impl std::error::Error for SharedHandleInitError {}

impl SHandle for SharedHandle {
    type InitError = SharedHandleInitError;

    /// Initialize the shared handle
    fn init() -> Result<Self, Self::InitError> {
        let desktop_hdc = unsafe { GetDC(None) }; // get the desktop device context
        let internal_hdc = unsafe { CreateCompatibleDC(Some(desktop_hdc)) }; // create a device context to use ourselves

        let hshared = Self {
            desktop_hdc,
            internal_hdc,
        }; // created early so it can `drop` if there's an error

        if hshared.desktop_hdc.is_invalid() {
            Err(SharedHandleInitError::DesktopHdc)
        } else if hshared.internal_hdc.is_invalid() {
            Err(SharedHandleInitError::InternalHdc)
        } else {
            Ok(hshared)
        }
    }

    fn href(&mut self) -> SharedHandleRef<'_> {
        self
    }
}

/// Depending on the platform, this could be implemented as
/// - An [`Rc<RefCell<SharedHandle>>`](`std::rc::Rc`)
///   (the non-implementation of [`Send`]/[`Sync`] makes
///   [`Arc<Mutex<SharedHandle>>`](`std::sync::Arc`) pointless)
/// - A shared reference with lifetime `'a` to a [`Clone`]able handle
/// - Or a private unit type
///
/// May not implement [`Copy`]
pub type SharedHandleRef<'a> = &'a SharedHandle;

/// User input (keyboard/mouse) handle.
#[derive(Debug)]
pub struct UInputHandle<'a> {
    hshared: SharedHandleRef<'a>,
}

impl Drop for UInputHandle<'_> {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

pub(super) const VK_W: i32 = 'W' as i32;
pub(super) const VK_A: i32 = 'A' as i32;
pub(super) const VK_S: i32 = 'S' as i32;
pub(super) const VK_D: i32 = 'D' as i32;
pub(super) const VK_F: i32 = 'F' as i32;
pub(super) const VK_C: i32 = 'C' as i32;
pub(super) const VK_ENTER: i32 = '\n' as i32;
pub(super) const VK_SPACE: i32 = ' ' as i32;
pub(super) const VK_1: i32 = '1' as i32;
pub(super) const VK_2: i32 = '2' as i32;
pub(super) const VK_3: i32 = '3' as i32;
pub(super) const VK_4: i32 = '4' as i32;
pub(super) const VK_5: i32 = '5' as i32;
pub(super) const VK_6: i32 = '6' as i32;
pub(super) const VK_X: i32 = 'X' as i32;
pub(super) const VK_Z: i32 = 'Z' as i32;
pub(super) const VK_ESC: i32 = '\x1b' as i32;

/// See [`UInput::GetKeyStateError`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GetKeyStateError;

impl std::fmt::Display for GetKeyStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "key state could not be read".fmt(f)
    }
}

impl std::error::Error for GetKeyStateError {}

impl<'a> UInput<'a> for VInputHandle<'a> {
    type InitError = !;

    fn init(hshared: SharedHandleRef<'a>) -> Result<Self, Self::InitError> {
        Ok(Self { hshared })
    }

    type GetKeyStateError = GetKeyStateError;

    /// This implementation can only tell if the key is up or down,
    /// not if it's been pressed or released due to unreliability
    fn get_key_state(&mut self, key: VirtualKey) -> Result<KeyState, Self::GetKeyStateError> {
        let result = unsafe { GetAsyncKeyState(key as i32) };
        if result == 0 {
            Err(GetKeyStateError)
        } else {
            Ok(if result.is_negative() {
                KeyState::Down
            } else {
                KeyState::Up
            })
        }
    }

    type GetMousePosError = windows::core::Error;

    fn get_mouse_pos(&mut self) -> Result<IVec2, Self::GetMousePosError> {
        let mut pt = POINT::default();
        unsafe { GetCursorPos(&mut pt) }.map(|()| IVec2 { x: pt.x, y: pt.y })
    }
}

/// Virtual input (keyboard/mouse) handle.
#[derive(Debug)]
pub struct VInputHandle<'a> {
    hshared: SharedHandleRef<'a>,
}

impl Drop for VInputHandle<'_> {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

/// WARNING: May be out of date if additional errors have occurred since this one.
#[derive(Debug, Clone)]
pub struct SendInputError(HRESULT);

impl std::fmt::Display for SendInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to send one or more inputs: {}", self.0.message())
    }
}

impl std::error::Error for SendInputError {}

impl<'a> VInput<'a> for VInputHandle<'a> {
    type InitError = !;

    /// Initialize the virtual input handle from the [shared handle](`SharedHandle`).
    fn init(hshared: SharedHandleRef<'a>) -> Result<Self, Self::InitError> {
        Ok(Self { hshared })
    }

    type SimulateMouseEventError = SendInputError;

    fn simulate_mouse_event(
        &mut self,
        event: MouseEvent,
    ) -> Result<(), Self::SimulateMouseEventError> {
        let (goto, m1) = event.unzip();
        let (dx, dy, goto_flags) = goto.map_or((0, 0, MOUSE_EVENT_FLAGS::default()), |goto| {
            (
                goto.x * 34,
                goto.y * 61,
                MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
            )
        });
        let m1_flags = m1.map_or(MOUSE_EVENT_FLAGS::default(), |m1| match m1 {
            KeyState::Down => MOUSEEVENTF_LEFTDOWN,
            KeyState::Up => MOUSEEVENTF_LEFTUP,
        });
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: 0,
                    dwFlags: goto_flags | m1_flags,
                    time: 0, // Pleaseeeee don't mess with this... it makes the monitor go funky...
                    dwExtraInfo: 0,
                },
            },
        };
        if unsafe { SendInput(&[input], size_of::<INPUT>() as i32) } == 0 {
            return Err(SendInputError(unsafe { GetLastError() }.to_hresult()));
        }
        Ok(())
    }

    type SimulateKeyEventError = SendInputError;

    fn simulate_key_event(
        &mut self,
        keys: &[VirtualKey],
        state: KeyState,
    ) -> Result<(), Self::SimulateKeyEventError> {
        for &key in keys {
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(key as u16),
                        wScan: 0,
                        dwFlags: match state {
                            KeyState::Up => KEYEVENTF_KEYUP,
                            _ => KEYBD_EVENT_FLAGS(0), // key down
                        },
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            if unsafe { SendInput(&[input], size_of::<INPUT>() as i32) } == 0 {
                return Err(SendInputError(unsafe { GetLastError() }.to_hresult()));
            }
        }
        Ok(())
    }
}

/// May not implement [`Send`], [`Sync`], or [`Clone`]. Must implement [`Drop`].
#[derive(Debug)]
pub struct ScreenHandle<'a> {
    hshared: SharedHandleRef<'a>,
    bitmap: HBITMAP,
    buffer: Box<[u8]>,

    screen_width: i32,
    screen_height: i32,
}

impl Drop for ScreenHandle<'_> {
    fn drop(&mut self) {
        _ = unsafe { DeleteObject(HGDIOBJ(self.bitmap.0)) }; // Free the bitmap memory to the OS
    }
}

/// Error returned by [`ScreenHandle::init`]
#[derive(Debug, Clone)]
pub enum InitScreenHandleError {
    /// `GetSystemMetrics` failed
    ScreenSize,
    /// `CreateCompatibleBitmap(hshared.desktop_hdc, screen_width, screen_height)` failed
    CreateBitmap,
    /// `SelectObject(hshared.internal_hdc, HGDIOBJ(bitmap.0))` failed
    SelectBitmap,
}

impl std::fmt::Display for InitScreenHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScreenSize => "failed to obtain virtual screen size system metrics",
            Self::CreateBitmap => "failed to create a compatible bitmap with the desktop hdc",
            Self::SelectBitmap => "failed to select the created bitmap into the internal hdc",
        }
        .fmt(f)
    }
}

impl std::error::Error for InitScreenHandleError {}

impl<'a> Screen<'a> for ScreenHandle<'a> {
    type InitError = InitScreenHandleError;

    /// Initialize the screen handle from the [shared handle](`SharedHandle`).
    fn init(hshared: SharedHandleRef<'a>) -> Result<Self, Self::InitError> {
        let screen_width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
        if screen_width == 0 || screen_height == 0 {
            return Err(InitScreenHandleError::ScreenSize);
        }

        let bitmap =
            unsafe { CreateCompatibleBitmap(hshared.desktop_hdc, screen_width, screen_height) };
        if bitmap.is_invalid() {
            return Err(InitScreenHandleError::CreateBitmap);
        }

        // Get a handle to our bitmap
        if unsafe { SelectObject(hshared.internal_hdc, HGDIOBJ(bitmap.0)) }.is_invalid() {
            unsafe { DeleteObject(bitmap.into()) };
            return Err(InitScreenHandleError::SelectBitmap);
        }

        Ok(Self {
            hshared,
            bitmap,
            buffer: vec![
                0;
                screen_width.unsigned_abs() as usize
                    * screen_height.unsigned_abs() as usize
                    * 4
            ]
            .into_boxed_slice(),

            screen_width,
            screen_height,
        })
    }

    type HintRefreshScreencapError = windows::core::Error;

    fn hint_refresh_screencap(&mut self) -> Result<(), Self::HintRefreshScreencapError> {
        let res = unsafe {
            BitBlt(
                self.hshared.internal_hdc,
                0,
                0,
                self.screen_width,
                self.screen_height,
                Some(self.hshared.desktop_hdc),
                0,
                0,
                SRCCOPY,
            )?;

            GetDIBits(
                self.hshared.desktop_hdc,
                self.bitmap,
                0,
                self.screen_height as u32,
                Some(self.buffer.as_mut_ptr().cast()),
                &mut BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: self.screen_width,
                        biHeight: -self.screen_height,
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
                },
                DIB_RGB_COLORS,
            )
        };
        if res == 0 {
            return Err(windows::core::Error::from(ERROR_INVALID_PARAMETER));
        }
        Ok(())
    }

    fn width(&mut self) -> Result<i32, Self::GetSizeError> {
        Ok(self.screen_width)
    }

    fn height(&mut self) -> Result<i32, Self::GetSizeError> {
        Ok(self.screen_height)
    }

    fn get_pixel(&mut self, pt: IVec2) -> Result<ColorRGB, Self::GetPixelError> {
        let index: usize = 4 * ((pt.y * self.screen_width) + pt.x) as usize;

        Ok(ColorRGB {
            r: self.buffer[index + 2],
            g: self.buffer[index + 1],
            b: self.buffer[index],
        })
    }

    fn get_region(
        &mut self,
        _rgn: IRect,
        _buffer: &mut [ColorRGB],
    ) -> Result<usize, Self::GetPixelError> {
        todo!() // definitely possible, but I'm not sure how to do it yet...
    }
}
