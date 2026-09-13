# Working on wgame

These instructions apply to the entire workspace. Read applicable nested
`AGENTS.md` files and explicitly required references fully before changing code.

## Design and documentation

- Prefer cohesive modules, single-purpose abstractions, and type-level contracts.
  Share naturally common behavior; allow independently evolving behavior to stay
  separate. Expose only what consumers need and preserve CPU/GPU crate boundaries.
- Keep one authoritative home per fact. [README.md](README.md) introduces the
  workspace; current usage and contracts live in crate/module/API doc-comments.
  Read the relevant docs before changing behavior and update them with examples.
  Report breaking changes with the work and in release notes when releasing.
  Do not add historical roadmaps, session reports, or benchmark output to the repo.
- Keep user documentation focused on purpose, typical usage, and consequential
  contracts. Put developer guidance at its narrowest useful scope. Link to
  existing explanations instead of duplicating API inventories or volatile counts.
- Comments explain non-obvious reasons and cross-module invariants. Remove
  narration and obsolete history; keep prose wrapped to the surrounding style.
- Preserve hard-won invariants: executor callbacks and future destruction can
  reenter runtime state, so release `RefCell` borrows before invoking them.
  Review the owning docs for [scene ordering](wgame-gfx/src/scene.rs),
  [atlas generations](wgame-image/src/atlas.rs), [frames](wgame/src/window.rs), and
  [shader layouts](wgame-shader/src/attribute.rs) when touching those paths.

## Rust

- Follow existing Rust style and preserve `forbid(unsafe_code)` boundaries.
  Use the project's stable `cargo fmt --all`; nightly formatting is not required.
- Inherit edition and shared dependencies from the workspace manifest. Declare
  member path dependencies there and keep `Cargo.lock` consistent with changes.
- Use typed errors at meaningful recoverable boundaries (`thiserror` where
  appropriate). Avoid unchecked unwraps on runtime data; explicit assertions are
  appropriate for established internal invariants.

## Verification

- Keep focused unit tests beside implementations in `src/`, with test-only
  imports inside test modules. Split large test modules into sibling files.
  Use integration tests for cross-module contracts, remove redundant coverage,
  and assert error kinds when available.
- Add regression tests for behavioral fixes. Prefer CPU tests for lifecycle,
  allocation, and ordering logic; use offscreen pixel tests for rendering.
  Macro changes need consumer compile coverage; see
  [entry-point tests](wgame/tests/entry_points.rs).
- Run affected crate tests and Clippy with warnings denied from the workspace
  root. For broad changes, follow [CI](.github/workflows/ci.yml), including
  [the feature matrix](scripts/check-features.sh). Prose-only edits need
  link/content and whitespace checks, not builds. Changed Rust doc examples need
  doctests (`cargo test --workspace --locked --doc`); changed rustdoc links need
  a documentation build with warnings denied, as configured in CI.
- Graphics changes require explicitly running the ignored GPU tests; missing
  adapters must fail that run. Window/runtime integration changes also require
  the playground smoke check. See [rendering test docs](wgame/tests/rendering.rs)
  and [example instructions](wgame-examples/README.md) for Mesa/Xvfb commands.
- For web changes, check generated Trunk artifacts and distinguish that from
  browser execution. For performance changes, run the benchmark described in
  [its source](wgame/examples/render_bench.rs) and report the adapter and method.
  Report actual results and platform gaps with the work; distinguish compilation,
  generated web artifacts, browser execution, offscreen tests, and window tests.
  Software-renderer measurements do not establish hardware frame rates.
- Bound potentially hanging GPU/window commands with timeouts and bound polling
  loops. Preserve verification exit status; capture logs before filtering them,
  rather than piping a running producer into a filter that may terminate it.
- Search relevant subtrees with ignore-aware tools such as `rg`. Avoid broad walks
  through build/cache directories. Check available disk space before large builds
  and clean only targeted artifacts when needed.

## Platform pitfalls

- Do not use `--all-features`: native and web runtime features are mutually
  exclusive. Preserve independently usable optional features. Standalone
  `wgame-app` checks need an appropriate platform backend; the feature matrix
  requires the `wasm32-unknown-unknown` target.
- The high-level web feature uses WebGL2. Older asset-loading examples require
  `wgame-examples/` as the working directory; the playground embeds its assets.
  Use [example instructions](wgame-examples/README.md) for exact launch commands.

## Collaboration

- During design discussions, explain examples and material tradeoffs before
  editing. Wait for explicit implementation instructions; earlier broad
  authorization does not carry into a new design discussion or authorize commits
  there. Resolve consequential ambiguity with the user and record the decision
  in its owning document.
- Commit verified implementation milestones when authorized, checking for foreign
  changes and staging only your work. Prefer additive commits or reverts; history
  rewriting requires explicit approval of the concrete rewrite and consequences.
- If foreign work blocks verification, use an isolated checkout/worktree without
  modifying that work.
