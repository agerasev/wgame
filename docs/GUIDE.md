# Usage guide

## Window and frame ownership

Use `#[wgame::window(size = (800, 600))]` on an async function taking
`Window<'_>`. The wrapper creates a window and invokes your function. Closing
ends the current function normally if it returns after `next_frame` yields
`None`. Suspension cancels the window function; on resume the single-window
wrapper invokes it again, rebuilding window-local graphics resources. Store
application data outside this function when it must survive suspension.

`#[wgame::app]` takes an async function with no arguments and is useful when you
need multiple windows. `create_windowed_task` returns a future for one window's
result. A manually managed window task reports `WindowError::Suspended`; its
caller decides whether to recreate it.

A high-level frame borrows its window. `present()` submits commands and presents
once; normal drop does the same. `discard()` submits nothing. Frames and automatic
scenes do not submit work during panic unwinding. Recoverable surface timeouts
request another redraw; lost/outdated surfaces are reconfigured. Fatal acquisition
errors propagate from `next_frame`. Resize notifications survive skipped frames.

An `AutoScene` renders on normal drop. Call `scene.render()` for an explicit
boundary or `scene.discard()` to abandon it. Present only after the scene has
finished borrowing the frame.

## Coordinates and shapes

`frame.scene()` uses a camera with Y pointing upward. The visible Y range is
-1 to 1; X spans minus to plus the window aspect ratio. For pixel coordinates,
use `physical_camera`: the origin is at the top left and Y increases downward.
These are physical pixels, including the display's scaling factor.

```rust,no_run
use wgame::{Library, Window, prelude::*, glam::Vec2, gfx::types::color};
async fn draw(mut window: Window<'_>) -> wgame::Result<()> {
    let library = Library::new(window.graphics());
    let circle = library.shapes().unit_circle().fill_color(color::CYAN);
    while let Some(mut frame) = window.next_frame().await? {
        frame.clear(color::BLACK);
        let camera = frame.physical_camera();
        let mut scene = frame.scene();
        scene.camera = camera;
        scene.add(&circle.scale(40.0).move_to(Vec2::new(100.0, 100.0)));
        scene.render();
        frame.present();
    }
    Ok(())
}
```

Transforms compose in call order: `scale(40).move_to(position)` scales the
object before translating it. `move_to` adds a translation; it does not replace
an existing transform. Shapes, text, and textures have cheap shared handles.
Create reusable geometry and libraries outside the frame loop.

## Drawing order and batching

Lower `.order(n)` values draw first. Within the same order, objects draw in
insertion order, so the last translucent object is composited on top. Nested
orders compare lexicographically, with missing components treated as zero.

Only adjacent compatible instances within an order are batched. A, B, A stays
three batches when B has different resources; combining both A objects would
change transparent compositing. `Scene::len()` reports draw batches, not object
count or render-pass count. `Target::render_iter` encodes the batches in one
render pass; clearing is a separate pass.

## Input and timing

`window.input()` creates an independent event stream. `try_next()` drains queued
events without waiting; with a direct `futures` dependency, `StreamExt::next()`
waits asynchronously. Redraw events are handled by the window loop and excluded
from input streams.

The default queue retains 1024 events per consumer. Overflow discards the oldest
event. `set_capacity(None)` makes it unbounded. On termination, consumers wake,
drain buffered events, and then receive `None`. Dropping the event handler also
terminates its streams. An overflow policy that drops events can lose a key
transition; applications that require complete input history should select a
suitable capacity or track focus/state explicitly.

Use elapsed time for animation and `PeriodicTimer` for periodic work. A periodic
timer preserves its deadline phase when ticks are missed; its return value is
the duration of whole elapsed periods, not an integer count. A zero period is
invalid. Background work runs cooperatively on the event-loop thread: blocking
CPU work blocks input and rendering. Awaiting file I/O or timers yields control.

## Tasks, cancellation, and results

`spawn` returns a single-consumer `Task<T>` future. Clone `task.handle()` when
another owner needs to cancel it. Dropping a plain task or its control handle
detaches it; the application waits for its remaining tasks to finish.
Cancellation takes effect at a scheduler boundary and resolves the result to
`Err(Terminated)`. Repeated cancellation, including after completion, is harmless.

Use `cancel_on_drop()` for work owned by a window or scope:

