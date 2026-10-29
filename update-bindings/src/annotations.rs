//! CEF's metadata about its own API, from the C++ headers.
//!
//! CEF's translator writes the C API from the C++ headers and reads a
//! `/*--cef(...)--*/` comment above each declaration. A parameter of a kind
//! CEF's generated glue checks, and which the comment does not name with
//! `optional_param`, is never null. The glue returns early on a null one
//! whichever side makes the call (`tools/make_cpptoc_impl.py` and
//! `make_ctocpp_impl.py` in CEF's source). A client class's method that the
//! C++ header declares pure virtual has no default.
//!
//! The headers are read once per CEF version into a text file committed with
//! the bindings; regeneration reads that file.

use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    sync::OnceLock,
};

/// What CEF's annotations say about one C API function.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// C names of the parameters that are never null
    pub required: BTreeSet<String>,
    /// A client method with no default, declared pure virtual in C++
    pub pure: bool,
}

/// CEF's annotations for every C API function, keyed by the C struct that
/// holds a method (without the leading underscore) or by `-` for a global
/// function, then by the function's C name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Annotations {
    entries: BTreeMap<(String, String), Entry>,
}

const GLOBAL_SCOPE: &str = "-";

impl Annotations {
    /// The annotations of method `method` of C struct `owner`, as bindgen names
    /// it (`_cef_render_handler_t`).
    pub fn method(&self, owner: &str, method: &str) -> Option<&Entry> {
        let owner = owner.strip_prefix('_').unwrap_or(owner);
        self.entries.get(&(owner.to_owned(), method.to_owned()))
    }

    /// The annotations of global function `function` (`cef_initialize`).
    pub fn function(&self, function: &str) -> Option<&Entry> {
        self.entries
            .get(&(GLOBAL_SCOPE.to_owned(), function.to_owned()))
    }

    /// Reads the annotations of every header under `include`, CEF's C++ API.
    pub fn extract(include: &Path) -> crate::Result<Self> {
        let mut annotations = Self::default();
        let mut headers = vec![];
        for dir in [include.to_path_buf(), include.join("views")] {
            for entry in fs::read_dir(&dir)? {
                let path = entry?.path();
                if path.extension().is_some_and(|ext| ext == "h") {
                    headers.push(path);
                }
            }
        }
        headers.sort();
        for header in headers {
            let text = fs::read_to_string(&header)?;
            annotations.add_header(&text);
        }
        Ok(annotations)
    }

    fn add_header(&mut self, text: &str) {
        let text = text.replace("\r\n", "\n").replace("> >", ">>");

        for caps in global_function_pattern().captures_iter(&text) {
            let attribs = Attributes::parse(&caps[1]);
            let Some(name) = declared_name(&caps[2]) else {
                continue;
            };
            let c_name = capi_name(attribs.capi_name.as_deref().unwrap_or(name), None);
            let entry = Entry {
                required: required_params(&caps[3], &attribs),
                pure: false,
            };
            self.insert(GLOBAL_SCOPE, &c_name, entry);
        }

        for caps in class_pattern().captures_iter(&text) {
            let attribs = Attributes::parse(&caps[1]);
            let class = &caps[2];
            let body = &caps[4];
            let struct_name = capi_struct_name(class);
            let is_client = attribs.source.as_deref() == Some("client");

            for caps in virtual_method_pattern().captures_iter(body) {
                let attribs = Attributes::parse(&caps[1]);
                let Some(name) = declared_name(&caps[2]) else {
                    continue;
                };
                let c_name = capi_name(attribs.capi_name.as_deref().unwrap_or(name), None);
                let entry = Entry {
                    required: required_params(&caps[3], &attribs),
                    pure: is_client && caps[4].trim_start().starts_with('='),
                };
                self.insert(&struct_name, &c_name, entry);
            }

            // Static methods are global functions in the C API, prefixed with
            // the class
            let prefix = struct_name.trim_end_matches("_t");
            for caps in static_method_pattern().captures_iter(body) {
                let attribs = Attributes::parse(&caps[1]);
                let Some(name) = declared_name(&caps[2]) else {
                    continue;
                };
                let c_name = capi_name(attribs.capi_name.as_deref().unwrap_or(name), Some(prefix));
                let entry = Entry {
                    required: required_params(&caps[3], &attribs),
                    pure: false,
                };
                self.insert(GLOBAL_SCOPE, &c_name, entry);
            }
        }
    }

