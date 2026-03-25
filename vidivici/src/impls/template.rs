//! An empty template for the most restrictive platform implementation.
//!
//! Functions that can fail in any implementation must return a [`Result`] in every implementation.
//! If it can't fail in a specific implementation, replace the error type with `!` and wrap the
//! infallible return in [`Ok`].

#![allow(
    clippy::missing_const_for_fn,
    reason = "template will lack a lot of non-const code that is present in more restrictive impls"
)]

use super::*;
use std::marker::PhantomData;

/// Some impls may require all subsystem handles to belong to a singular shared handle.
/// In that case, they will all reference this one through a [`SharedHandleRef`].
///
/// - Cannot implement [`Send`] or [`Sync`] because some impls hold mutable pointers.
/// - Cannot implement [`Clone`] because some impls may risk double-free if duplicated.
/// - Must implement [`Drop`] because most imples with a shared handle will need be cleaned.
#[derive(Debug)]
pub struct SharedHandle(PhantomData<*mut ()>);

impl Drop for SharedHandle {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

/// Initialize the shared handle
pub fn init() -> Result<SharedHandle, !> {
    Ok(SharedHandle(PhantomData))
}

/// Depending on the platform, this could be implemented as
/// - An [`Rc<RefCell<SharedHandle>>`](`std::rc::Rc`)
///   (the non-implementation of [`Send`]/[`Sync`] makes
///   [`Arc<Mutex<SharedHandle>>`](`std::sync::Arc`) pointless)
/// - A shared reference with lifetime `'a` to a [`Clone`]able handle
/// - Or a private unit type
///
/// Cannot implement [`Copy`]
#[derive(Debug)]
pub struct SharedHandleRef<'a>(PhantomData<&'a ()>);

/// User input (keyboard/mouse) handle.
///
/// Cannot implement [`Send`], [`Sync`], or [`Clone`]. Must implement [`Drop`].
#[derive(Debug)]
pub struct UInputHandle<'a>(PhantomData<*mut ()>, SharedHandleRef<'a>);

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

impl<'a> UInputHandle<'a> {
    /// Initialize the user input handle from the [shared handle](`SharedHandle`).
    ///
    /// Some platforms do not have a shared handle, and may simply initialize
    /// the user input handle directly.
    pub fn init(shared_handle: SharedHandleRef<'a>) -> Result<Self, !> {
        Ok(Self(PhantomData, shared_handle))
    }
}

impl<'a> super::UInput for VInputHandle<'a> {
    fn get_key_state(&mut self) -> Result<KeyState, Self::GetKeyStateError> {
        unimplemented!()
    }

    fn get_mouse_pos(&mut self) -> Result<IVec2, Self::GetMousePosError> {
        unimplemented!()
    }
}

/// Virtual input (keyboard/mouse) handle.
///
/// Cannot implement [`Send`], [`Sync`], or [`Clone`]. Must implement [`Drop`].
#[derive(Debug)]
pub struct VInputHandle<'a>(PhantomData<*mut ()>, SharedHandleRef<'a>);

impl Drop for VInputHandle<'_> {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

impl<'a> VInputHandle<'a> {
    /// Initialize the virtual input handle from the [shared handle](`SharedHandle`).
    ///
    /// Some platforms do not have a shared handle, and may simply initialize
    /// the virtual input handle directly.
    pub fn init(shared_handle: SharedHandleRef<'a>) -> Result<Self, !> {
        Ok(Self(PhantomData, shared_handle))
    }
}

impl<'a> super::VInput for VInputHandle<'a> {
    fn simulate_mouse_event(
        &mut self,
        _event: MouseEvent,
    ) -> Result<(), Self::SimulateMouseEventError> {
        unimplemented!()
    }

    fn simulate_key_event(
        &mut self,
        _keys: &[VirtualKey],
        _state: KeyState,
    ) -> Result<(), Self::SimulateKeyEventError> {
        unimplemented!()
    }
}

/// Cannot implement [`Send`], [`Sync`], or [`Clone`]. Must implement [`Drop`].
#[derive(Debug)]
pub struct ScreenHandle<'a>(PhantomData<*mut ()>, SharedHandleRef<'a>);

impl Drop for ScreenHandle<'_> {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

impl<'a> ScreenHandle<'a> {
    /// Initialize the screen handle from the [shared handle](`SharedHandle`).
    ///
    /// Some platforms do not have a shared handle, and may simply initialize
    /// the screen handle directly.
    pub fn init(shared_handle: SharedHandleRef<'a>) -> Self {
        Self(PhantomData, shared_handle)
    }
}

impl<'a> super::Screen for ScreenHandle<'a> {
    fn get_pixel(&mut self, _pt: IVec2) -> Result<ColorRGB, Self::GetPixelError> {
        unimplemented!()
    }
}
