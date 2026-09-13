# wgame

A modular Rust framework for async 2D graphics applications, built on winit and
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

## Crates

| Layer | Crates |
| --- | --- |
| Application | `wgame`, `wgame-app`, `wgame-app-input`, `wgame-macros` |
| Rendering | `wgame-gfx`, `wgame-gfx-shapes`, `wgame-gfx-texture`, `wgame-gfx-typography` |
| CPU resources | `wgame-image`, `wgame-typography`, `wgame-fs` |
| Shader attributes | `wgame-shader`, `wgame-shader-macros` |
| Timing helpers and examples | `wgame-utils`, `wgame-examples` |

## License

MIT. Example font and image assets are used only by examples and tests.
