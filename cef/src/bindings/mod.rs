#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod x86_64_unknown_linux_gnu;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub use x86_64_unknown_linux_gnu::*;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod aarch64_unknown_linux_gnu;
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub use aarch64_unknown_linux_gnu::*;

#[cfg(all(target_os = "linux", target_arch = "arm"))]
mod arm_unknown_linux_gnueabi;
#[cfg(all(target_os = "linux", target_arch = "arm"))]
pub use arm_unknown_linux_gnueabi::*;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
mod x86_64_pc_windows_msvc;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub use x86_64_pc_windows_msvc::*;

#[cfg(all(target_os = "windows", target_arch = "x86"))]
mod i686_pc_windows_msvc;
#[cfg(all(target_os = "windows", target_arch = "x86"))]
pub use i686_pc_windows_msvc::*;

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
mod aarch64_pc_windows_msvc;
#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
pub use aarch64_pc_windows_msvc::*;

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod x86_64_apple_darwin;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub use x86_64_apple_darwin::*;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod aarch64_apple_darwin;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub use aarch64_apple_darwin::*;

#[cfg(test)]
mod test {
    use crate::*;

    wrap_life_span_handler! {
        struct TestLifeSpanHandler {
            calls: std::rc::Rc<std::cell::Cell<u32>>,
        }

        impl LifeSpanHandler {
            fn on_before_popup(
                &self,
                _browser: &mut Browser,
                _frame: &mut Frame,
                _popup_id: ::std::os::raw::c_int,
                _target_url: Option<&CefString>,
                _target_frame_name: Option<&CefString>,
                _target_disposition: WindowOpenDisposition,
                _user_gesture: ::std::os::raw::c_int,
                _popup_features: &PopupFeatures,
                _window_info: &mut WindowInfo,
                _client: &mut Option<Client>,
                _settings: &mut BrowserSettings,
                _extra_info: &mut Option<DictionaryValue>,
                _no_javascript_access: &mut ::std::os::raw::c_int,
            ) -> ::std::os::raw::c_int {
                self.calls.set(self.calls.get() + 1);
                1
            }
        }
    }

    #[test]
    fn a_null_required_argument_answers_the_default_without_the_callback() {
        use std::ptr::{null, null_mut};

        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let handler = TestLifeSpanHandler::new(calls.clone());
        let raw = ImplLifeSpanHandler::get_raw(&handler);
        // SAFETY: `raw` is the live C struct `handler` holds. The call passes
        // null for the browser, the frame and the other parameters CEF
        // requires, which CEF's own glue never forwards
        let answer = unsafe {
            let on_before_popup = (*raw).on_before_popup.expect("on_before_popup is set");
            on_before_popup(
                raw,
                null_mut(),
                null_mut(),
                1,
                null(),
                null(),
                sys::cef_window_open_disposition_t::CEF_WOD_CURRENT_TAB,
                0,
                null(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };
        assert_eq!(answer, 0);
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn a_transition_holds_its_source_and_qualifiers() {
        let forward = TransitionType::LINK.get_raw() | TransitionType::FORWARD_BACK_FLAG.get_raw();
        let transition = TransitionType::from(sys::cef_transition_type_t(forward));
        assert_eq!(transition.get_raw() & TransitionType::SOURCE_MASK.get_raw(), 0);
        assert_ne!(transition.get_raw() & TransitionType::FORWARD_BACK_FLAG.get_raw(), 0);
    }

    #[test]
    fn an_id_mapper_knows_this_build_s_names() {
        test_init_cef();

        assert!(id_for_command_id_name(c"IDC_BACK").is_some());
        assert_eq!(id_for_command_id_name(c"IDC_NOT_A_COMMAND"), None);
    }

    #[test]
    fn the_id_mappers_take_calls_from_several_threads() {
        test_init_cef();

        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for _ in 0..100 {
                        assert!(id_for_command_id_name(c"IDC_BACK").is_some());
                        let _ = id_for_pack_resource_name(c"IDR_BROKENCANVAS");
                        let _ = id_for_pack_string_name(c"IDS_WEB_FONT_FAMILY");
                    }
                });
            }
        });
    }
}
