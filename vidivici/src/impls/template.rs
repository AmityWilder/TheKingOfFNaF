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
/// - May not implement [`Send`] or [`Sync`] because some impls hold mutable pointers.
/// - May not implement [`Clone`] because some impls may risk double-free if duplicated.
/// - Must implement [`Drop`] because most imples with a shared handle will need be cleaned.
#[derive(Debug)]
pub struct SharedHandle(PhantomData<*mut ()>);

impl Drop for SharedHandle {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

impl SHandle for SharedHandle {
    type InitError = !;

    fn init() -> Result<Self, Self::InitError> {
        Ok(SharedHandle(PhantomData))
    }

    type Ref = SharedHandleRef;

    fn href(&mut self) -> Self::Ref {
        SharedHandleRef(())
    }

    type UInput = UInputHandle;
    type VInput = VInputHandle;
    type Screen = ScreenHandle;
}

/// Depending on the platform, this could be implemented as
/// - An [`Rc<RefCell<SharedHandle>>`](`std::rc::Rc`)
///   (the non-implementation of [`Send`]/[`Sync`] makes
///   [`Arc<Mutex<SharedHandle>>`](`std::sync::Arc`) pointless)
/// - A [`ManuallyDrop`](`std::mem::ManuallyDrop`) clone of a
///   duplicable handle (only the original should drop)
/// - Or a private unit type
///
/// May not implement [`Copy`]
#[derive(Debug)]
pub struct SharedHandleRef(());

/// User input (keyboard/mouse) handle.
///
/// May not implement [`Send`], [`Sync`], or [`Clone`]. Must implement [`Drop`].
#[derive(Debug)]
pub struct UInputHandle(PhantomData<*mut ()>, SharedHandleRef);

impl Drop for UInputHandle {
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

impl UInput for UInputHandle {
    type SharedHandleRef = SharedHandleRef;

    fn init(shared_handle: Self::SharedHandleRef) -> Result<Self, Self::InitError> {
        Ok(Self(PhantomData, shared_handle))
    }

    fn get_key_state(&mut self, _key: VirtualKey) -> Result<KeyState, Self::GetKeyStateError> {
        unimplemented!()
    }

    fn get_mouse_pos(&mut self) -> Result<IVec2, Self::GetMousePosError> {
        unimplemented!()
    }
}

/// Virtual input (keyboard/mouse) handle.
///
/// May not implement [`Send`], [`Sync`], or [`Clone`]. Must implement [`Drop`].
#[derive(Debug)]
pub struct VInputHandle(PhantomData<*mut ()>, SharedHandleRef);

impl Drop for VInputHandle {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

impl VInput for VInputHandle {
    type SharedHandleRef = SharedHandleRef;

    fn init(shared_handle: Self::SharedHandleRef) -> Result<Self, Self::InitError> {
        Ok(Self(PhantomData, shared_handle))
    }

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

/// May not implement [`Send`], [`Sync`], or [`Clone`]. Must implement [`Drop`].
#[derive(Debug)]
pub struct ScreenHandle(PhantomData<*mut ()>, SharedHandleRef);

impl Drop for ScreenHandle {
    fn drop(&mut self) {
        // If a platform has fallible cleanup: that's a shame.
    }
}

impl Screen for ScreenHandle {
    type SharedHandleRef = SharedHandleRef;

    fn init(shared_handle: Self::SharedHandleRef) -> Result<Self, Self::InitError> {
        Ok(Self(PhantomData, shared_handle))
    }

    fn width(&mut self) -> Result<i32, Self::GetPixelError> {
        unimplemented!()
    }

    fn height(&mut self) -> Result<i32, Self::GetPixelError> {
        unimplemented!()
    }

    fn get_pixel(&mut self, _pt: IVec2) -> Result<ColorRgb, Self::GetPixelError> {
        unimplemented!()
    }
}
