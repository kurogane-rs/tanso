#![doc = include_str!("../README.md")]

pub mod args;
mod array_buffer;
pub mod rc;
pub mod string;
pub mod window_info;

#[cfg(feature = "wrapper")]
pub mod wrapper;

#[cfg(target_os = "macos")]
pub mod application_mac;

#[cfg(target_os = "macos")]
pub mod sandbox;

#[cfg(feature = "resources")]
pub mod resources;

#[rustfmt::skip]
mod bindings;
pub use array_buffer::v8_value_create_array_buffer_from_bytes;
pub use bindings::*;

pub use rc::Rc as _;

pub use tanso_sys as sys;

/// Held around every call to CEF's ID mappers. They share tables CEF fills on
/// first use and two threads calling them at once can crash (seen on macOS).
/// The lock guards no Rust data; a poisoned lock is taken as it is.
static ID_MAPPERS: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

/// Loads libcef once for the tests. Loading sets the API version, which
/// every other call needs first.
#[cfg(test)]
fn test_init_cef() {
    use std::sync::Once;

    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let cef = sys::find_cef_dir().expect("CEF not found");
        unsafe { sys::load_libcef(&cef.libcef()) }.expect("cannot load libcef");
    });
}