    fn insert(&mut self, scope: &str, name: &str, entry: Entry) {
        self.entries
            .insert((scope.to_owned(), name.to_owned()), entry);
    }

    /// Reads annotations written by [`Self::write`].
    pub fn read(path: &Path) -> crate::Result<Self> {
        let mut annotations = Self::default();
        for line in fs::read_to_string(path)?.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut words = line.split_whitespace();
            let (Some(scope), Some(name), Some(pure)) = (words.next(), words.next(), words.next())
            else {
                return Err(crate::Error::Annotations(line.to_owned()));
            };
            let pure = match pure {
                "pure" => true,
                "-" => false,
                _ => return Err(crate::Error::Annotations(line.to_owned())),
            };
            let entry = Entry {
                required: words.map(str::to_owned).collect(),
                pure,
            };
            annotations.insert(scope, name, entry);
        }
        Ok(annotations)
    }

    /// Writes one line per function: its scope, its name, `pure` or `-`, then
    /// its required parameters, under a header naming CEF version `source`.
    pub fn write(&self, path: &Path, source: &str) -> crate::Result<()> {
        let mut text = format!(
            "# CEF's C++ annotations of CEF {source}, read by update-bindings --annotations.\n\
             # Each line: the C struct of a method (- for a global function), the C function,\n\
             # pure for a client method without a default (else -), the parameters never null.\n"
        );
        for ((scope, name), entry) in &self.entries {
            text.push_str(scope);
            text.push(' ');
            text.push_str(name);
            text.push_str(if entry.pure { " pure" } else { " -" });
            for param in &entry.required {
                text.push(' ');
                text.push_str(param);
            }
            text.push('\n');
        }
        fs::write(path, text)?;
        Ok(())
    }
}

/// The attributes of one `/*--cef(...)--*/` comment that matter here.
#[derive(Default)]
struct Attributes {
    optional: BTreeSet<String>,
    capi_name: Option<String>,
    source: Option<String>,
}

impl Attributes {
    fn parse(text: &str) -> Self {
        let mut attribs = Self::default();
        for item in text.split(',') {
            let Some((key, value)) = item.split_once('=') else {
                continue;
            };
            let value = value.trim().to_owned();
            match key.trim() {
                "optional_param" => {
                    attribs.optional.insert(value);
                }
                "capi_name" => attribs.capi_name = Some(value),
                "source" => attribs.source = Some(value),
                _ => {}
            }
        }
        attribs
    }
}

// The patterns of CEF's translator (tools/cef_parser.py, `_cre_attrib`,
// `_cre_func` and the class, virtual and static function expressions)
const ATTRIB: &str = r"/\*--cef\(([A-Za-z0-9_ ,=:\n]*)\)--\*/";
const FUNC: &str = r"([A-Za-z][A-Za-z0-9_<>:,\*&\s]+)";

fn global_function_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(&format!(r"(?s)\n{ATTRIB}\n{FUNC}\((.*?)\)")).expect("valid pattern")
    })
}

fn class_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(&format!(
            // A body opens on a line of its own; an empty class (`{};`) has none
            r"(?s)\n{ATTRIB}\nclass\s+([A-Za-z0-9_]+)\s*:\s*public(?:\s+virtual)?\s+([A-Za-z0-9_]+)\s*\{{\n(.*?)\n\}};"
        ))
        .expect("valid pattern")
    })
}

fn virtual_method_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(&format!(
            r"(?s)\n\s+{ATTRIB}\n\s+virtual\s+{FUNC}\((.*?)\)[\sA-Za-z0-9_]*(=|\{{|;)"
        ))
        .expect("valid pattern")
    })
}

fn static_method_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(&format!(r"(?s)\n\s+{ATTRIB}\n\s+static\s+{FUNC}\((.*?)\)"))
            .expect("valid pattern")
    })
}

