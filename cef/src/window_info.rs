use crate::{sys::cef_window_handle_t, *};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::*;

impl WindowInfo {
    /// Create the browser as a child window.
    pub fn set_as_child(self, parent: cef_window_handle_t, bounds: &Rect) -> Self {
        Self {
            #[cfg(target_os = "windows")]
            style: WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_TABSTOP | WS_VISIBLE,
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            parent_window: parent,
            #[cfg(target_os = "macos")]
            parent_view: parent,
            bounds: *bounds,
            #[cfg(target_os = "macos")]
            hidden: 0,
            ..self
        }
    }

    /// Create the browser as a popup window.
    #[cfg(target_os = "windows")]
    pub fn set_as_popup(self, parent: cef_window_handle_t, title: &str) -> Self {
        Self {
            window_name: CefString::from(title),
            parent_window: parent,
            style: WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE,
            bounds: Rect {
                x: CW_USEDEFAULT,
                y: CW_USEDEFAULT,
                width: CW_USEDEFAULT,
                height: CW_USEDEFAULT,
            },
            ..self
        }
    }

    /// Create the browser using windowless (off-screen) rendering. No window
    /// is created for the browser; its frames reach the client's
    /// `RenderHandler`. `parent` identifies the monitor and parents dialogs and
    /// context menus. Without one CEF takes the main screen, and what needs a
    /// parent window may not work; `None` keeps the parent `self` holds, none
    /// in a default `WindowInfo`. Windowless browsers are always of the Alloy
    /// style. `shared_texture_enabled` and `external_begin_frame_enabled`
    /// choose GPU frames and frames the client drives.
    pub fn set_as_windowless(self, parent: Option<cef_window_handle_t>) -> Self {
        Self {
            windowless_rendering_enabled: 1,
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            parent_window: parent.unwrap_or(self.parent_window),
            #[cfg(target_os = "macos")]
            parent_view: parent.unwrap_or(self.parent_view),
            runtime_style: RuntimeStyle::ALLOY,
            ..self
        }
    }
}
