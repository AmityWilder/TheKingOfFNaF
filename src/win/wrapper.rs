//! Ensures the program can compile whether we're on windows or not, even if it can't operate outside of windows

use std::{
    borrow::Cow,
    ffi::c_void,
    num::{NonZeroI32, NonZeroU32},
    ptr::NonNull,
};

use windows::{
    Win32::{
        Foundation::{HWND, POINT},
        Graphics::Gdi::{
            BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap,
            CreateCompatibleDC, DIB_PAL_COLORS, DIB_RGB_COLORS, DIB_USAGE, DeleteDC, DeleteObject,
            GetDC, GetDIBits, GetObjectA, GetWindowDC, HBITMAP, HDC, HGDIOBJ, RGBQUAD, ROP_CODE,
            ReleaseDC, SRCCOPY, SelectObject,
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, HARDWAREINPUT, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
                KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSE_EVENT_FLAGS,
                MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE,
                MOUSEINPUT, SendInput, VIRTUAL_KEY,
            },
            WindowsAndMessaging::{
                FindWindowA, GetCursorPos, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
            },
        },
    },
    core::{BOOL, PCSTR, Result as WinResult},
};

pub type RopCode = ROP_CODE;

pub type Point = POINT;

trait AsDc {
    fn as_dc(&self) -> HDC;
}

/// Marker trait for types that can be used as [`HDC`]
pub trait DeviceContext: AsDc {}

impl<T: AsDc> DeviceContext for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Hdc_(NonNull<c_void>);

