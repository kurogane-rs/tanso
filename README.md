# tanso

Use CEF in Rust. tanso is the CEF binding of [Kurogane](https://github.com/0x48piraj/kurogane) with the crates `tanso`, `tanso-sys`, `tanso-download` and `tanso-build`.

tanso began as a fork of [cef-rs](https://github.com/tauri-apps/cef-rs) and fixes the problems Kurogane met there. Kurogane's needs decide its shape. Other projects are welcome to use it; it makes no promise to stay a general-purpose binding.

## Supported Targets

| Target | Linux | macOS | Windows |
| ------ | ----- | ----- | ------- |
| x86_64 | ✅    | ✅    | ✅      |
| ARM64  | ✅    | ✅    | ✅      |

## Usage

### Install CEF

Applications load libcef when they start (see [Loading libcef](#loading-libcef)), so building needs no CEF. At run time `tanso_sys::find_cef_dir` looks in the macOS application bundle the executable belongs to, beside the executable, in the directory `CEF_PATH` names, then in the shared installation `tanso/cef/<version>/cef_<os>_<arch>` under the local data directory (`~/.local/share`, `%LOCALAPPDATA%`, `~/Library/Application Support`). Install the CEF version the crate was generated for there, once for every project:

```sh
cargo run -p export-cef-dir
```

Repeat this step each time you upgrade to a `tanso` crate of a new CEF version.

### Use a CEF of your own

Export a distribution elsewhere and point `CEF_PATH` at it when starting the application. A set `CEF_PATH` that names no directory is an error; the lookup never skips it for another CEF.

#### Linux or macOS

```sh
cargo run -p export-cef-dir -- --force $HOME/.local/share/cef
export CEF_PATH="$HOME/.local/share/cef"
```

#### Windows (using PowerShell)

```pwsh
cargo run -p export-cef-dir -- --force $env:USERPROFILE/.local/share/cef
$env:CEF_PATH="$env:USERPROFILE/.local/share/cef"
```

#### Nix (on Linux)

With the `nixpkgs` channel configured, `export-cef-dir --nix` takes the distribution from the `cef-binary` package:

```sh
cargo run -p export-cef-dir -- --nix --force $HOME/.local/share/cef
export CEF_PATH="$HOME/.local/share/cef"
```

libcef needs no library search path; the application loads it from the directory it finds.

### Loading libcef

The bindings link no libcef. Before any other call into CEF the application finds its CEF and loads it:

```rust
let cef = tanso::sys::find_cef_dir()?;
unsafe { tanso::sys::load_libcef(&cef.libcef()) }?;
```

Loading fixes the process's CEF API version to the bindings' and refuses a libcef that is not the CEF build the bindings were generated from. A CEF function called before it panics, naming `load_libcef`. Building needs no CEF distribution on any platform, so nothing is compiled from C++ either.

Outside a bundle, CEF finds its resources (the `.pak` files, `icudtl.dat`, the V8 snapshot and `locales/`) where the application names them (`resources_dir_path`, `locales_dir_path`), in the directory `find_cef_dir` returned.

### The Windows application manifest

CEF's Windows executables carry an application manifest; without it Windows tells Chromium it runs on Windows 8. `tanso-build` embeds the same one from a build script:

```rust
fn main() {
    tanso_build::embed_windows_manifest();
}
```

It does nothing on other targets. `tanso_build::WINDOWS_MANIFEST` is the manifest itself.

### Features of `tanso`

- `resources` (default): the names of Chromium's command and resource IDs, for `tanso_sys::cef_id_for_command_id_name` and its siblings.
- `wrapper`: Rust ports of CEF's C++ wrapper, such as its message router and resource manager.
- `accelerated_osr`: imports the textures of accelerated off-screen rendering into `wgpu`.

### Cross-compiling to Windows

The `tanso-sys` crate can be cross-compiled to `x86_64-pc-windows-msvc` from Linux with [cargo-xwin](https://github.com/rust-cross/cargo-xwin), which downloads the Windows SDK and links with `lld-link`. Install `clang`, `lld` and `llvm` from your package manager, then:

```sh
rustup target add x86_64-pc-windows-msvc
cargo install cargo-xwin
cargo xwin build -p tanso --target x86_64-pc-windows-msvc
```

Nothing is compiled from C++ and nothing of CEF is linked, so a cross-compiled build needs no CEF distribution. `cargo run -p export-cef-dir -- --target x86_64-pc-windows-msvc <dir>` exports the Windows one to run it with.

## Following CEF and cef-rs

tanso takes cef-rs as a reference and never merges it. [UPSTREAM.md](UPSTREAM.md) says how cef-rs is reviewed and how tanso moves to a new CEF.

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.
