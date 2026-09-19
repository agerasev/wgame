# Browser gallery

The dependency-free landing page links to the browser examples and games built
with wgame. Example behavior and controls live in the
[example guide](../wgame-examples/README.md).

Install the wasm target and Trunk as described in that guide. The build script
also uses Bash and GNU `realpath`. Build from the repository root:

```sh
./scripts/build-pages.sh /wgame/
```

The script builds each example in release mode with the locked dependencies,
WebGL2, and size optimization. It stages the landing page and complete Trunk
artifacts under `target/pages/`, including `.nojekyll`. Temporary canvas wrappers
stay outside the checkout. Existing output is replaced only after every build
succeeds; a nonempty destination must contain the script's `.wgame-pages` marker.

The second argument selects an output directory. `TRUNK` may select a Trunk
binary; the script preserves `PATH`, `CARGO_TARGET_DIR`, and Trunk's tool cache
and version environment variables. For example:

```sh
./scripts/build-pages.sh / /tmp/wgame-gallery
python3 -m http.server --directory /tmp/wgame-gallery 8080
```

Open `http://localhost:8080/` to inspect the gallery and run the demos. Artifact
generation alone does not verify browser execution.

To publish at `https://agerasev.github.io/wgame/`, build with `/wgame/` and copy the
contents of `target/pages/` to the root of the `gh-pages` branch. Preserve any
existing deployment history with an ordinary commit, then push that branch.
GitHub Pages must serve `gh-pages` from `/`.
