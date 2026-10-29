//! A by-reference parameter follows CEF's reference transfer.
//!
//! CEF passes the value with a reference for the call and takes one back
//! only when the handler changed the pointer. These tests call a handler the
//! way CEF does.

use std::ptr;

use tanso::rc::{ConvertReturnValue, Rc};
use tanso::*;

wrap_client! {
    struct Plain;

    impl Client {}
}

wrap_life_span_handler! {
    struct Handler {
        replacement: Option<Option<Client>>,
    }

    impl LifeSpanHandler {
        fn on_before_dev_tools_popup(
            &self,
            _browser: &mut Browser,
            _window_info: &mut WindowInfo,
            client: &mut Option<Client>,
            _settings: &mut BrowserSettings,
            _extra_info: &mut Option<DictionaryValue>,
            _use_default_window: &mut i32,
        ) {
            if let Some(replacement) = &self.replacement {
                *client = replacement.clone();
            }
        }
    }
}

/// Calls `handler` as CEF does, passing `client` with a reference for the
/// call, and returns the pointer CEF finds afterwards.
fn call(handler: &LifeSpanHandler, client: &Client) -> *mut sys::_cef_client_t {
    // SAFETY: the reference CEF adds for the call
    unsafe { client.add_ref() };
    let mut passed = client.get_raw();
    let raw = handler.get_raw();
    // SAFETY: the handler's own function table, filled when it was wrapped
    let callback = unsafe { (*raw).on_before_dev_tools_popup }.expect("the handler is wrapped");
    // The other arguments CEF never passes null. The browser stands in with
    // no reference counting of its own
    let mut browser: sys::_cef_browser_t = unsafe { std::mem::zeroed() };
    browser.base.size = std::mem::size_of::<sys::_cef_browser_t>();
    let mut window_info: sys::_cef_window_info_t = unsafe { std::mem::zeroed() };
    window_info.size = std::mem::size_of::<sys::_cef_window_info_t>();
    let mut settings: sys::_cef_browser_settings_t = unsafe { std::mem::zeroed() };
    settings.size = std::mem::size_of::<sys::_cef_browser_settings_t>();
    let mut extra_info = ptr::null_mut();
    let mut use_default_window = 0;
    // SAFETY: every pointer is valid for the call
    unsafe {
        callback(
            raw,
            &mut browser,
            &mut window_info,
            &mut passed,
            &mut settings,
            &mut extra_info,
            &mut use_default_window,
        )
    };
    passed
}

#[test]
fn an_unchanged_value_returns_the_reference_cef_added() {
    let opener = Plain::new();
    let handler = Handler::new(None);

    assert_eq!(call(&handler, &opener), opener.get_raw());
    assert!(opener.has_one_ref(), "the call's reference is released");
}

#[test]
fn a_replaced_value_hands_its_reference_to_cef() {
    let opener = Plain::new();
    let replacement = Plain::new();
    let handler = Handler::new(Some(Some(replacement.clone())));

    let passed = call(&handler, &opener);
    assert_eq!(passed, replacement.get_raw());
    assert!(
        opener.has_one_ref(),
        "the call's reference to the old value is released"
    );

    // CEF adopts the reference that came with the new pointer
    let adopted: Client = passed.wrap_result();
    drop(adopted);
    drop(handler);
    assert!(
        replacement.has_one_ref(),
        "the new value carried exactly one reference"
    );
}

#[test]
fn a_cleared_value_reaches_cef_as_null() {
    let opener = Plain::new();
    let handler = Handler::new(Some(None));

    assert!(call(&handler, &opener).is_null());
    assert!(opener.has_one_ref(), "the call's reference is released");
}
