# Stabilization roadmap

Implemented against `40c70e6`, incorporating the updated checkout rather than
reapplying the already-fixed web import. The six implementation stages are done.
Platform validation limits remain explicit below.

- [x] **1. Build baseline.** Fixed entry/derive macros and optional helper gates;
  compiled the quickstart; added native CI and 34 feature checks.
- [x] **2. Lifecycle.** Fixed suspension, cancellation (including queued tasks),
  destructor reentrancy, input wakeups, initial/zero-size redraw state, and timer
  phase. Added headless lifecycle regressions and separate cancellation handles.
- [x] **3. Rendering correctness.** Established stable painter order, fixed Mat3
  byte layouts and partial texture updates, covered atlas growth/resize and text
  rendering, and classified recoverable surface errors. Added four GPU tests.
- [x] **4. API and application.** Added explicit render/present/discard boundaries,
  scoped tasks, offscreen targets, and the interactive playground. Verified a
  twelve-frame window smoke test under Xvfb.
- [x] **5. Performance.** Added four reproducible workloads, recorded release
  measurements, consolidated scene passes, and added reusable baked renderers.
  Pixel comparisons verify retained and rebuilt rendering agree.
- [x] **6. Documentation.** Rewrote onboarding and crate summaries, added a compiled
  usage guide, migration notes, validation commands, and performance results.

## Final local validation

- 34 ordinary unit/integration/documentation tests passed.
- All four explicitly selected GPU tests passed on Mesa llvmpipe Vulkan.
- All 32 desktop optional-feature combinations and two web configurations passed.
- Formatting and Clippy (`-D warnings`) passed.
- Native playground smoke test and full Trunk build passed.

See [validation](VALIDATION.md) for repeatable commands and coverage boundaries.
Hosted CI, Windows/macOS execution, real-display suspend/resume and HiDPI tests,
and browser runtime testing remain external platform checks; they are not
claimed as local passes. No browser is exposed to this session's automation.

See [performance](PERFORMANCE.md) for the recorded baseline. Dynamic buffer
pooling is a possible subsequent optimization, subject to measurements; retained
snapshots already eliminate repeated instance uploads for unchanged content.
