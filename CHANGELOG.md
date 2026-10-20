# Changelog

All notable changes to tanso are recorded here, from the fork from cef-rs on. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions are `<CEF major>.<minor>.<patch>+<CEF version>`; the minor moves with a breaking change. Each entry names its commits.

## [Unreleased]

## [154.5.0+154.0.33] - 2026-10-20

The changes since 154.4.0. tanso takes over from tetsu in its code as well as its package names.

### Changed

- **Breaking.** tanso continues tetsu. `links` is `tanso`, so a build script reads `DEP_TANSO_*`. `tanso` pins `tanso-sys` exactly (`95ea571`).
- **Breaking.** The shared CEF installation is `tanso/cef` under the local data directory, where tanso-download installs CEF and tanso-sys looks for it. tanso-download moves to 4.0.0 for it (`95ea571`).
- **Breaking.** `cef_transition_type_t` is a flag set, since CEF returns a transition as a source combined with qualifier flags. Compare `TransitionType` constants or test bits through `get_raw`. Every flag type's wrapper gains the named constants and `get_raw` that enum wrappers have (`4e9035d`).
- A panic in a generated callback aborts at the CEF boundary, its message kept, instead of unwinding across the C ABI (`d091521`).

### Added

- 32-bit Windows (`i686-pc-windows-msvc`). The bindings use the calling convention CEF's callbacks use there. Generation fails on a convention it cannot express instead of leaving the method out. CI tests against 32-bit CEF (`1c15c14`).
- Safe ID mappers. `id_for_command_id_name`, `id_for_pack_resource_name` and `id_for_pack_string_name` take a `&CStr` and answer `None` for a name this CEF build does not know (`8006203`).
- On macOS a CEF framework dyld has already mapped is reused when it is the requested file. A different CEF binary is refused with `AlreadyLoaded` (`7fab35e`).
- `v8_value_create_array_buffer_from_bytes` creates an ArrayBuffer holding a copy of a byte slice, on a thread where V8 runs (`748d237`).

### Fixed

- By-reference parameters follow CEF's reference transfer. A value a handler leaves unchanged has its reference released, a cleared value is written back as null and a returned value is adopted only when the pointer changed. A popup no longer keeps its opener's client alive for the life of the process. A value CEF left unchanged is no longer released once too often (`cebdea5`).
- CEF's ID mappers are called one at a time. They share tables CEF fills on first use. Two threads calling them at once crashed on macOS (`5f75921`).
- An empty `CefString` passed as an out-parameter reaches CEF as an empty string of its own, not a null pointer CEF ignored or wrote through (`b0412e5`).
- Clones of string lists, maps and multimaps own a copy instead of handing CEF a pointer into the Rust value. A dropped multimap is freed, not only cleared (`6a05f8b`).
- String conversions keep CEF's best-effort output instead of returning an empty string and leaking CEF's buffer. `CefStringUtf8::as_str` checks UTF-8, `Display` reads invalid bytes as U+FFFD and nothing prints to stderr (`c679d1d`).
- The cefsimple example's sandbox library and binary no longer share a name, which failed parallel Windows builds with LNK1201 (`4c1868a`). The example is removed below.

### Removed

- The examples (cefsimple, osr, tests_shared) and the bundler (bundle-cef-app). Kurogane is tanso's acceptance suite. `export-cef-dir` stays to install CEF for the tests (`9ed82e6`).
- `export-cef-dir` is no longer published (`95ea571`).

### Repository

