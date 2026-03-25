//! Ensures the program can compile whether we're on windows or not, even if it can't operate outside of windows

#![allow(
    nonstandard_style,
    unused_variables,
    clippy::style,
    clippy::too_many_arguments
)]

#[derive(Clone)]
pub struct Error(());

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

/// A specialized [`Result`] type that provides Windows error information.
pub type WindowsResult<T> = std::result::Result<T, Error>;

/// A 32-bit value representing boolean values and returned by some functions to indicate success or failure.
#[must_use]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BOOL(pub i32);

impl BOOL {
    /// Converts the [`BOOL`] to a [`prim@bool`] value.
    #[inline]
    pub fn as_bool(self) -> bool {
        unimplemented!()
    }

    /// Converts the [`BOOL`] to [`Result<()>`][Result<_>].
    #[inline]
    pub fn ok(self) -> Result<()> {
        unimplemented!()
    }

    /// Asserts that `self` is a success code.
    #[inline]
    #[track_caller]
    pub fn unwrap(self) {
        unimplemented!()
    }

    /// Asserts that `self` is a success code using the given panic message.
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) {
        unimplemented!()
    }
}

impl From<BOOL> for bool {
    fn from(value: BOOL) -> Self {
        unimplemented!()
    }
}

impl From<&BOOL> for bool {
    fn from(value: &BOOL) -> Self {
        unimplemented!()
    }
}

impl From<bool> for BOOL {
    fn from(value: bool) -> Self {
        unimplemented!()
    }
}

impl From<&bool> for BOOL {
    fn from(value: &bool) -> Self {
        unimplemented!()
    }
}

impl PartialEq<bool> for BOOL {
    fn eq(&self, other: &bool) -> bool {
        unimplemented!()
    }
}

impl PartialEq<BOOL> for bool {
    fn eq(&self, other: &BOOL) -> bool {
        unimplemented!()
    }
}

impl core::ops::Not for BOOL {
    type Output = Self;
    fn not(self) -> Self::Output {
        unimplemented!()
    }
}

