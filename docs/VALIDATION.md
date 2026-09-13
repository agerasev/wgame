# Validation

## Repeatable checks

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
rustup target add wasm32-unknown-unknown
bash scripts/check-features.sh
cargo test --locked -p wgame --test rendering -- --ignored
cargo run --locked -p wgame-examples --bin playground -- --smoke
```

The feature script checks all 32 combinations of the optional `shapes`, `fs`,
`image`, `typography`, and `utils` features with desktop, plus minimal web and the
full web examples. No `--all-features` build is supported: native and web runtime
features are mutually exclusive. The standalone runtime requires a backend
feature, such as `x11`, `wayland`, or `std` on the appropriate native platform.

The root README and usage guide are included in rustdoc, so their Rust examples
are compiled by `cargo test`. Consumer integration tests compile both entry
macros and named/tuple shader derives. Most lifecycle and scene-order tests run
without a window or GPU.

The eight rendering tests are explicitly ignored in the ordinary suite so
machines without graphics drivers can run CPU tests. The explicit rendering
command must pass; adapter absence is an error, not a silent skip. Tests cover:

- transparent A/B/A compositing and retained-vs-rebuilt equality;
- texture atlas growth, updates, and linear filtering;
- partial updates at nonzero offsets, border duplication, empty updates, resize;
- glyph atlas growth and text rendering across offscreen target sizes;
- source drop with baked or encoded drawing, including append, same-size
  compaction, and atlas growth;
- resize without outer atlas growth and rebaking an existing scene;
- font-local glyph repacking while retaining the same outer GPU generation;
- initial GPU upload of an already populated CPU atlas.

CPU atlas tests also cover abandoned rectangles, live pixels across compaction,
packing fallback for long items, and repeated reclamation without atlas growth.

On a Linux machine without a desktop, use Mesa Vulkan and Xvfb:

```sh
WGPU_BACKEND=vulkan cargo test --locked -p wgame --test rendering -- --ignored
WGPU_BACKEND=vulkan xvfb-run -a cargo run --locked -p wgame-examples --bin playground -- --smoke
```

These commands assume the Mesa Vulkan driver and Xvfb are installed. CI installs
them in its disposable Ubuntu runner. The local review used Xvfb unpacked into
`/tmp` rather than installing system packages.

## Web artifacts

```sh
cd wgame-examples
trunk build --no-default-features --features web --locked
```

If an environment sets `NO_COLOR=1`, the installed Trunk version may reject it;
use `NO_COLOR=true trunk build ...` or unset that variable. Successful Trunk output
includes the wasm module, JavaScript glue, HTML, and copied assets. Cargo `check`
is a compilation check only.

## Results and practical limits

Locally validated on Linux x86-64 with rustc 1.98.0:

- 34 workspace tests, including documentation and compile probes.
- Formatting and Clippy with warnings treated as errors.
- The optional-feature matrix and both WebAssembly configurations.
- Four offscreen pixel tests on Mesa llvmpipe Vulkan.
- Playground startup, twelve frames, and clean shutdown under Xvfb.
- A full Trunk build and a release rendering benchmark.

The GitHub workflow is configured for native tests on Linux, Windows, and macOS,
plus Linux feature/GPU/smoke jobs. Hosted CI has not been run from this local
session. Windows/macOS execution, real display suspend/resume, HiDPI behavior,
and browser runtime behavior still need platform testing. No browser is exposed
to the current automation session, so the successful wasm/Trunk build is not
reported as a browser runtime test.

Pixel tests use analytic values and comparisons, not a complete visual gallery.
They do not establish complete typography support or driver portability.
Typography limitations and baked-scene invalidation rules are in the usage
guide. Performance results are software-renderer measurements, not hardware
frame-rate guarantees.

## Atlas generation follow-up (2026-09-13)

After implementing append-only atlas generations, Linux validation passed:

- 39 workspace tests/doctests and all eight explicitly selected GPU tests.
- Formatting, strict Clippy, and all 34 feature configurations.
- The twelve-frame playground smoke check under Xvfb.
- The release rendering benchmark on Mesa llvmpipe Vulkan.

The initial sandboxed Xvfb attempt could not bind its display socket; rerunning
with local socket access succeeded. The GPU tests verify source drop before
rendering and after encoding, same-size compaction, growth, resize, and glyph
repacking. Existing browser and other-platform runtime limitations still apply.
