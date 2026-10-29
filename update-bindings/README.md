# update-bindings

Download the prebuilt [Chromium Embedded Framework](https://github.com/chromiumembedded/cef)
archive on any supported platform and run `bindgen` on the C API for the `tanso-sys` crate,
then regenerate the safe bindings in the `tanso` crate.

It then rewrites the `tanso-sys` bindings (`src/loader.rs`): each function
libcef exports becomes a function of the same name and signature that calls through a table the
application fills when it loads libcef (`tanso_sys::load_libcef`). Run without `--bindgen`, it
rewrites bindings bindgen or upstream left as `extern "C"` declarations and leaves rewritten ones
as they are; when upstream changes a bindings file, take upstream's file and run it again.

It reads CEF's C++ annotations from `annotations.txt` beside this README: which parameters CEF
never passes null and which client methods have no default (`src/annotations.rs`). A required
parameter is a reference rather than an `Option`, and a method without a default has no default
body. `--bindgen` refreshes the file from the target's CEF headers; `--annotations <include dir>`
refreshes it from any CEF include directory.

You can find the latest version of the prebuilt CEF archives on the [Chromium Embedded Framework
(CEF) Automated Builds](https://cef-builds.spotifycdn.com/index.html).
