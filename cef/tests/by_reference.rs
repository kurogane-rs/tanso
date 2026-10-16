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
            _browser: Option<&mut Browser>,
            _window_info: Option<&mut WindowInfo>,
            client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _use_default_window: Option<&mut i32>,
        ) {
            if let (Some(client), Some(replacement)) = (client, &self.replacement) {
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
    // SAFETY: every other argument may be null
    unsafe {
        callback(
            raw,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut passed,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
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
