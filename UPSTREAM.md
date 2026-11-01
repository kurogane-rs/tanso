# Following CEF and cef-rs

tanso began as a fork of [cef-rs](https://github.com/tauri-apps/cef-rs). cef-rs is a reference. tanso never merges it. Fixes worth having are ported by hand and tanso's bindings come from its own generator, run on CEF's headers.

## Last reviewed

cef-rs `84d6c77` (2026-10-24, `156.2.0+156.0.14`).

## Reviewing cef-rs

1. Fetch cef-rs and list its commits since the last one reviewed:

   ```sh
   git fetch https://github.com/tauri-apps/cef-rs dev
   git log --oneline f044f89..FETCH_HEAD
   ```

2. Read each one. Port what tanso wants by hand, fixes to the hand-written code (`rc.rs`, `string.rs`, `args.rs`, the macros), to the generator (`update-bindings`) and to the tools. Skip cef-rs's binding regenerations; tanso regenerates its own.
3. Record the commit reviewed above.

## Moving to a new CEF

1. `get-latest` finds the newest stable CEF published for every target. With `--update-version` it moves the workspace version, `<CEF major>.<minor>.<patch>+<CEF version>`. The scheduled `get-latest` workflow does this and opens a pull request against `dev`.
2. `update-bindings` regenerates the bindings for each of the eight targets. `cargo run -p update-bindings -- --bindgen --download --target <triple>` runs bindgen on CEF's headers, rewrites the sys bindings to call through the loaded libcef and generates the safe bindings and resources from them. It also refreshes `update-bindings/annotations.txt` from CEF's C++ headers; review its diff, since a parameter CEF stops requiring turns a reference back into an `Option`. The `update-bindings` workflow runs it on each platform.
3. tanso's tests run on each platform (`cargo test --workspace`).
4. Kurogane's parity runs on the new tanso decide its pin. Kurogane moves to a tanso release only once they pass; tanso releases in pairs with Kurogane ([CONTRIBUTING.md](CONTRIBUTING.md), "Releasing").

The generator emits tanso's crate names, so a regeneration's diff is CEF's change alone. A patch to the bindings goes into the generator or the hand-written code, never into generated files alone.