pub struct BI_COMPRESSION(pub u32);
pub const BI_RGB: BI_COMPRESSION = BI_COMPRESSION(0u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1],
}
impl Default for BITMAPINFO {
    fn default() -> Self {
        unimplemented!()
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}

#[inline]
pub unsafe fn BitBlt(
    hdc: HDC,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    hdcsrc: Option<HDC>,
    x1: i32,
    y1: i32,
    rop: ROP_CODE,
) -> Result<()> {
    unimplemented!()
}

#[inline]
pub unsafe fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP {
    unimplemented!()
}
#[inline]
pub unsafe fn CreateCompatibleDC(hdc: Option<HDC>) -> HDC {
    unimplemented!()
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DIB_USAGE(pub u32);

pub const DIB_RGB_COLORS: DIB_USAGE = DIB_USAGE(0u32);

#[inline]
pub unsafe fn DeleteDC(hdc: HDC) -> BOOL {
    unimplemented!()
}

#[inline]
pub unsafe fn DeleteObject(ho: HGDIOBJ) -> BOOL {
    unimplemented!()
}

#[inline]
pub unsafe fn GetDC(hwnd: Option<HWND>) -> HDC {
    unimplemented!()
}

#[inline]
pub unsafe fn GetDIBits(
    hdc: HDC,
    hbm: HBITMAP,
    start: u32,
    clines: u32,
    lpvbits: Option<*mut core::ffi::c_void>,
    lpbmi: *mut BITMAPINFO,
    usage: DIB_USAGE,
) -> i32 {
    unimplemented!()
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HBITMAP(pub *mut core::ffi::c_void);
impl HBITMAP {
    pub fn is_invalid(&self) -> bool {
        unimplemented!()
    }
}

impl Default for HBITMAP {
    fn default() -> Self {
        unimplemented!()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HDC(pub *mut core::ffi::c_void);
impl HDC {
    pub fn is_invalid(&self) -> bool {
        unimplemented!()
    }
}
impl Default for HDC {
    fn default() -> Self {
        unimplemented!()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HWND(pub *mut core::ffi::c_void);
impl HWND {
    pub fn is_invalid(&self) -> bool {
        unimplemented!()
    }
}
impl Default for HWND {
    fn default() -> Self {
        unimplemented!()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HGDIOBJ(pub *mut core::ffi::c_void);
impl HGDIOBJ {
    pub fn is_invalid(&self) -> bool {
        unimplemented!()
    }
}
impl Default for HGDIOBJ {
    fn default() -> Self {
        unimplemented!()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RGBQUAD {
    pub rgbBlue: u8,
    pub rgbGreen: u8,
    pub rgbRed: u8,
    pub rgbReserved: u8,
}

#[inline]
pub unsafe fn ReleaseDC(hwnd: Option<HWND>, hdc: HDC) -> i32 {
    unimplemented!()
}

pub const SRCCOPY: ROP_CODE = ROP_CODE(13369376u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ROP_CODE(pub u32);
impl ROP_CODE {
    pub const fn contains(&self, other: Self) -> bool {
        unimplemented!()
    }
}
impl core::ops::BitOr for ROP_CODE {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        unimplemented!()
    }
}
impl core::ops::BitAnd for ROP_CODE {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        unimplemented!()
    }
}
impl core::ops::BitOrAssign for ROP_CODE {
    fn bitor_assign(&mut self, other: Self) {
        unimplemented!()
    }
}
impl core::ops::BitAndAssign for ROP_CODE {
    fn bitand_assign(&mut self, other: Self) {
        unimplemented!()
    }
}
impl core::ops::Not for ROP_CODE {
    type Output = Self;
    fn not(self) -> Self {
        unimplemented!()
    }
}

#[inline]
pub unsafe fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ {
    unimplemented!()
}

#[inline]
pub unsafe fn GetAsyncKeyState(vkey: i32) -> i16 {
    unimplemented!()
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct INPUT {
    pub r#type: INPUT_TYPE,
    pub Anonymous: INPUT_0,
}
impl Default for INPUT {
    fn default() -> Self {
        unimplemented!()
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union INPUT_0 {
    pub mi: MOUSEINPUT,
    pub ki: KEYBDINPUT,
    pub hi: HARDWAREINPUT,
}
impl Default for INPUT_0 {
    fn default() -> Self {
        unimplemented!()
    }
}

pub const INPUT_HARDWARE: INPUT_TYPE = INPUT_TYPE(2u32);
pub const INPUT_KEYBOARD: INPUT_TYPE = INPUT_TYPE(1u32);
pub const INPUT_MOUSE: INPUT_TYPE = INPUT_TYPE(0u32);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct INPUT_TYPE(pub u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MOUSEINPUT {
    pub dx: i32,
    pub dy: i32,
    pub mouseData: u32,
    pub dwFlags: MOUSE_EVENT_FLAGS,
    pub time: u32,
    pub dwExtraInfo: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MOUSEMOVEPOINT {
    pub x: i32,
    pub y: i32,
    pub time: u32,
    pub dwExtraInfo: usize,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MOUSE_EVENT_FLAGS(pub u32);
impl MOUSE_EVENT_FLAGS {
    pub const fn contains(&self, other: Self) -> bool {
        unimplemented!()
    }
}
impl core::ops::BitOr for MOUSE_EVENT_FLAGS {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        unimplemented!()
    }
}
impl core::ops::BitAnd for MOUSE_EVENT_FLAGS {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        unimplemented!()
    }
}
impl core::ops::BitOrAssign for MOUSE_EVENT_FLAGS {
    fn bitor_assign(&mut self, other: Self) {
        unimplemented!()
    }
}
impl core::ops::BitAndAssign for MOUSE_EVENT_FLAGS {
    fn bitand_assign(&mut self, other: Self) {
        unimplemented!()
    }
}
impl core::ops::Not for MOUSE_EVENT_FLAGS {
    type Output = Self;
    fn not(self) -> Self {
        unimplemented!()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VIRTUAL_KEY(pub u16);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KEYBDINPUT {
    pub wVk: VIRTUAL_KEY,
    pub wScan: u16,
    pub dwFlags: KEYBD_EVENT_FLAGS,
    pub time: u32,
    pub dwExtraInfo: usize,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KEYBD_EVENT_FLAGS(pub u32);
impl KEYBD_EVENT_FLAGS {
    pub const fn contains(&self, other: Self) -> bool {
        unimplemented!()
    }
}
impl core::ops::BitOr for KEYBD_EVENT_FLAGS {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        unimplemented!()
    }
}
impl core::ops::BitAnd for KEYBD_EVENT_FLAGS {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        unimplemented!()
    }
}
impl core::ops::BitOrAssign for KEYBD_EVENT_FLAGS {
    fn bitor_assign(&mut self, other: Self) {
        unimplemented!()
    }
}
impl core::ops::BitAndAssign for KEYBD_EVENT_FLAGS {
    fn bitand_assign(&mut self, other: Self) {
        unimplemented!()
    }
}
impl core::ops::Not for KEYBD_EVENT_FLAGS {
    type Output = Self;
    fn not(self) -> Self {
        unimplemented!()
    }
}
pub const KEYEVENTF_KEYUP: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(2u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HARDWAREINPUT {
    pub uMsg: u32,
    pub wParamL: u16,
    pub wParamH: u16,
}

pub const MOUSEEVENTF_ABSOLUTE: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(32768u32);
pub const MOUSEEVENTF_LEFTDOWN: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(2u32);
pub const MOUSEEVENTF_LEFTUP: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(4u32);
pub const MOUSEEVENTF_MOVE: MOUSE_EVENT_FLAGS = MOUSE_EVENT_FLAGS(1u32);

#[inline]
pub unsafe fn SendInput(pinputs: &[INPUT], cbsize: i32) -> u32 {
    unimplemented!()
}

#[inline]
pub unsafe fn GetCursorPos(lppoint: *mut POINT) -> Result<()> {
    unimplemented!()
}

#[inline]
pub unsafe fn GetSystemMetrics(nindex: SYSTEM_METRICS_INDEX) -> i32 {
    unimplemented!()
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SYSTEM_METRICS_INDEX(pub i32);
pub const SM_CXVIRTUALSCREEN: SYSTEM_METRICS_INDEX = SYSTEM_METRICS_INDEX(78i32);
pub const SM_CYVIRTUALSCREEN: SYSTEM_METRICS_INDEX = SYSTEM_METRICS_INDEX(79i32);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