impl Hdc_ {
    #[inline]
    const fn as_hdc(self) -> HDC {
        HDC(self.0.as_ptr())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Hwnd_(NonNull<c_void>);

impl Hwnd_ {
    #[inline]
    const fn as_hwnd(self) -> HWND {
        HWND(self.0.as_ptr())
    }
}

#[derive(Debug, PartialEq)]
pub struct CommonHdc(Hdc_);

impl AsDc for CommonHdc {
    #[inline]
    fn as_dc(&self) -> HDC {
        self.0.as_hdc()
    }
}

impl Drop for CommonHdc {
    fn drop(&mut self) {
        // SAFETY: CommonHdc contains a mutable pointer, so it does not implement send/sync,
        // therefore it must have been created on the current thread.
        //
        // Its field is private and the structure does not implement Clone, so
        // the HDC cannot be double-freed.
        //
        // It can only be created through GetDC, so it is correct to use ReleaseDC
        // instead of CreateDC.
        unsafe {
            ReleaseDC(None, self.0.as_hdc());
        }
    }
}

impl CommonHdc {
    pub fn get_dc() -> Result<Self, ()> {
        let hdc = unsafe { GetDC(None) };
        if !hdc.is_invalid()
            && let Some(hdc) = NonNull::new(hdc.0)
        {
            Ok(Self(Hdc_(hdc)))
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct Hwnd(Hwnd_);

#[derive(Debug, PartialEq)]
pub struct HwndHdc(Hwnd_, Hdc_);

impl AsDc for HwndHdc {
    #[inline]
    fn as_dc(&self) -> HDC {
        self.1.as_hdc()
    }
}

impl Drop for HwndHdc {
    fn drop(&mut self) {
        // SAFETY: HwndHdc contains a mutable pointer, so it does not implement send/sync,
        // therefore it must have been created on the current thread.
        //
        // Its fields are private and the structure does not implement Clone, so
        // the HDC cannot be double-freed.
        //
        // It can only be created through GetWindowDC, so it is correct to use ReleaseDC
        // instead of CreateDC.
        unsafe {
            ReleaseDC(Some(self.0.as_hwnd()), self.1.as_hdc());
        }
    }
}

impl HwndHdc {
    pub fn get_window_dc(hwnd: Hwnd) -> Result<Self, Hwnd> {
        let hdc = unsafe { GetWindowDC(Some(hwnd.0.as_hwnd())) };
        if !hdc.is_invalid()
            && let Some(hdc) = NonNull::new(hdc.0)
        {
            let hwnd_ = hwnd.0;
            std::mem::forget(hwnd);
            Ok(Self(hwnd_, Hdc_(hdc)))
        } else {
            Err(hwnd)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CompatibleDc(Hdc_);

impl AsDc for CompatibleDc {
    #[inline]
    fn as_dc(&self) -> HDC {
        self.0.as_hdc()
    }
}

impl Drop for CompatibleDc {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.0.as_hdc());
        }
    }
}

impl CompatibleDc {
    pub fn create<H: AsDc>(hdc: &H) -> Result<Self, ()> {
        let dc = unsafe { CreateCompatibleDC(Some(hdc.as_dc())) };
        if !dc.is_invalid()
            && let Some(dc) = NonNull::new(dc.0)
        {
            Ok(Self(Hdc_(dc)))
        } else {
            Err(())
        }
    }
}

pub type BitmapInfoHeader = BITMAPINFOHEADER;

pub type RgbQuad = RGBQUAD;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HBitmap_(NonNull<c_void>);

impl HBitmap_ {
    #[inline]
    const fn as_hbitmap(&self) -> HBITMAP {
        HBITMAP(self.0.as_ptr())
    }

    #[inline]
    const fn as_hgdi_obj(&self) -> HGDIOBJ {
        HGDIOBJ(self.0.as_ptr())
    }
}

trait AsHgdiObj {
    fn as_hgdi_obj(&self) -> HGDIOBJ;
}

trait AsDDB {
    fn as_ddb(&self) -> HBITMAP;
}

#[derive(Debug, PartialEq)]
pub struct HBitmap(HBitmap_);

impl AsDDB for HBitmap {
    fn as_ddb(&self) -> HBITMAP {
        self.0.as_hbitmap()
    }
}

impl Drop for HBitmap {
    fn drop(&mut self) {
        // SAFETY: HBitmap contains a mutable pointer, so it does not implement send/sync,
        // therefore it must have been created on the current thread.
        //
        // Its field is private and the structure does not implement Clone, so
        // the handle cannot be double-freed.
        //
        // It can only be created through CreateCompatibleBitmap, so it is correct to use
        // DeleteObject.
        unsafe {
            DeleteObject(self.0.as_hgdi_obj());
        }
    }
}

impl HBitmap {
    /// # Parameters
    ///
    /// `[in] hdc`
    ///
    /// A handle to a device context.
    ///
    /// `[in] cx`
    ///
    /// The bitmap width, in pixels.
    ///
    /// `[in] cy`
    ///
    /// The bitmap height, in pixels.
    ///
    /// # Return value
    ///
    /// If the function succeeds, the return value is a handle to the compatible bitmap (DDB).
    ///
    /// If the function fails, the return value is [`None`].
    pub fn create_compatible_bitmap<H: AsDc>(hdc: &H, cx: i32, cy: i32) -> Result<Self, ()> {
        let hbitmap = unsafe { CreateCompatibleBitmap(hdc.as_dc(), cx, cy) };
        if !hbitmap.is_invalid()
            && let Some(hbitmap) = NonNull::new(hbitmap.0)
        {
            Ok(Self(HBitmap_(hbitmap)))
        } else {
            Err(())
        }
    }
}

/// Denies the HBitmap from being selected more than once
#[derive(Debug, PartialEq)]
pub struct SelectedHBitmap(HBitmap);

impl AsHgdiObj for SelectedHBitmap {
    #[inline]
    fn as_hgdi_obj(&self) -> HGDIOBJ {
        self.0.0.as_hgdi_obj()
    }
}

impl AsDDB for SelectedHBitmap {
    fn as_ddb(&self) -> HBITMAP {
        self.0.0.as_hbitmap()
    }
}

trait GetObject<Buffer> {
    /// The [`get_object`](GetObject::get_object) function retrieves information for the specified graphics object.
    ///
    /// # Parameters
    ///
    /// `[in] h`
    ///
    /// A handle to the graphics object of interest. This can be a handle to one of the following: a logical bitmap,
    /// a brush, a font, a palette, a pen, or a device independent bitmap created by calling the [`CreateDIBSection`] function.
    ///
    /// `[in] c`
    ///
    /// The number of bytes of information to be written to the buffer.
    ///
    /// `[out] pv`
    ///
    /// A pointer to a buffer that receives the information about the specified graphics object.
    ///
    /// If the `lpvObject` parameter is [`None`], the function return value is the number of bytes required to store the
    /// information it writes to the buffer for the specified graphics object.
    ///
    /// The address of `lpvObject` must be on a 4-byte boundary; otherwise, [`get_object`] fails.
    ///
    /// # Return value
    ///
    /// If the function succeeds, and `lpvObject` is a valid pointer, the return value is the number of bytes stored
    /// into the buffer.
    ///
    /// If the function succeeds, and `lpvObject` is [`None`], the return value is the number of bytes required to hold the
    /// information the function would store into the buffer.
    ///
    /// If the function fails, the return value is zero.
    ///
    /// # Remarks
    ///
    /// The buffer pointed to by the `lpvObject` parameter must be sufficiently large to receive the information about
    /// the graphics object. Depending on the graphics object, the function uses a `BITMAP`, `DIBSECTION`, `EXTLOGPEN`, `LOGBRUSH`,
    /// `LOGFONT`, or `LOGPEN` structure, or a count of table entries (for a logical palette).
    ///
    /// If `hgdiobj` is a handle to a bitmap created by calling CreateDIBSection, and the specified buffer is large enough,
    /// the [`get_object`] function returns a `DIBSECTION` structure. In addition, the `bmBits` member of the `BITMAP` structure
    /// contained within the `DIBSECTION` will contain a pointer to the bitmap's bit values.
    ///
    /// If `hgdiobj` is a handle to a bitmap created by any other means, [`get_object`] returns only the width, height, and color
    /// format information of the bitmap. You can obtain the bitmap's bit values by calling the [`get_di_bits`] or `GetBitmapBits`
    /// function.
    ///
    /// If `hgdiobj` is a handle to a logical palette, [`get_object`] retrieves a 2-byte integer that specifies the number of
    /// entries in the palette. The function does not retrieve the `LOGPALETTE` structure defining the palette. To retrieve
    /// information about palette entries, an application can call the `GetPaletteEntries` function.
    ///
    /// If `hgdiobj` is a handle to a font, the `LOGFONT` that is returned is the `LOGFONT` used to create the font. If Windows
    /// had to make some interpolation of the font because the precise `LOGFONT` could not be represented, the interpolation
    /// will not be reflected in the `LOGFONT`. For example, if you ask for a vertical version of a font that doesn't support
    /// vertical painting, the `LOGFONT` indicates the font is vertical, but Windows will paint it horizontally.
    fn get_object(&self, c: i32, pv: &mut Buffer) -> Result<NonZeroI32, ()>;
}

#[derive(Debug, PartialEq)]
pub struct Bitmap<Data: AsMut<[u8]>> {
    /// The width, in pixels, of the bitmap. The width must be greater than zero.
    pub width: NonZeroU32,
    /// The height, in pixels, of the bitmap. The height must be greater than zero.
    pub height: NonZeroU32,
    /// The number of bytes in each scan line. This value must be divisible by 2,
    /// because the system assumes that the bit values of a bitmap form an array that is word aligned.
    pub width_bytes: i32,
    /// The count of color planes.
    pub planes: u16,
    /// The number of bits required to indicate the color of a pixel.
    pub bits_pixel: u16,
    /// A pointer to the location of the bit values for the bitmap.
    pub bits: Data,
}

impl<Data: AsMut<[u8]>> GetObject<Bitmap<Data>> for SelectedHBitmap {
    fn get_object(&self, c: i32, pv: &mut Bitmap<Data>) -> Result<NonZeroI32, ()> {
        let mut proxy = BITMAP {
            bmType: 0,
            bmWidth: pv.width.get().try_into().unwrap(),
            bmHeight: pv.height.get().try_into().unwrap(),
            bmWidthBytes: pv.width_bytes,
            bmPlanes: pv.planes,
            bmBitsPixel: pv.bits_pixel,
            bmBits: pv.bits.as_mut().as_mut_ptr().cast(),
        };
        NonZeroI32::new(unsafe {
            GetObjectA(self.0.0.as_hgdi_obj(), c, Some((&raw mut proxy).cast()))
        })
        .ok_or(())
    }
}

pub trait SelectObject: AsDc {
    fn select_object(&mut self, bitmap: HBitmap) -> Result<SelectedHBitmap, HBitmap>;
}

impl<T: AsDc> SelectObject for T {
    fn select_object(&mut self, bitmap: HBitmap) -> Result<SelectedHBitmap, HBitmap> {
        let succ = unsafe { SelectObject(self.as_dc(), bitmap.0.as_hgdi_obj()) };
        if succ.is_invalid() {
            Err(bitmap)
        } else {
            Ok(SelectedHBitmap(bitmap))
        }
    }
}

pub type BitmapInfo = BITMAPINFO;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DibUsage {
    /// The color table should consist of literal red, green, blue (RGB) values.
    #[default]
    Rgb = DIB_RGB_COLORS.0 as isize,

    /// The color table should consist of an array of 16-bit indexes into the current logical palette.
    Pal = DIB_PAL_COLORS.0 as isize,
}

impl From<DibUsage> for DIB_USAGE {
    #[inline]
    fn from(value: DibUsage) -> Self {
        DIB_USAGE(value as u32)
    }
}

/// The [`bit_blt`] function performs a bit-block transfer of the color data corresponding to a
/// rectangle of pixels from the specified source device context into a destination device context.
///
/// # Parameters
///
/// `[in] hdc`
///
/// A handle to the destination device context.
///
/// `[in] x`
///
/// The x-coordinate, in logical units, of the upper-left corner of the destination rectangle.
///
/// `[in] y`
///
/// The y-coordinate, in logical units, of the upper-left corner of the destination rectangle.
///
/// `[in] cx`
///
/// The width, in logical units, of the source and destination rectangles.
///
/// `[in] cy`
///
/// The height, in logical units, of the source and the destination rectangles.
///
/// `[in] hdc_src`
///
/// A handle to the source device context.
///
/// `[in] x1`
///
/// The x-coordinate, in logical units, of the upper-left corner of the source rectangle.
///
/// `[in] y1`
///
/// The y-coordinate, in logical units, of the upper-left corner of the source rectangle.
///
/// # Return value
///
/// If the function succeeds, the return value is nonzero.
///
/// If the function fails, the return value is zero. To get extended error information, call [`GetLastError`].
///
/// # Remarks
///
/// [`bit_blt`] only does clipping on the destination DC.
///
/// If a rotation or shear transformation is in effect in the source device context, [`bit_blt`] returns an
/// error. If other transformations exist in the source device context (and a matching transformation is not
/// in effect in the destination device context), the rectangle in the destination device context is stretched,
/// compressed, or rotated, as necessary.
///
/// If the color formats of the source and destination device contexts do not match, the [`bit_blt`]
/// function converts the source color format to match the destination format.
///
/// When an enhanced metafile is being recorded, an error occurs if the source device context identifies
/// an enhanced-metafile device context.
///
/// Not all devices support the [`bit_blt`] function. For more information, see the `RC_BITBLT` raster
/// capability entry in the GetDeviceCaps function as well as the following functions: `MaskBlt`, `PlgBlt`,
/// and `StretchBlt`.
///
/// [`bit_blt`] returns an error if the source and destination device contexts represent different devices.
/// To transfer data between DCs for different devices, convert the memory bitmap to a DIB by calling
/// [`get_di_bits`]. To display the DIB to the second device, call `SetDIBits` or `StretchDIBits`.
///
/// ICM: No color management is performed when blits occur.
#[inline]
pub unsafe fn bit_blt<T: AsDc, U: AsDc>(
    hdc: &T,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    hdc_src: &U,
    x1: i32,
    y1: i32,
) -> windows::core::Result<()> {
    unsafe {
        BitBlt(
            hdc.as_dc(),
            x,
            y,
            cx,
            cy,
            Some(hdc_src.as_dc()),
            x1,
            y1,
            SRCCOPY,
        )
    }
}

/// The [`get_di_bits`] function retrieves the bits of the specified compatible bitmap and copies them
/// into a buffer as a DIB using the specified format.
///
/// # Parameters
///
/// `[in] hdc`
///
/// A handle to the device context.
///
/// `[in] hbm`
///
/// A handle to the bitmap. This must be a compatible bitmap (DDB).
///
/// `[in] start`
///
/// The first scan line to retrieve.
///
/// `[in] c_lines`
///
/// The number of scan lines to retrieve.
///
/// `[out] lpv_bits`
///
/// A pointer to a buffer to receive the bitmap data. If this parameter is [`None`], the function passes
/// the dimensions and format of the bitmap to the [`BITMAPINFO`] structure pointed to by the `lpbmi` parameter.
///
/// `[in, out] lpbmi`
///
/// A pointer to a [`BITMAPINFO`] structure that specifies the desired format for the DIB data.
///
/// `[in] usage`
///
/// The format of the `bmiColors` member of the [`BITMAPINFO`] structure.
///
/// # Return
///
/// If the `lpv_bits` parameter is [`Some`] and the function succeeds, the return value is the number of scan lines copied from the bitmap.
///
/// If the `lpv_bits` parameter is [`None`] and [`get_di_bits`] successfully fills the [`BITMAPINFO`] structure, the return value is nonzero.
///
/// If the function fails, the return value is zero.
///
/// # Remarks
///
/// If the requested format for the DIB matches its internal format, the RGB values for the bitmap are copied. If the requested format
/// doesn't match the internal format, a color table is synthesized. The following table describes the color table synthesized for each
/// format.
///
/// | Value  | Meaning |
/// |--------|---------|
/// | 1_BPP  | The color table consists of a black and a white entry. |
/// | 4_BPP  | The color table consists of a mix of colors identical to the standard VGA palette. |
/// | 8_BPP  | The color table consists of a general mix of 256 colors defined by GDI. (Included in these 256 colors are the 20 colors found in the default logical palette.) |
/// | 24_BPP | No color table is returned. |
///
/// If the `lpv_bits` parameter is a valid pointer, the first six members of the [`BITMAPINFOHEADER`] structure must be initialized to specify
/// the size and format of the DIB. The scan lines must be aligned on a `DWORD` except for RLE compressed bitmaps.
///
/// A bottom-up DIB is specified by setting the height to a positive number, while a top-down DIB is specified by setting the height to
/// a negative number. The bitmap color table will be appended to the [`BITMAPINFO`] structure.
///
/// If `lpv_bits` is [`None`], [`get_di_bits`] examines the first member of the first structure pointed to by `lpbi`. This member must specify the size,
/// in bytes, of a `BITMAPCOREHEADER` or a [`BITMAPINFOHEADER`] structure. The function uses the specified size to determine how the remaining
/// members should be initialized.
///
/// If `lpv_bits` is [`None`] and the bit count member of [`BITMAPINFO`] is initialized to zero, [`get_di_bits`] fills in a [`BITMAPINFOHEADER`] structure or
/// `BITMAPCOREHEADER` without the color table. This technique can be used to query bitmap attributes.
///
/// The bitmap identified by the `hbmp` parameter must not be selected into a device context when the application calls this function.
///
/// The origin for a bottom-up DIB is the lower-left corner of the bitmap; the origin for a top-down DIB is the upper-left corner.
///
/// # Safety
///
/// - If the `lpv_bits` parameter is a valid pointer, the first six members of the [`BITMAPINFOHEADER`] structure must be initialized to specify
///   the size and format of the DIB. The scan lines must be aligned on a `DWORD` except for RLE compressed bitmaps.
///
/// - If `lpv_bits` is [`None`], the first member of the first structure pointed to by `lpbi` must specify the size, in bytes, of a `BITMAPCOREHEADER`
///   or a [`BITMAPINFOHEADER`] structure.
///
/// - The bitmap identified by the `hbmp` parameter must not be selected into a device context when the application calls this function.
pub unsafe fn get_di_bits<T: AsDc>(
    hdc: &T,
    hbm: &HBitmap,
    start: u32,
    c_lines: u32,
    lpv_bits: Option<&mut [u8]>,
    lpbmi: &mut BitmapInfo,
    usage: DibUsage,
) -> Result<NonZeroI32, ()> {
    NonZeroI32::new(unsafe {
        GetDIBits(
            hdc.as_dc(),
            hbm.0.as_hbitmap(),
            start,
            c_lines,
            lpv_bits.map(|slice| slice.as_mut_ptr().cast()),
            lpbmi,
            usage.into(),
        )
    })
    .ok_or(())
}

pub type MouseInput = MOUSEINPUT;
pub type KeybdInput = KEYBDINPUT;
pub type HardwareInput = HARDWAREINPUT;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, u32)]
pub enum Input {
    Mouse(MouseInput),
    Keybd(KeybdInput),
    Hardware(HardwareInput),
}

const _: () = {
    assert!(std::mem::size_of::<Input>() == std::mem::size_of::<INPUT>());
    assert!(std::mem::align_of::<Input>() == std::mem::align_of::<INPUT>());
    assert!(std::mem::offset_of!(Input, Mouse.0) == std::mem::offset_of!(INPUT, Anonymous));
};

/// Synthesizes keystrokes, mouse motions, and button clicks.
///
/// # Parameters
///
/// `[in] pInputs`
///
/// Type: `LPINPUT`
///
/// An array of [`INPUT`] structures. Each structure represents an event to be inserted into
/// the keyboard or mouse input stream.
///
/// # Return value
///
/// Type: `UINT`
///
/// The function returns the number of events that it successfully inserted into the keyboard
/// or mouse input stream. If the function returns zero, the input was already blocked by
/// another thread. To get extended error information, call [`GetLastError`].
///
/// This function fails when it is blocked by UIPI. Note that neither [`GetLastError`] nor the
/// return value will indicate the failure was caused by UIPI blocking.
///
/// # Remarks
///
/// This function is subject to UIPI. Applications are permitted to inject input only into
/// applications that are at an equal or lesser integrity level.
///
/// The [`send_input`] function inserts the events in the [`INPUT`] structures serially into the
/// keyboard or mouse input stream. These events are not interspersed with other keyboard or
/// mouse input events inserted either by the user (with the keyboard or mouse) or by calls
/// to `keybd_event`, `mouse_event`, or other calls to [`send_input`].
///
/// This function does not reset the keyboard's current state. Any keys that are already
/// pressed when the function is called might interfere with the events that this function
/// generates. To avoid this problem, check the keyboard's state with the [`GetAsyncKeyState`]
/// function and correct as necessary.
///
/// Because the touch keyboard uses the surrogate macros defined in winnls.h to send input
/// to the system, a listener on the keyboard event hook must decode input originating from
/// the touch keyboard. For more information, see Surrogates and Supplementary Characters.
///
/// An accessibility application can use [`send_input`] to inject keystrokes corresponding to
/// application launch shortcut keys that are handled by the shell. This functionality is
/// not guaranteed to work for other types of applications.
pub unsafe fn send_input(inputs: &[Input]) -> Result<NonZeroU32, ()> {
    NonZeroU32::new(unsafe {
        SendInput(
            std::slice::from_raw_parts(inputs.as_ptr().cast(), inputs.len()),
            std::mem::size_of::<INPUT>() as i32,
        )
    })
    .ok_or(())
}

/// Retrieves the position of the mouse cursor, in screen coordinates.
///
/// # Parameters
///
/// `[out] lpPoint`
///
/// Type: `LPPOINT`
///
/// A pointer to a [`POINT`] structure that receives the screen coordinates of the cursor.
///
/// # Return value
///
/// Type: `BOOL`
///
/// Returns nonzero if successful or zero otherwise. To get extended error information,
/// call [`GetLastError`].
///
/// # Remarks
///
/// The cursor position is always specified in screen coordinates and is not affected by
/// the mapping mode of the window that contains the cursor.
///
/// The calling process must have `WINSTA_READATTRIBUTES` access to the window station.
///
/// The input desktop must be the current desktop when you call [`get_cursor_pos`]. Call
/// `OpenInputDesktop` to determine whether the current desktop is the input desktop.
/// If it is not, call `SetThreadDesktop` with the `HDESK` returned by `OpenInputDesktop`
/// to switch to that desktop.
///
/// # Safety
///
/// - The calling process must have `WINSTA_READATTRIBUTES` access to the window station.
/// - The input desktop must be the current desktop when you call [`get_cursor_pos`].
pub unsafe fn get_cursor_pos() -> WinResult<Point> {
    let mut point = std::mem::MaybeUninit::uninit();
    unsafe { GetCursorPos(point.as_mut_ptr()) }
        // SAFETY: GetCursorPos promises to initialize point on success.
        .map(|()| unsafe { point.assume_init() })
}
