# FFI Bridge Overview

## Current Flow
- Rust entrypoint (`src/main.rs`) spawns futures via `Executor::spawn`, which boxes the future, wraps it in `Arc<Task>`, and hands ownership to Verona using `verona_schedule_task`.
- The C++ shim (`libverona/verona.cc`) exposes `runtime_init`, `scheduler_run`, and `schedule_task` that call into the Verona scheduler. `schedule_task` enqueues a closure that invokes the Rust export `poll_future_in_rust` with the raw task pointer.
- `poll_future_in_rust` reconstructs the `Arc<Task>`, builds a `Context`/`Waker`, and polls the future. `ArcWake::wake_by_ref` re-enqueues the task through `verona_schedule_task`, letting Verona drive wakeups. Completed futures drop naturally when the `Arc` refcount reaches zero.
- Build path: `build.rs` runs CMake in `libverona/` to produce `libverona.a`, which Cargo links; `#[link(name = "verona")]` in `verona_stubs.rs` binds Rust to the shim symbols.

## Why `Mutex` Around the Task Future
- Tasks are shared (`Arc<Task>`) between the scheduler and the waker, and Verona can schedule polling on multiple worker threads. Without synchronization, two workers could poll the same future concurrently, violating Rust’s `&mut Future` exclusivity and causing data races.
- The `Mutex` serializes access so each poll gets exclusive ownership. It’s correctness-focused rather than convenience; removing it would require a provable single-threaded polling invariant.
- If the lock overhead becomes an issue, swap to a lighter mutex (e.g., `parking_lot::Mutex`) or add a “scheduled” flag to coalesce wakes, but keep the one-at-a-time polling guarantee.
