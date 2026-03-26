//! Cross-platform vision (vidi) and action (vici) library
//!
//! ```
//! let hshared = init();
//! let vinput = VInputHandle::init(hshared.href());
//! let uinput = UInputHandle::init(hshared.href());
//! let screen = ScreenHandle::init(hshared.href());
//! ```

#![deny(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    clippy::missing_panics_doc,
    missing_docs
)]
#![deny(
    unsafe_code,
    reason = "unsafe code should be limited to platform-specific implementations"
)]
#![feature(
    never_type,
    associated_type_defaults,
    new_range_api,
    derive_const,
    const_default,
    const_trait_impl,
    const_clone,
    const_cmp
)]

macro_rules! platform_impl {
    (
        $(#[$m:meta])*
        $vis:vis mod $module:ident = match {
            $(cfg($($cfg:tt)+) => $path:literal,)*
            _ => $default_path:literal $(,)?
        };
    ) => {
        $(#[cfg_attr($($cfg)+, path = $path)])*
        #[cfg_attr(not(any($($($cfg)+),*)), path = $default_path)]
        $(#[$m])*
        $vis mod $module;
    };
}

platform_impl! {
    #[allow(unsafe_code)]
    pub mod platform = match {
        cfg(windows) => "impls/windows.rs",
        _ => "impls/template.rs",
    };
}

pub mod color;
pub mod irect;
pub mod ivec2;

pub use color::*;
pub use irect::*;
pub use ivec2::*;
pub use platform::*;

// "hints" get empty default implementations.
// By default, all error types are `!` to make it easier for infallible implementations.

/// The current state of a key or button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KeyState {
    /// Depressed
    #[default]
    Up,
    /// Held
    Down,
}

impl KeyState {
    /// The key is not held
    pub const fn is_up(self) -> bool {
        matches!(self, Self::Up)
    }

    /// The key is held
    pub const fn is_down(self) -> bool {
        matches!(self, Self::Down)
    }
}

/// Virtual keys used by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    /// The key that toggles the front vent
    FrontVent = platform::VK_W as isize,
    /// The key that toggles the left door
    LeftDoor = platform::VK_A as isize,
    /// The key that toggles the camera monitor
    CameraToggle = platform::VK_S as isize,
    /// The key that toggles the right door
    RightDoor = platform::VK_D as isize,
    /// The key that toggles the right vent
    RightVent = platform::VK_F as isize,
    /// The key that catches Old Man Consequences fish
    CatchFish = platform::VK_C as isize,
    /// The key that closes ads
    CloseAd = platform::VK_ENTER as isize,
    /// The key that toggles the desk fan
    DeskFan = platform::VK_SPACE as isize,
    /// The 1 key
    One = platform::VK_1 as isize,
    /// The 2 key
    Two = platform::VK_2 as isize,
    /// The 3 key
    Three = platform::VK_3 as isize,
    /// The 4 key
    Four = platform::VK_4 as isize,
    /// The 5 key
    Five = platform::VK_5 as isize,
    /// The 6 key
    Six = platform::VK_6 as isize,
    /// The X key (I forget what this does)
    X = platform::VK_X as isize,
    /// The key that activates the flashlight while held
    Flashlight = platform::VK_Z as isize,
    /// The key that exits the game
    Exit = platform::VK_ESC as isize,
}

/// Mouse movement, left button, or both
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseEvent {
    /// Move the mouse to the absolute position.
    Goto(IVec2),

    /// Set the m1 (left mouse) button to this state.
    Btn(KeyState),

    /// Move the mouse to the absolute position, then set m1 (left mouse)
    /// button to this state.
    ///
    /// Platforms that have mouse movement and button events separately should
    /// perform each separately.
    GotoAndBtn(IVec2, KeyState),
}

impl MouseEvent {
    /// Split a [`MouseEvent`] into a position and key state.
    ///
    /// At least one of these is guaranteed to be [`Some`].
    #[inline]
    pub const fn unzip(self) -> (Option<IVec2>, Option<KeyState>) {
        (
            match self {
                MouseEvent::Goto(pt) | MouseEvent::GotoAndBtn(pt, _) => Some(pt),
                _ => None,
            },
            match self {
                MouseEvent::Btn(s) | MouseEvent::GotoAndBtn(_, s) => Some(s),
                _ => None,
            },
        )
    }
}

/// Shared handle
pub trait SHandle: Sized {
    /// Error returned by [`Self::init`]
    type InitError: std::error::Error = !;

    /// Initialize the user input handle from the [shared handle](`SharedHandle`).
    ///
    /// Some platforms do not have a shared handle, and may simply initialize
    /// the user input handle directly.
    fn init() -> Result<Self, Self::InitError>;

    /// Depending on the platform, this could be implemented as
    /// - An [`Rc<RefCell<SharedHandle>>`](`std::rc::Rc`)
    ///   (the non-implementation of [`Send`]/[`Sync`] makes
    ///   [`Arc<Mutex<SharedHandle>>`](`std::sync::Arc`) pointless)
    /// - A shared reference with lifetime `'a` to a [`Clone`]able handle
    /// - Or a private unit type
    ///
    /// May not implement [`Copy`]
    type Ref<'a>
    where
        Self: 'a;

    /// Get a [`Self::Ref`] for initializing other subsystem handles.
    ///
    /// Must be infallible on all platforms.
    /// Failure should occur while initializing, not while referencing.
    fn href(&mut self) -> Self::Ref<'_>;

    /// The platform [`UInput`] subsystem
    type UInput<'a>: UInput<SharedHandleRef = Self::Ref<'a>>
    where
        Self: 'a;

    /// Initialize the [`UInput`] subsystem
    fn init_uinput(&mut self) -> Result<Self::UInput<'_>, <Self::UInput<'_> as UInput>::InitError> {
        Self::UInput::init(self.href())
    }

    /// The platform [`VInput`] subsystem
    type VInput<'a>: VInput<SharedHandleRef = Self::Ref<'a>>
    where
        Self: 'a;

    /// Initialize the [`VInput`] subsystem
    fn init_vinput(&mut self) -> Result<Self::VInput<'_>, <Self::VInput<'_> as VInput>::InitError> {
        Self::VInput::init(self.href())
    }

    /// The platform [`Screen`] subsystem
    type Screen<'a>: Screen<SharedHandleRef = Self::Ref<'a>>
    where
        Self: 'a;

    /// Initialize the [`Screen`] subsystem
    fn init_screen(&mut self) -> Result<Self::Screen<'_>, <Self::Screen<'_> as Screen>::InitError> {
        Self::Screen::init(self.href())
    }
}

/// Alias to the platform-specific [`SHandle::init()`]
pub fn init() -> Result<SharedHandle, <SharedHandle as SHandle>::InitError> {
    <SharedHandle as SHandle>::init()
}

/// User input (keyboard/mouse)
///
/// Platforms that separate keyboard and mouse inputs should combine both handles in a tuple.
pub trait UInput: Sized {
    /// The platform [`SharedHandle`] ref type
    type SharedHandleRef;

    /// Error returned by [`Self::init`]
    type InitError: std::error::Error = !;

    /// Initialize the user input subsystem from the shared handle.
    ///
    /// Some platforms do not have a shared handle, and may simply initialize
    /// the user input handle directly.
    fn init(shared_handle: Self::SharedHandleRef) -> Result<Self, Self::InitError>;

    /// Error returned by [`Self::hint_check_events`]
    type HintCheckEventsError: std::error::Error = !;

    /// Hint the key events to be tested and buffered, for platforms where
    /// all key events are checked at once.
    fn hint_check_events(&mut self) -> Result<(), Self::HintCheckEventsError> {
        Ok(())
    }

    /// Error returned by [`Self::get_key_state`]
    type GetKeyStateError: std::error::Error = !;

    /// Check for the current status of a particular virtual key
    fn get_key_state(&mut self, key: VirtualKey) -> Result<KeyState, Self::GetKeyStateError>;

    /// Error returned by [`Self::get_mouse_pos`]
    type GetMousePosError: std::error::Error = !;

    /// Read the current mouse position.
    ///
    /// Note that platforms with buffered inputs may have updated this no
    /// more recently than the last call to [`Self::hint_check_events`].
    fn get_mouse_pos(&mut self) -> Result<IVec2, Self::GetMousePosError>;
}

/// Virtual input (keyboard/mouse)
///
/// Platforms that separate virtual keyboard and mouse inputs should combine them as a tuple.
///
/// Platforms that have a single handle performing multiple duties should create a handle with
/// a reference to the shared handle with an implementation for this.
pub trait VInput: Sized {
    /// The platform [`SharedHandle`] ref type
    type SharedHandleRef;

    /// Error returned by [`Self::init`]
    type InitError: std::error::Error = !;

    /// Initialize the virtual input subsystem from the shared handle.
    ///
    /// Some platforms do not have a shared handle, and may simply initialize
    /// the virtual input handle directly.
    fn init(shared_handle: Self::SharedHandleRef) -> Result<Self, Self::InitError>;

    /// Error that occurs in [`Self::hint_flush_events`].
    type HintFlushEventsError: std::error::Error = !;

    /// Hint for the buffered key events to be sent all at once now,
    /// for platforms that batch input events.
    ///
    /// This should be called on every tick that has produced at least one
    /// input event.
    fn hint_flush_events(&mut self) -> Result<(), Self::HintFlushEventsError> {
        Ok(())
    }

    /// Error that occurs in [`Self::simulate_mouse_event`].
    type SimulateMouseEventError: std::error::Error = !;

    /// Move the mouse to the specified position and set the left mouse
    /// button to the specified state.
    fn simulate_mouse_event(
        &mut self,
        event: MouseEvent,
    ) -> Result<(), Self::SimulateMouseEventError>;

    /// Error that occurs in [`Self::simulate_key_event`].
    type SimulateKeyEventError: std::error::Error = !;

    /// Set each key in `keys` to `state`.
    ///
    /// Platforms that cannot batch key events should push each event separately.
    ///
    /// If `keys` is empty, no events will be produced.
    fn simulate_key_event(
        &mut self,
        keys: &[VirtualKey],
        state: KeyState,
    ) -> Result<(), Self::SimulateKeyEventError>;
}

/// Pixel access
///
/// Platforms that have a single handle performing multiple duties should create a handle with
/// a reference to the shared handle with an implementation for this.
pub trait Screen: Sized {
    /// The platform [`SharedHandle`] ref type
    type SharedHandleRef;

    /// Error returned by [`Self::init`]
    type InitError: std::error::Error = !;

    /// Initialize the screen subsystem from the shared handle.
    ///
    /// Some platforms do not have a shared handle, and may simply initialize
    /// the screen handle directly.
    fn init(shared_handle: Self::SharedHandleRef) -> Result<Self, Self::InitError>;

    /// Error returned by [`Self::width`] and [`Self::height`]
    type GetSizeError: std::error::Error = !;

    /// Width of the screen in pixels
    fn width(&mut self) -> Result<i32, Self::GetSizeError>;

    /// Height of the screen in pixels
    fn height(&mut self) -> Result<i32, Self::GetSizeError>;

    /// Error returned by [`Self::hint_refresh_screencap`]
    type HintRefreshScreencapError: std::error::Error = !;

    /// Hint the screenshot to be updated.
    ///
    /// Note that not all platforms are guaranteed to support updating
    /// the entire screencap at once, and some may do nothing here.
    fn hint_refresh_screencap(&mut self) -> Result<(), Self::HintRefreshScreencapError> {
        Ok(())
    }

    /// Error returned by [`Self::get_pixel`]
    type GetPixelError: std::error::Error = !;

    /// Read the color of the pixel at `pt`
    ///
    /// `self` is mutable in case the platform has to buffer the pixels
    /// on-demand.
    fn get_pixel(&mut self, pt: IVec2) -> Result<ColorRGB, Self::GetPixelError>;

    /// Error returned by [`Self::get_region`] (default implementation calls [`Self::get_pixel_rgb`])
    type GetRegionError: std::error::Error = Self::GetPixelError;

    /// Copies all the pixels in a region of the screen to `buffer`.
    /// By default, this calls [`Self::get_pixel_rgb`] for each pixel.
    ///
    /// Returns the number of elements copied to `buffer`.
    ///
    /// # Panics
    ///
    /// This method will panic if `buffer` has fewer elements than `rgn`, or if
    /// the quantity of pixels cannot fit into a [`usize`].
    fn get_region(
        &mut self,
        rgn: IRect,
        buffer: &mut [ColorRGB],
    ) -> Result<usize, Self::GetPixelError> {
        let rgn = rgn.into_iter();
        let area = match rgn.size_hint() {
            (n, Some(m)) if n == m => n,
            _ => panic!("region size exceeds usize::MAX"),
        };
        assert!(buffer.len() >= area);
        for (pt, px) in rgn.zip(buffer) {
            *px = self.get_pixel(pt)?;
        }
        Ok(area)
    }
}
