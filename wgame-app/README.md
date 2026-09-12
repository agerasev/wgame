# wgame-app

A cooperative event-loop runtime with windows, tasks, and timers. Task results have one consumer; clone their cancellation handles. Use `cancel_on_drop()` for work owned by a window. See [lifecycle and task contracts](../docs/GUIDE.md#tasks-cancellation-and-results). Headless regression tests cover scheduling, cancellation, and suspension.
