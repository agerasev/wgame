# wgame

A modular Rust framework for async 2D/3D graphics applications, built on winit and
wgpu. It provides an async window loop, composable shapes, texture atlases, and text.

## Get started

Build and open the [facade crate documentation](wgame/src/lib.rs) for the
quickstart, feature selection, and links to API contracts:

```sh
cargo doc --locked --workspace --no-deps --open
```

Run the self-contained playground:

```sh
cargo run --locked -p wgame-examples --bin playground
```

See [example instructions](wgame-examples/README.md) for controls and desktop/web
launch commands, and [AGENTS.md](AGENTS.md) for contributor practices and checks.

The optional [egui wrapper](wgame-egui/src/lib.rs) adds controls around a drawing
area while keeping content code generic over the [window host](wgame/src/host.rs).
Try `cargo run -p wgame-egui --example playground` (or add `-- --plain`).

## Crates

| Layer | Crates |
| --- | --- |
| Application | `wgame`, `wgame-app`, `wgame-app-input`, `wgame-macros` |
| Rendering | `wgame-gfx`, `wgame-gfx-shapes`, `wgame-gfx-texture`, `wgame-gfx-typography` |
| Canvas input and UI | `wgame-input`, `wgame-egui` |
| CPU resources | `wgame-image`, `wgame-typography`, `wgame-fs` |
| Shader attributes | `wgame-shader`, `wgame-shader-macros` |
| Timing helpers and examples | `wgame-utils`, `wgame-examples` |

The optional `3d` facade feature adds [solid primitives and lighting](wgame-gfx-3d/README.md)
to the shared shape renderer. Cameras, textures, meshes, materials and scenes are
common to 2D and 3D; targets provide depth by default.

[CPU textures and GPU render textures](wgame-gfx-texture/src/lib.rs) share sampling
in shapes and materials. Render into previews using the ordinary target API and
download detached CPU snapshots when needed.

## License

MIT. Example font and image assets are used only by examples and tests.
