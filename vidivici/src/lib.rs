//! Cross-platform vision (vidi) and action (vici) library

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
        $vis:vis mod $module:ident = match cfg {
            $(#[$($cfg:tt)+] => $path:literal,)+
            _ => $default_path:literal $(,)?
        };
    ) => {
        $(#[cfg_attr($($cfg)+, path = $path)])+
        #[cfg_attr(not(any($($($cfg)+),*)), path = $default_path)]
        $(#[$m])*
        $vis mod $module;
    };
}

platform_impl! {
    #[allow(unsafe_code)]
    pub mod platform = match cfg {
        #[windows] => "impls/windows.rs",
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
    /// Inactive
    #[default]
    Up,
    /// Up -> Down
    Press,
    /// Held
    Down,
    /// Down -> Up
    Release,
}

/// Virtual keys used by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    /// The key that toggles the front vent
    FrontVent = VK_W as isize,
    /// The key that toggles the left door
    LeftDoor = VK_A as isize,
    /// The key that toggles the camera monitor
    CameraToggle = VK_S as isize,
    /// The key that toggles the right door
    RightDoor = VK_D as isize,
    /// The key that toggles the right vent
    RightVent = VK_F as isize,
    /// The key that catches Old Man Consequences fish
    CatchFish = VK_C as isize,
    /// The key that closes ads
    CloseAd = VK_ENTER as isize,
    /// The key that toggles the desk fan
    DeskFan = VK_SPACE as isize,
    /// The 1 key
    One = VK_1 as isize,
    /// The 2 key
    Two = VK_2 as isize,
    /// The 3 key
    Three = VK_3 as isize,
    /// The 4 key
    Four = VK_4 as isize,
    /// The 5 key
    Five = VK_5 as isize,
    /// The 6 key
    Six = VK_6 as isize,
    /// The X key (I forget what this does)
    X = VK_X as isize,
    /// The key that activates the flashlight while held
    Flashlight = VK_Z as isize,
    /// The key that exits the game
    Exit = VK_ESC as isize,
}

/// User input (keyboard/mouse)
///
/// Platforms that separate keyboard and mouse inputs should combine both handles in a tuple.
pub trait UInput {
    /// Error returned by [`Self::hint_check_events`]
    type HintCheckEventsError: std::error::Error = !;

    /// Error returned by [`Self::get_key_state`]
    type GetKeyStateError: std::error::Error = !;

    /// Error returned by [`Self::get_mouse_pos`]
    type GetMousePosError: std::error::Error = !;

    /// Hint the key events to be tested and buffered, for platforms where
    /// all key events are checked at once.
    fn hint_check_events(&mut self) -> Result<(), Self::HintCheckEventsError> {
        Ok(())
    }

    /// Check for the current status of a particular virtual key
    fn get_key_state(&mut self) -> Result<KeyState, Self::GetKeyStateError>;

    /// Read the current mouse position.
    ///
    /// Note that platforms with buffered inputs may have updated this no
    /// more recently than the last call to [`Self::hint_check_events`].
    fn get_mouse_pos(&mut self) -> Result<IVec2, Self::GetMousePosError>;
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

/// Virtual input (keyboard/mouse)
///
/// Platforms that separate virtual keyboard and mouse inputs should combine them as a tuple.
///
/// Platforms that have a single handle performing multiple duties should create a handle with
/// a reference to the shared handle with an implementation for this.
pub trait VInput {
    /// Error that occurs in [`Self::hint_flush_events`].
    type HintFlushEventsError: std::error::Error = !;

    /// Error that occurs in [`Self::simulate_mouse_event`].
    type SimulateMouseEventError: std::error::Error = !;

    /// Error that occurs in [`Self::simulate_key_event`].
    type SimulateKeyEventError: std::error::Error = !;

    /// Hint for the buffered key events to be sent all at once now,
    /// for platforms that batch input events.
    ///
    /// This should be called on every tick that has produced at least one
    /// input event.
    fn hint_flush_events(&mut self) -> Result<(), Self::HintFlushEventsError> {
        Ok(())
    }

    /// Move the mouse to the specified position and set the left mouse
    /// button to the specified state.
    fn simulate_mouse_event(
        &mut self,
        event: MouseEvent,
    ) -> Result<(), Self::SimulateMouseEventError>;

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
pub trait Screen {
    /// Error returned by [`Self::hint_refresh_screencap`]
    type HintRefreshScreencapError: std::error::Error = !;

    /// Error returned by [`Self::get_pixel`]
    type GetPixelError: std::error::Error = !;

    /// Error returned by [`Self::get_region`] (default implementation calls [`Self::get_pixel_rgb`])
    type GetRegionError: std::error::Error = Self::GetPixelError;

    /// Hint the screenshot to be updated.
    ///
    /// Note that not all platforms are guaranteed to support updating
    /// the entire screencap at once, and some may do nothing here.
    fn hint_refresh_screencap(&mut self) -> Result<(), Self::HintRefreshScreencapError> {
        Ok(())
    }

    /// Read the color of the pixel at `pt`
    ///
    /// `self` is mutable in case the platform has to buffer the pixels
    /// on-demand.
    fn get_pixel(&mut self, pt: IVec2) -> Result<ColorRGB, Self::GetPixelError>;

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
