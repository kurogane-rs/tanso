//! Frames of off-screen (windowless) rendering.
//!
//! CEF lends a frame for one paint callback. `RenderHandler::on_paint`'s
//! pixels are a slice borrowed for the call, so they cannot be kept:
//!
//! ```compile_fail
//! # use std::cell::RefCell;
//! # use tanso::*;
//! wrap_render_handler! {
//!     struct Keeper {
//!         kept: RefCell<Option<&'static [u8]>>,
//!     }
//!
//!     impl RenderHandler {
//!         fn view_rect(&self, _browser: &mut Browser, rect: &mut Rect) {
//!             *rect = Rect { x: 0, y: 0, width: 1, height: 1 };
//!         }
//!
//!         fn on_paint(
//!             &self,
//!             _browser: &mut Browser,
//!             _type_: PaintElementType,
//!             _dirty_rects: &[Rect],
//!             buffer: &[u8],
//!             _width: i32,
//!             _height: i32,
//!         ) {
//!             *self.kept.borrow_mut() = Some(buffer);
//!         }
//!     }
//! }
//! ```
//!
//! A copy can be:
//!
//! ```
//! # use std::cell::RefCell;
//! # use tanso::*;
//! wrap_render_handler! {
//!     struct Copier {
//!         copied: RefCell<Vec<u8>>,
//!     }
//!
//!     impl RenderHandler {
//!         fn view_rect(&self, _browser: &mut Browser, rect: &mut Rect) {
//!             *rect = Rect { x: 0, y: 0, width: 1, height: 1 };
//!         }
//!
//!         fn on_paint(
//!             &self,
//!             _browser: &mut Browser,
//!             _type_: PaintElementType,
//!             _dirty_rects: &[Rect],
//!             buffer: &[u8],
//!             _width: i32,
//!             _height: i32,
//!         ) {
//!             self.copied.borrow_mut().clear();
//!             self.copied.borrow_mut().extend_from_slice(buffer);
//!         }
//!     }
//! }
//! ```
//!
//! An accelerated frame's GPU handles are lent the same way: CEF returns
//! them to its pool when `on_accelerated_paint` returns. The accessors below
//! borrow them from the [`AcceleratedPaintInfo`] for as long as it is
//! borrowed; their callers vouch that it is the one CEF passed to the
//! running callback.

use crate::AcceleratedPaintInfo;

#[cfg(target_os = "windows")]
impl AcceleratedPaintInfo {
    /// The frame's shared D3D11 texture, an NT handle without a keyed mutex.
    /// None when CEF passed none.
    ///
    /// # Safety
    ///
    /// `self` is the info CEF passed to the running `on_accelerated_paint`.
    /// CEF closes the handle when that callback returns.
    pub unsafe fn shared_texture(&self) -> Option<std::os::windows::io::BorrowedHandle<'_>> {
        let handle = self.shared_texture_handle;
        // SAFETY: the caller's promise; the handle is open for the callback,
        // which outlasts the borrow of `self`
        (!handle.is_null())
            .then(|| unsafe { std::os::windows::io::BorrowedHandle::borrow_raw(handle) })
    }
}

#[cfg(target_os = "macos")]
impl AcceleratedPaintInfo {
    /// The frame's IOSurface. None when CEF passed none.
    ///
    /// # Safety
    ///
    /// `self` is the info CEF passed to the running `on_accelerated_paint`.
    /// CEF hands the surface back to its pool when that callback returns.
    pub unsafe fn io_surface(&self) -> Option<BorrowedIoSurface<'_>> {
        std::ptr::NonNull::new(self.shared_texture_io_surface).map(|surface| BorrowedIoSurface {
            surface,
            _lent: std::marker::PhantomData,
        })
    }
}

/// An IOSurface CEF lends for one paint callback.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug)]
pub struct BorrowedIoSurface<'a> {
    surface: std::ptr::NonNull<std::os::raw::c_void>,
    _lent: std::marker::PhantomData<&'a AcceleratedPaintInfo>,
}

#[cfg(target_os = "macos")]
impl BorrowedIoSurface<'_> {
    /// The `IOSurfaceRef`, not retained.
    pub fn as_ptr(&self) -> *mut std::os::raw::c_void {
        self.surface.as_ptr()
    }
}

#[cfg(target_os = "linux")]
impl AcceleratedPaintInfo {
    /// The planes of the frame's dmabuf, as many as CEF filled.
    pub fn planes(&self) -> &[crate::AcceleratedPaintNativePixmapPlaneInfo] {
        let count = usize::try_from(self.plane_count)
            .unwrap_or_default()
            .min(self.planes.len());
        &self.planes[..count]
    }
}

#[cfg(target_os = "linux")]
impl crate::AcceleratedPaintNativePixmapPlaneInfo {
    /// The plane's dmabuf file descriptor. None when CEF passed none.
    ///
    /// # Safety
    ///
    /// The plane belongs to the info CEF passed to the running
    /// `on_accelerated_paint`. CEF closes the descriptor when that callback
    /// returns; duplicate it to keep it.
    pub unsafe fn fd(&self) -> Option<std::os::fd::BorrowedFd<'_>> {
        // SAFETY: the caller's promise; the descriptor is open for the
        // callback, which outlasts the borrow of `self`
        (self.fd >= 0).then(|| unsafe { std::os::fd::BorrowedFd::borrow_raw(self.fd) })
    }
}
