# wgame

A modular framework for building graphics applications.

## Overview

wgame provides a layered architecture for creating cross-platform graphics applications:

- **Application**: Window management, async runtime, and application lifecycle
- **Graphics**: GPU rendering abstractions, scene management, and camera support
- **Shapes**: Geometry primitives with fill, stroke, and texture support
- **Typography**: Font rasterization and text rendering
- **Utilities**: Cross-platform file I/O, input handling, and timing utilities

## Crates

- [`wgame`](wgame/) - Main crate
- [`wgame-app`](wgame-app/) - Application entry points, window management, async application runtime and executor
- [`wgame-gfx`](wgame-gfx/) - GPU rendering framework with scene management
- [`wgame-gfx-shapes`](wgame-gfx-shapes/) - Geometry shape rendering
- [`wgame-gfx-texture`](wgame-gfx-texture/) - Texture handling and atlas management
- [`wgame-gfx-typography`](wgame-gfx-typography/) - Text rendering
- [`wgame-image`](wgame-image/) - Image processing and texture atlas utilities
- [`wgame-typography`](wgame-typography/) - Font rasterization and text metrics
- [`wgame-shader`](wgame-shader/) - Shader utilities for bridging Rust types with GPU shaders
- [`wgame-shader-macros`](wgame-shader-macros/) - Procedural macros for deriving shader attributes
- [`wgame-macros`](wgame-macros/) - Macros for application and window entry points
- [`wgame-app-input`](wgame-app-input/) - Input event multiplexer for window events
- [`wgame-fs`](wgame-fs/) - Cross-platform file reading utilities
- [`wgame-utils`](wgame-utils/) - Utility types and functions

## Getting Started

See [`wgame-examples`](wgame-examples/) for usage examples.

```rust
use wgame_macros::window;

#[window(width = 800, height = 600)]
fn main() {
    // Your application logic here
}
```

## Platform Support

- **Desktop**: Windows, macOS, Linux via winit and wgpu
- **Web**: WebAssembly via wasm-bindgen and WebGPU
- Any other supported by WGPU and Winit, but not tested

## License

MIT
