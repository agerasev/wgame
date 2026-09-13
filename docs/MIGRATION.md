# Stabilization migration

Changes relative to `40c70e6`:

- Replace `task.clone()` with `task.handle()` (clone that handle when needed).
  `Task` and `WindowedTask` have one result consumer. Use `cancel_on_drop()` for
  tasks that should end with their owning window/scope.
- Equal-order objects now preserve insertion order. An interleaved A/B/A
  resource sequence no longer batches the two A objects together. Use explicit
  layers only when changing compositing order is intentional.
- `Scene::len()` counts draw batches. They share one render pass through
  `Target::render_iter`; existing pass-count displays should say “batches”.
- `scene.render()`, `frame.present()`, and `frame.discard()` provide explicit
  lifecycle boundaries. Normal drop keeps the previous automatic behavior;
  unwinding no longer submits unfinished frames/scenes.
- Replace the misspelled `tranform_texcoord` with `transform_texcoord`. The old
  spelling remains as a deprecated forwarding method.
- `#[app]` now works with an async, zero-argument entry function. Window options
  use `size = (800, 600)`; there are no separate width/height builder methods.
- Input consumers wake on termination and drain buffered events before ending.
- Periodic timers count whole missed periods without shifting their phase.
- Optional feature helpers are gated explicitly: `load_texture` needs `fs` and
  `image`; `load_font` needs `fs` and `typography`.
- Examples no longer dump atlas images by default. Add `--features dump` when
  running the shapes example from `wgame-examples/` to request those files.
- Typography explicitly uses monochrome outlines. Color bitmap/outline sources
  are not interpreted as single-channel masks.

New capabilities: an offscreen render target, immutable baked scenes, a
representative playground example, headless regression tests, and a benchmark.

## Append-only atlas generations

Atlas drop/resize no longer returns rectangles to the current allocator. Dead
space is reclaimed by repacking live items into a replacement generation. With
live area (including the pending allocation and padding) at most half the atlas
area, same-size packing is attempted before growth. GPU mirrors must compare
`Atlas::generation()`, not only dimensions, and fully upload each replacement.

Built-in shape/text scenes resolve UVs when baking. Earlier documentation saying
that `Scene::add` froze atlas coordinates was incorrect: rebaking an existing
scene follows relocation. Baked/encoded drawing now safely retains old contents
across source drop/resize and atlas replacement. Explicit pixel updates remain
mutable within the GPU generation being updated.
