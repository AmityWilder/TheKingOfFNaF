//! Ensures the program can compile whether we're on windows or not, even if it can't operate outside of windows

pub use windows::{Win32::{
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP, HDC, HGDIOBJ, RGBQUAD,
        ReleaseDC, SRCCOPY, SelectObject,
    },
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, INPUT, INPUT_KEYBOARD, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
            MOUSEEVENTF_LEFTUP, SendInput,
            INPUT_0, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
            INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_MOVE, MOUSEINPUT,
        },
        WindowsAndMessaging::{
            GetCursorPos, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
        },
    },
    Win32::Foundation::POINT
}, core::Result};