/// The declared name at the end of a return type and name.
fn declared_name(retval_and_name: &str) -> Option<&str> {
    retval_and_name
        .split(|c: char| c.is_whitespace() || c == '*' || c == '&')
        .rfind(|word| !word.is_empty())
}

/// The C API struct name of C++ class `class` (`CefRenderHandler` is
/// `cef_render_handler_t`).
fn capi_struct_name(class: &str) -> String {
    let mut name = capi_name(class, None);
    name.push_str("_t");
    name
}

/// CEF's conversion of a C++ CamelCase name to its C API name
/// (`get_capi_name` in tools/cef_parser.py). A static method's name takes
/// its class's C name as `prefix`, less a repeat of the class in the name.
fn capi_name(name: &str, prefix: Option<&str>) -> String {
    let mut result = String::new();
    let mut last = None::<char>;
    for c in name.chars() {
        if !result.is_empty()
            && c.is_alphabetic()
            && c.is_uppercase()
            && last.is_some_and(|last| last.is_alphanumeric() && !last.is_uppercase())
        {
            result.push('_');
        }
        result.extend(c.to_lowercase());
        last = Some(c);
    }
    match prefix {
        Some(prefix) => {
            if let Some(subprefix) = prefix.strip_prefix("cef") {
                if let Some(pos) = result.find(subprefix) {
                    result.replace_range(pos..pos + subprefix.len(), "");
                }
            }
            format!("{prefix}_{result}")
        }
        None => result,
    }
}

/// The parameters of `params`, a C++ parameter list, that CEF's glue checks
/// for null and the annotation leaves required.
fn required_params(params: &str, attribs: &Attributes) -> BTreeSet<String> {
    split_params(params)
        .into_iter()
        .filter_map(|param| {
            let name = declared_name(param)?;
            let ty = param[..param.rfind(name)?].trim();
            (is_checked(ty) && !attribs.optional.contains(name)).then(|| name.to_owned())
        })
        .collect()
}