```rust,no_run
async fn background_work() {
    let task = wgame::spawn(async {
        loop { wgame::sleep(std::time::Duration::from_secs(1)).await; }
    }).cancel_on_drop();
    let handle = task.handle();
    handle.terminate();
    let _ = task.await;
}
```

`WindowedTask` likewise has one result consumer and a clonable `handle()` for
cancellation. Cancelling before creation completes resolves without waiting
for an OS resume, and prevents a queued creation from opening a window.
Lower-level `CallOutput` clones share one result and one waiter; they are not a
broadcast channel. Polling a consumed result or completing it twice is a
programming error and panics. A runtime handle is local to its event-loop thread.

## Textures and files

`Library` owns shared graphics helpers. `make_texture` creates a GPU texture from
an image; `load_texture` additionally reads and decodes a file. Native paths are
relative to the process working directory. Web paths are URLs relative to the
page. Use embedded bytes when the same example must work from any directory.
Image decoding currently defaults to PNG. Values are passed through without an
automatic sRGB-to-linear conversion; the default surface prefers a non-sRGB
format. Choose conversions and custom target formats deliberately.

Textures in one atlas can share GPU resources. `Texture::update` and
`update_part` refresh pixels and the one-pixel border used by linear filtering.
Use these methods, rather than mutating the backing `AtlasImage` directly, to
maintain the border. `nearest()` keeps hard texel edges; `linear()` interpolates.
Atlas handles remain valid when an atlas grows and moves its contents.
Create/grow resources before adding objects to a scene: adding an object can
capture its current atlas coordinates. Rebuild a scene from its original objects
after atlas layout changes. Borrowed
image views must not outlive their callback, and callbacks must not reenter the
same atlas. Dimensions must be positive and fit the allocator/device limits.

## Typography

Load a font once, rasterize it for a pixel size, and reuse that raster for labels.
Cache unchanged `Text` objects outside the frame loop. Recreate size-dependent
rasters when appropriate after resizing. `TextAlign` changes the horizontal
origin relative to the measured advance width. Text's default transform is
normalized by font size; multiply by the raster's size for physical-pixel output.

Text uses swash shaping and respects glyph placement offsets. Rasterization
currently supports monochrome outline glyphs. Color emoji/bitmap-only fonts,
font fallback, automatic line wrapping, bidirectional paragraph layout, and
editable-text navigation are not implemented. Use metrics from the same font
when calling `Text::from_metrics`.

## Custom rendering and shader attributes

Use `Graphics::device` and `queue` for direct wgpu access. Implement `Renderer<C>`
to encode custom drawing in a supplied render pass, and `Context` for its shared
bindings. `Target` abstracts a window frame or an `Offscreen` texture target.
`Graphics::new` wraps an existing adapter/device/queue for headless work.
The raw-wgpu example demonstrates constructing a full custom pipeline.

```rust
use wgame::shader::Attribute;
#[derive(wgame::shader::Attribute)]
struct InstanceData {
    transform: wgame::glam::Mat4,
    tint: wgame::glam::Vec4,
}
let layout = InstanceData::bindings().layout(0).unwrap();
assert_eq!(layout.len(), 5); // four matrix columns plus tint
assert_eq!(InstanceData::SIZE, 80);
```

The derive serializes fields in declaration order and derives vertex attribute
locations and offsets. `Mat3` occupies 36 bytes (three `Float32x3` columns).
Arrays concatenate their element attributes; `[f32; 3]` is three scalar
locations, whereas `Vec3` is one vector location. These byte layouts are for
vertex/instance buffers, not a general WGSL uniform-buffer packing scheme.
The `AttributeGlobal` derive selects the `wgame::shader` path; it does not remove
field-name prefixes. Procedural macros currently require the canonical crate
names in the dependency graph.

## Reusing GPU work

Build a `Scene` and call `scene.bake()` to upload immutable instance data once.
Render the resulting `BakedScene` with different cameras without allocating new
instance buffers each frame. Rebuild the source scene from its objects and bake again when object data changes
or an atlas it references grows/rearranges. A baked scene is a snapshot; it does not observe
later scene edits. Grow/populate shared atlases before baking static content.
For dynamic objects, continue to build scenes normally.

The benchmark separates rebuilding with multiple passes, rebuilding with one
pass, and rendering retained snapshots. See the performance guide before
choosing a rendering strategy.