- The bindings are regenerated for every target by the bindings workflow (pull request #3, `b946486`).
- One root CHANGELOG.md replaces the per-crate changelogs inherited from cef-rs.

## [154.4.0+154.0.33] - 2026-10-09

tetsu's release, the first after the fork from cef-rs at `f044f89` (2026-10-03). It is the tag `v154.4.0+154.0.33` on `881d8cb` (2026-10-08), which Kurogane 0.0.6-alpha.2 pinned. On 2026-10-09 the same tree went to crates.io as tanso, tanso-sys and tanso-build 154.4.0 and tanso-download 3.0.0, renamed in the package manifests alone. Its code still says tetsu (`links = "tetsu"`, the shared installation `tetsu/cef`, `tetsu_sys` in its docs) until 154.5.0.

### Changed

- **Breaking.** The bindings are Kurogane's fork of tauri-apps/cef-rs, taken from upstream by choice. The crates are tetsu, tetsu-sys and tetsu-download (`e784395`). The `dox` features and tetsu-sys's `sandbox` feature are gone (`95fc48c`).
- **Breaking.** libcef loads at run time from the path the application chooses instead of being linked at build time, on Linux and Windows (`794b6bc`) and on macOS (`585c436`). Building needs no C++ wrapper, CMake, Ninja or CEF distribution. An application calls `load_libcef` before any CEF function. `FRAMEWORK_PATH` is `LIBCEF_FILE`. The old library, loader and sandbox bindings are removed.
- **Breaking.** CEF is found at run time in the bundle, beside the executable, through `CEF_PATH` or in the shared installation. A build that does not match the bindings is refused. `find_cef_dir` replaces `get_cef_dir`, `LibraryLoader` and `TETSU_STAGE_RUNTIME`. `load_libcef` verifies the CEF build and sets the API version (`043b5b8`).
- **Breaking.** A set `CEF_PATH` must name a valid CEF distribution. The build no longer downloads CEF into a missing or invalid one (`eeb5b69`).
- **Breaking.** Each CEF version installs once in the user's shared `tetsu/cef` directory, which every project reuses. Without `CEF_PATH`, tetsu-sys installs there instead of `OUT_DIR` (`12aca39`).
- **Breaking.** CEF installs are validated by version and target. `check_archive_json` takes both, install errors are returned instead of panicking and the metadata bundled applications need is kept. `get-latest` owns the CEF bump (`f3a1d6c`).
- **Breaking.** `LoadError::VersionMismatch` carries the expected CEF version beside the loaded one (`d352433`).
- **Breaking.** macOS sandbox setup reports a failure instead of panicking and refuses a second initialization. `Sandbox::new` and `Sandbox::initialize` return `Result`. `Sandbox` no longer implements `Default`. Process arguments pass without lossy UTF-8 conversion. Windows command-line quoting stays as the OS gave it (`9c32009`).
- **Breaking.** The Rust ports of CEF's C++ wrapper (the message router, the resource manager) are opt-in behind `wrapper`. Bundling moves out of tetsu into a development tool. `sandbox`, `build-util`, `linux-x11` and `tetsu::build_util` are removed (`03e4a59`).
- The CEF runtime was staged beside the binaries only with `TETSU_STAGE_RUNTIME=1` (`e710a1f`), until finding CEF at run time replaced staging (`043b5b8`).
- Windows builds linked `libcef.dll` without building the C++ wrapper (`0a469a1`), until loading at run time replaced linking (`794b6bc`).
- The Windows command line is built the way Unix's is (`0e56b55`).

### Added

- tetsu-build embeds CEF's Windows application manifest in an application's executables, so unsandboxed and bundled applications report the same Windows compatibility as CEF's own executables (`6b3305a`).

### Fixed

- Binary query responses are copied into V8. CEF's official builds enable the V8 sandbox, where `cef_v8_value_create_array_buffer` returns null, so responses were lost unless the sandbox feature was on (`c67526d`).
- A build script error names the file or directory that failed (`265c32d`).

### Repository

- release-plz is gone. The bindings workflow opens pull requests against `dev` (`5fde943`).
- CI drops the obsolete CEF setup and tests Kurogane on every change (`16d1742`).
- The upstream relationship with cef-rs, the CEF bump workflow and private vulnerability reporting are documented. Dependabot updates dependencies and actions (`881d8cb`).
