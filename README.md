# wgame

A modular Rust framework for 2D graphics applications, built on winit and wgpu.
It provides an async window loop, composable shapes, texture atlases, and text.

## Run an example

From the repository root:

```sh
cargo run -p wgame-examples --bin playground
```

Move the mouse to move the ring; press Space to pause animation. The example
embeds its assets, handles resizing, and runs a background task that is cancelled
when the window ends. `-- --smoke` exits after twelve frames on desktop.

The [examples guide](wgame-examples/README.md) includes desktop and web commands.

## Use the library

Until a release is published, point your application's Cargo.toml at the `wgame`
crate inside this checkout:

```toml
[dependencies]
wgame = { path = "/path/to/checkout/wgame" }
```

Put this in `src/main.rs`:

```rust,no_run
use wgame::{Window, prelude::*, gfx::types::color};

#[wgame::window(size = (800, 600), title = "Hello wgame")]
async fn main(mut window: Window<'_>) -> wgame::Result<()> {
    while let Some(mut frame) = window.next_frame().await? {
        frame.clear(color::BLACK);
        frame.present();
    }
    Ok(())
}
```

`next_frame` returns `None` when the window closes. Present explicitly, or let
normal scope exit present the frame. `frame.discard()` drops unfinished work.

## Learn and contribute

- [Usage guide](docs/GUIDE.md): coordinates, input, timing, assets, text, ordering,
  custom rendering, and lifecycle contracts.
- [Migration notes](docs/MIGRATION.md): API and behavior changes in stabilization.
- [Validation](docs/VALIDATION.md): supported build matrix, tests, and limitations.
- [Performance](docs/PERFORMANCE.md): benchmark workloads and results.
- [Roadmap](docs/ROADMAP.md): implementation progress.

## Crates

| Layer | Crates |
| --- | --- |
| Application | `wgame`, `wgame-app`, `wgame-app-input`, `wgame-macros` |
| Rendering | `wgame-gfx`, `wgame-gfx-shapes`, `wgame-gfx-texture`, `wgame-gfx-typography` |
| CPU resources | `wgame-image`, `wgame-typography`, `wgame-fs` |
| Shader attributes | `wgame-shader`, `wgame-shader-macros` |
| Timing helpers and examples | `wgame-utils`, `wgame-examples` |

## Platform and feature support

Default features select `desktop`, `shapes`, `fs`, `image`, `typography`, and
`utils`. Each optional content feature can be enabled independently.
`Library::load_texture` needs both `fs` and `image`; `Library::load_font` needs
both `fs` and `typography`.

The `desktop` feature enables native windowing and Vulkan/GLES/Metal/DX12
backends as appropriate for the target. The `web` feature selects WebGL2;
disable default features when selecting it. Desktop and web cannot be enabled
together. WebGPU is available at the lower `wgame-gfx` layer but is not the
high-level `web` feature's backend.

Linux is locally tested, including Mesa software Vulkan rendering and an Xvfb
window smoke test. CI is configured for Linux, Windows, and macOS compilation
and tests. WebAssembly compilation is checked separately; browser and other
platform limitations are recorded in the validation guide.

## License

MIT. Example font and image assets are used only by examples and tests.
