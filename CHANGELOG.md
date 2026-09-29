# Release notes

## 0.1.0

First crates.io release of the modular wgame framework and optional egui host.

- Async windows, cooperative tasks, cancellation, timers, and canvas input.
- Retained 2D/3D scenes, texture atlases and render textures, typography, and
  programmable shapes with lighting and masked materials.
- Desktop backends and WebGL2 browser applications; select one runtime backend.
- Independently packaged libraries, examples, and regression-test fixtures.

Image slices support excluded starting bounds and empty ranges at image edges.
Invalid or overflowing bounds panic consistently in debug and optimized builds.

The examples package is maintained in the repository and is not published.
Example assets have separate notices in `wgame-examples/assets/README.md`.
