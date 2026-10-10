//! Links what tanso's own code needs; nothing of CEF.
//!
//! The application loads libcef when it starts (`tanso_sys::load_libcef`),
//! so a build reads no CEF distribution.

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    // tanso's macOS application code uses AppKit
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo::rustc-link-lib=framework=AppKit");
    }
}
