# Examples

## Desktop

From the repository root, run the self-contained playground:

```sh
cargo run -p wgame-examples --bin playground
```

Mouse movement positions the ring; Space pauses animation. Resize the window to
exercise camera/text updates. Close it to cancel its background task. Add
`-- --smoke` for a twelve-frame startup/render/shutdown check.

Inspect four-point quads and variable-width polylines:

```sh
cargo run -p wgame-examples --bin polylines
```

The gallery shows texture interpolation, tapered widths, miter/bevel joins at
limit 4, index-based UVs, repeated points, zero widths, and translucent crossings.
Space pauses the angle sweep, P toggles junction markers, and Escape closes the
window. It embeds its assets and also supports `-- --smoke`.

The older examples load assets relative to the current directory:

```sh
cd wgame-examples
cargo run --bin shapes
cargo run --bin events
cargo run --bin shapes --features dump
```

The optional dump feature writes atlas PNGs under `dump/`.

## Web (WebGL2)

Install the Rust wasm target and Trunk, then run from this directory:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve --no-default-features --features web
```

Open the URL printed by Trunk. `index.html` selects the shapes example and copies
`assets/`. To use the playground, change `data-bin="shapes"` to
`data-bin="playground"` or `data-bin="polylines"`. Both embed their assets.

Do not combine the default desktop features with `web`. WebGPU is not enabled
by the high-level web feature. Use `trunk build --no-default-features --features
web` for a static build; Cargo's `check` alone does not create browser glue.
If Trunk rejects an inherited `NO_COLOR=1`, set `NO_COLOR=true` or unset it.

## Other examples

```sh
cargo run -p wgame-app --features x11 --example multiple_windows
cargo run -p wgame --example raw_wgpu
cargo run -p wgame --example render_bench --release
```

For `wgame-app`, select a native backend appropriate to your platform (`std` on
Windows/macOS, `x11` or `wayland` on Linux). `multiple_windows` and `raw_wgpu`
open real windows; `render_bench` is headless. Benchmark methodology lives in
[its source documentation](../wgame/examples/render_bench.rs).

The [offscreen rendering tests](../wgame/tests/rendering.rs) need an adapter but
no display. For a window smoke check on headless Linux with Mesa Vulkan and Xvfb:

```sh
WGPU_BACKEND=vulkan xvfb-run -a cargo run --locked -p wgame-examples --bin playground -- --smoke
```
