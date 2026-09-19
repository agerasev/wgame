# Examples

Gallery labels use a separate glyph raster for each displayed font size. The
shared `Labels` helper accounts for physical pixels per drawing unit, including
display scaling and fitted cameras, and refreshes its cache when that scale
changes. Text objects created before a scale change should be rebuilt. Run its
offscreen size/scale regression check with an available GPU or Mesa adapter:

```sh
cargo test --locked -p wgame-examples --lib -- --ignored
```

## Desktop

From the repository root, run the self-contained playground:

```sh
cargo run -p wgame-examples --bin playground
```

Mouse movement positions the ring; Space pauses animation. Resize the window to
exercise camera/text updates. Move it between differently scaled displays to
exercise logical sizing and sharp font rasters. On X11,
`WINIT_X11_SCALE_FACTOR=2` forces a scale factor for manual checking. Close it to cancel its background task. Add
`-- --smoke` for a twelve-frame startup/render/shutdown check.

Inspect four-point quads and variable-width polylines:

```sh
cargo run -p wgame-examples --bin polylines
```

The gallery shows texture interpolation, tapered widths, miter/bevel joins at
limit 4, index-based UVs, repeated points, zero widths, and translucent crossings.
Space pauses the angle sweep, P toggles junction markers, and Escape closes the
window. It embeds its assets and also supports `-- --smoke`.

Explore borrowed viewports with 2D and perspective cameras:

```sh
cargo run -p wgame-examples --bin viewports
```

The first panel contains a nested preview. Move the pointer over a view to see
picking in its local coordinates; Space pauses and Escape closes. Resize to
switch between horizontal and vertical layouts. The example embeds its font and
supports `-- --smoke`. Viewport rectangles use physical pixels; see
[`Target::viewport`](../wgame-gfx/src/target.rs) for bounds and clearing behavior.

Render into a texture and capture a detached CPU snapshot:

```sh
cargo run -p wgame-examples --bin render_textures
```

The live view and frozen snapshot use the same shape sampling API. A checkerboard
reveals transparency, and the live texture contains a borrowed viewport. Press S
to capture again, Space to pause, or Escape to close. The example embeds its font
and supports `-- --smoke`, including asynchronous readback and re-upload. See
[`RenderTexture`](../wgame-gfx-texture/src/render_texture.rs) for ownership and
submission rules.

The older examples load assets relative to the current directory:

```sh
cd wgame-examples
cargo run --bin shapes
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
Dedicated pages are available for these examples:

```sh
trunk serve events.html --no-default-features --features web
trunk serve viewports.html --no-default-features --features web
trunk serve render_textures.html --no-default-features --features web
```

Do not combine the default desktop features with `web`. WebGPU is not enabled
by the high-level web feature. Use `trunk build --no-default-features --features
web` for a static build; Cargo's `check` alone does not create browser glue.
If Trunk rejects an inherited `NO_COLOR=1`, set `NO_COLOR=true` or unset it.

## On-demand repaint

```sh
cargo run -p wgame-examples --bin events
```

The graphical event example embeds its font and starts idle. Move the pointer,
resize the window, or change display scale to redraw. The displayed frame count
stays still otherwise. **Space** toggles a one-second timer, **A** toggles continuous
animation, and **Escape** closes. Timer deadlines remain fixed while other events
arrive. Turning both modes off returns to indefinite waiting. Add `-- --smoke`
for a twelve-frame startup/render/shutdown check.

`WindowHost::wait_for_update(None)` sleeps until an event; a duration also allows
application timers to wake it. Pass zero while animating. Draw the first frame
before waiting. `next_frame` continues to request a frame on every call.

For the low-level event stream without a renderer:

```sh
cargo run -p wgame-app --features x11 --example events
cargo run -p wgame-app --features x11 --example events -- --redraws
```

The logger uses `Input::next()` directly. `--redraws` opts into OS redraw events,
which are excluded from ordinary input streams. Close the window to end the
stream consumer; `--smoke` limits observation to 250 ms.

## Optional egui host

```sh
cargo run -p wgame-egui --example playground
cargo run -p wgame-egui --example playground -- --plain
```

Both hosts use the same event-driven content loop. Move the pointer or resize
while idle; click the canvas and press **Space** to toggle continuous animation.
The egui version adds a matching checkbox, a text field that keeps its own keyboard
focus, a delayed repaint button, and a tooltip that wakes at egui's hover deadline.
The delayed status updates after one second even without further input.

Add `--smoke` for twelve frames including an explicitly discarded frame. With egui,
it also checks that the host settles to idle and wakes for delayed and immediate
repaint requests. The same Mesa/Xvfb setup below applies. Run offscreen composition
tests with `cargo test -p wgame-egui --lib -- --ignored`.

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