/// Splits a parameter list at the commas outside template brackets.
fn split_params(params: &str) -> Vec<&str> {
    let mut parts = vec![];
    let mut depth = 0usize;
    let mut start = 0;
    for (i, c) in params.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(params[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(params[start..].trim());
    parts
        .into_iter()
        .filter(|part| !part.is_empty() && *part != "void")
        .collect()
}

/// Whether CEF's glue checks a parameter of C++ type `ty` for null: a
/// reference of any kind, a CEF object pointer (`CefRefPtr`, `CefOwnPtr`,
/// `CefRawPtr`) or the address of a simple value (`bool*`, `int*`). A
/// `void*` or a platform handle is passed as it is.
fn is_checked(ty: &str) -> bool {
    if ty.ends_with('&') {
        return true;
    }
    if ["CefRefPtr<", "CefOwnPtr<", "CefRawPtr<"]
        .iter()
        .any(|ptr| ty.contains(ptr))
    {
        return true;
    }
    match ty.strip_suffix('*') {
        Some(pointee) => {
            let pointee = pointee.trim().trim_start_matches("const ").trim();
            SIMPLE_TYPES.contains(&pointee)
        }
        None => false,
    }
}

/// The simple types whose address CEF's glue checks (`simple_byaddr` and
/// `bool_byaddr` in tools/cef_parser.py).
const SIMPLE_TYPES: &[&str] = &[
    "bool", "int", "int16_t", "uint16_t", "int32_t", "uint32_t", "int64_t", "uint64_t", "double",
    "float", "size_t", "char", "char16_t", "time_t",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capi_names_follow_cefs_translator() {
        assert_eq!(capi_struct_name("CefRenderHandler"), "cef_render_handler_t");
        assert_eq!(capi_name("GetViewRect", None), "get_view_rect");
        assert_eq!(capi_name("GetURL", None), "get_url");
        assert_eq!(
            capi_name("OnLoadingStateChange", None),
            "on_loading_state_change"
        );
        assert_eq!(capi_name("CefInitialize", None), "cef_initialize");
        assert_eq!(
            capi_name("CreateBrowserSync", Some("cef_browser_host")),
            "cef_browser_host_create_browser_sync"
        );
        assert_eq!(
            capi_name("GetGlobalContext", Some("cef_request_context")),
            "cef_request_context_get_global_context"
        );
    }

    #[test]
    fn required_parameters_are_the_checked_unannotated_ones() {
        let header = "
/*--cef(source=client)--*/
class CefRenderHandler : public virtual CefBaseRefCounted {
 public:
  ///
  /*--cef()--*/
  virtual void GetViewRect(CefRefPtr<CefBrowser> browser, CefRect& rect) = 0;

  /*--cef(optional_param=text)--*/
  virtual void OnTooltip(CefRefPtr<CefBrowser> browser, CefString& text) {}

  /*--cef()--*/
  virtual void OnPaint(CefRefPtr<CefBrowser> browser,
                       PaintElementType type,
                       const RectList& dirtyRects,
                       const void* buffer,
                       int width,
                       int height) {}

  /*--cef()--*/
  virtual bool OnPreKeyEvent(CefRefPtr<CefBrowser> browser,
                             const CefKeyEvent& event,
                             CefEventHandle os_event,
                             bool* is_keyboard_shortcut) { return false; }
};

/*--cef(optional_param=application,optional_param=windows_sandbox_info)--*/
bool CefInitialize(const CefMainArgs& args,
                   const CefSettings& settings,
                   CefRefPtr<CefApp> application,
                   void* windows_sandbox_info);
";
        let mut annotations = Annotations::default();
        annotations.add_header(header);

        let rect = annotations
            .method("_cef_render_handler_t", "get_view_rect")
            .expect("get_view_rect");
        assert!(rect.pure);
        assert_eq!(rect.required, ["browser", "rect"].map(str::to_owned).into());

        let tooltip = annotations
            .method("_cef_render_handler_t", "on_tooltip")
            .expect("on_tooltip");
        assert!(!tooltip.pure);
        assert_eq!(tooltip.required, ["browser"].map(str::to_owned).into());

        let paint = annotations
            .method("_cef_render_handler_t", "on_paint")
            .expect("on_paint");
        assert_eq!(
            paint.required,
            ["browser", "dirtyRects"].map(str::to_owned).into()
        );

        let key = annotations
            .method("_cef_render_handler_t", "on_pre_key_event")
            .expect("on_pre_key_event");
        assert_eq!(
            key.required,
            ["browser", "event", "is_keyboard_shortcut"]
                .map(str::to_owned)
                .into()
        );

        let initialize = annotations
            .function("cef_initialize")
            .expect("cef_initialize");
        assert_eq!(
            initialize.required,
            ["args", "settings"].map(str::to_owned).into()
        );
    }

    #[test]
    fn an_empty_class_keeps_the_next_class_to_itself() {
        let header = "
/*--cef(source=library)--*/
class CefMenuButtonPressedLock : public CefBaseRefCounted {};

/*--cef(source=client)--*/
class CefMenuButtonDelegate : public CefButtonDelegate {
 public:
  /*--cef()--*/
  virtual void OnMenuButtonPressed(
      CefRefPtr<CefMenuButton> menu_button,
      const CefPoint& screen_point,
      CefRefPtr<CefMenuButtonPressedLock> button_pressed_lock) = 0;
};
";
        let mut annotations = Annotations::default();
        annotations.add_header(header);
        let pressed = annotations
            .method("_cef_menu_button_delegate_t", "on_menu_button_pressed")
            .expect("on_menu_button_pressed");
        assert!(pressed.pure);
        assert_eq!(
            pressed.required,
            ["button_pressed_lock", "menu_button", "screen_point"]
                .map(str::to_owned)
                .into()
        );
    }

    #[test]
    fn written_annotations_read_back_the_same() {
        let mut annotations = Annotations::default();
        annotations.insert(
            "cef_render_handler_t",
            "get_view_rect",
            Entry {
                required: ["browser", "rect"].map(str::to_owned).into(),
                pure: true,
            },
        );
        annotations.insert(GLOBAL_SCOPE, "cef_shutdown", Entry::default());
        let path = std::env::temp_dir().join(format!(
            "update-bindings-annotations-{}.txt",
            std::process::id()
        ));
        annotations.write(&path, "a test").expect("written");
        let read = Annotations::read(&path).expect("read");
        let _ = fs::remove_file(&path);
        assert_eq!(read, annotations);
    }
}
