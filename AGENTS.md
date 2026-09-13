# Working on wgame

These instructions apply to the entire workspace.

## Project map and references

wgame is a safe Rust workspace for cooperative async window applications and
2D rendering with winit and wgpu.

- `wgame`: public facade, windows, frames, libraries, and reexports.
- `wgame-app`, `wgame-app-input`, `wgame-utils`: runtime, task lifecycles,
  event streams, and timers.
- `wgame-gfx`: graphics context, scenes, ordering, cameras, and render targets.
- `wgame-gfx-shapes`, `wgame-gfx-texture`, `wgame-gfx-typography`: GPU resources
  and renderers; `wgame-image`, `wgame-typography`, `wgame-fs`: CPU resources/I/O.
- `wgame-shader`, `wgame-shader-macros`, `wgame-macros`: shader attributes,
  templates, and entry-point macros.
- `wgame-examples`: runnable applications and assets.

Read [README.md](README.md) for features and the quickstart,
[the usage guide](docs/GUIDE.md) for API contracts, and
[migration notes](docs/MIGRATION.md) before changing public behavior.
[Validation](docs/VALIDATION.md) describes platform checks and their limitations;
[performance](docs/PERFORMANCE.md) documents benchmark methodology.
[The roadmap](docs/ROADMAP.md) records the stabilization work.

## Change conventions

- Follow the existing Rust style and preserve `forbid(unsafe_code)` boundaries.
- Keep shared dependency declarations in the workspace `Cargo.toml` and use
  workspace inheritance. Keep `Cargo.lock` consistent with dependency changes.
- Preserve the separation between CPU resources and GPU rendering crates.
- Add focused regression coverage for behavioral fixes. Prefer CPU tests for
  lifecycle and ordering logic; use offscreen pixel tests for rendering behavior.
- Update public API documentation and affected examples with API changes, and
  record breaking changes in the migration notes. The root README and usage
  guide are included in rustdoc, so their Rust snippets are tested.
- Macro changes need consumer-level coverage; see
  [entry-point tests](wgame/tests/entry_points.rs) for existing compile probes.

## Contracts to preserve

- `Task<T>` and `WindowedTask<T>` have one result consumer. Clone their control
  handles, not the futures. Dropping a plain task/handle detaches it;
  `cancel_on_drop()` ties cancellation to a scope. Cancellation must be
  idempotent and handle tasks/windows still waiting in a creation queue.
- The executor is cooperative and local to the event-loop thread. Avoid blocking
  work there. Do not hold `RefCell` borrows while dropping futures, invoking user
  callbacks, or performing other actions that can reenter runtime state.
- Input termination and handler drop wake consumers; buffered events drain before
  `None`. Redraw events stay outside input queues. Periodic timers preserve their
  deadline phase and report the duration of whole elapsed periods.
- Lower scene orders draw first; nested orders compare lexicographically with
  zero padding. Equal-order insertion order is stable. Batch only adjacent
  compatible instances within an order: interleaved A/B/A resources must retain
  their transparent compositing order. `Scene::len()` counts batches.
- Atlas generations are append-only: dropping/resizing items must not reuse or
  clear old rectangles. On exhaustion, repack live items plus the pending item
  into a new generation; try the same dimensions at up to 50% live area, otherwise
  grow. GPU mirrors must replace textures on generation changes, even at the
  same dimensions. Baked/encoded drawing retains its matching old GPU texture.
- Built-in shape/text scenes retain handles and resolve atlas coordinates when
  baking. Rebaking an existing scene follows relocation; rebuilding source
  objects is needed for object-data changes. Baked instance buffers and bindings
  are fixed, but explicit pixel updates to their GPU generation remain mutable.
- Texture updates must maintain the one-pixel filtering border, including partial
  updates and resize. Direct backing-atlas edits bypass this maintenance.
- Shader attributes describe vertex/instance buffers, not uniform packing.
  `Mat3` uses 36 bytes and three `Float32x3` columns; arrays concatenate attributes.
- `AutoScene::render/discard` and `Frame::present/discard` are explicit lifecycle
  boundaries. Normal drop retains automatic rendering/presentation, but panic
  unwinding must not submit work.
- Surface recovery must preserve pending resize state; do not configure zero-size
  surfaces. Typography currently supports monochrome outlines and glyph offsets;
  do not imply font fallback or complete paragraph layout support.

## Validation

Run commands from the repository root unless indicated otherwise. For broad code
changes, the baseline is:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
bash scripts/check-features.sh
```

The feature script requires the `wasm32-unknown-unknown` target. It checks every
combination of optional facade features with desktop, plus web configurations.
Do not use `--all-features`: native and web runtime features are mutually
exclusive. Preserve independent optional features; `load_texture` needs `fs` +
`image`, and `load_font` needs `fs` + `typography`. Standalone `wgame-app` commands
need a platform backend, such as `x11`/`wayland` on Linux or `std` on Windows/macOS.

For graphics changes, explicitly run the otherwise ignored GPU tests. For window
or runtime integration changes, also run the playground smoke check:

```sh
cargo test --locked -p wgame --test rendering -- --ignored
cargo run --locked -p wgame-examples --bin playground -- --smoke
```

Explicit GPU tests require an adapter and must fail if none is available. On
headless Linux, Mesa Vulkan and Xvfb can support these checks; see the validation
guide. For performance changes, use
`cargo run --locked -p wgame --example render_bench --release` and report the
adapter, workload, and measurement method.

Run older asset-loading examples from `wgame-examples/`; the playground embeds
its assets. For web changes, run `trunk build --no-default-features --features web
--locked` from `wgame-examples/`. The high-level web feature uses WebGL2.
See [example instructions](wgame-examples/README.md) for launch options.

Choose checks appropriate to the change. Prose-only edits need link/content and
whitespace checks; changed executable snippets need doctests. Report checks
actually run and remaining gaps. Distinguish compilation, generated web artifacts,
browser execution, offscreen GPU tests, and real-window tests. Software-renderer
benchmarks do not establish hardware frame rates. Keep measured results and
environment-specific limitations in the validation/performance documents.
