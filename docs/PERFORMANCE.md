# Rendering performance

## Reproduce

```sh
cargo run --locked -p wgame --example render_bench --release
```

The benchmark prints adapter information to stderr and CSV to stdout. It runs
at 256×256 with ten warmup frames and forty measured frames per workload/mode.
CPU time includes scene construction, buffer creation, encoding, and queue
submission. End-to-end time also waits for the device to complete that frame.
This is deliberately synchronized, not a pipelined throughput/FPS benchmark.

Modes isolate the rendering changes using the same painter-ordered scene:

- `multipass`: rebuild instances and encode each batch in its own pass (the
  previous pass strategy, with the corrected ordering contract).
- `singlepass`: rebuild instances and encode all batches in one pass.
- `retained`: bake once, then reuse immutable instance buffers in one pass.
  This mode is omitted for changing text because freezing that text would change
  the workload.

The reported pass count includes clearing. Instance-buffer counts are derived
from the renderer's one-buffer-per-batch code path, not global allocation
instrumentation. This benchmark does not measure all CPU allocations, GPU
hardware timestamps, or peak memory.

## Recorded baseline

Recorded on Linux x86-64, rustc 1.98.0, release profile, Mesa llvmpipe Vulkan
(LLVM 15.0.7, Mesa 23.2.1), on 2026-09-12. This is a **software GPU** on a shared
machine, so timings are illustrative rather than hardware performance claims.
[Raw CSV](benchmarks/llvmpipe.csv) preserves the complete run.

| Workload | Batches | Multipass total | Single-pass total | Retained total |
| --- | ---: | ---: | ---: | ---: |
| 1,000 identical shapes | 1 | 1.40 ms | 1.39 ms | 0.59 ms |
| 1,000 alternating shapes | 1,000 | 72.62 ms | 20.74 ms | 9.73 ms |
| 1,000 overlapping translucent shapes | 1,000 | 72.08 ms | 21.22 ms | 11.53 ms |
| 100 changing text objects | 1 | 1.08 ms | 1.08 ms | — |

For the mixed workload, single-pass encoding reduces passes from 1,001 to 2,
including clearing. Retaining the scene additionally removes 1,000 instance
buffer creations per frame. Identical shapes already use a single batch, so
consolidating passes should not materially improve that workload.

## Choosing a strategy

Use normal scenes for changing content. Keep geometry, textures, font rasters,
and unchanged text outside the frame loop. Use `Scene::bake()` for static content;
rebuild the source scene and snapshot after changing objects or growing/repacking
referenced atlases.
The offscreen regression tests compare retained and rebuilt pixels to ensure
that the optimization preserves compositing.

The immediate path still creates an instance buffer per batch per frame. A
persistent dynamic buffer allocator may help workloads with many changing
batches, but requires a separate measured design and lifetime/synchronization
contract. It is not necessary for unchanged content, which can use baked scenes.
