# Implementation Plan: Verona Cowns in Rust

This plan outlines the steps to implement the `Cown<T>` abstraction in `verona-rs`, incorporating best practices from the `boxcars` analysis and leveraging `cxx` for a robust FFI bridge.

## Design Philosophy

-   **Type Safety**: Use `Box<T>` and generics instead of `Box<dyn Any>` to ensure compile-time type safety and avoid runtime downcasting.
-   **Memory Safety**: Use `PhantomData<Mutex<T>>` to correctly model thread-safety guarantees.
-   **Performance**: Use thin pointers (`Box<T>`) and minimize allocations where possible.
-   **Integration**: Leverage `cxx` for the bridge, allowing future expansion to shared Rust/C++ types.

## Phase 1: Core Cown Abstraction

**Goal**: Create `Cown<T>` that can hold any `Send` Rust type, can be cloned (ref-counted), and correctly drops the Rust data when the last reference is lost.

### 1. Rust Side (`src/cown.rs`)
-   Define `Cown<T>`:
    ```rust
    pub struct Cown<T: Send + 'static> {
        inner: cxx::UniquePtr<ffi::VoidCown>,
        _marker: std::marker::PhantomData<std::sync::Mutex<T>>,
    }
    ```
-   Implement `new(value: T)`:
    -   Box the value: `Box::new(value)`.
    -   Get raw pointer: `Box::into_raw`.
    -   Define a generic destructor trampoline: `extern "C" fn drop_trampoline<T>(ptr: usize)`.
    -   Call FFI: `ffi::make_cown(ptr, drop_trampoline::<T>)`.
-   Implement `Clone` using `ffi::cown_clone`.
-   Implement `Drop` (relying on `UniquePtr` to trigger C++ side cleanup).

### 2. C++ Side (`libverona/verona.cc` & `.h`)
-   Define `VoidCown` struct to hold:
    -   `verona::cpp::cown_ptr<void> ptr` (The Verona cown).
    -   `void (*dtor)(size_t)` (Function pointer to Rust destructor).
-   Implement `make_cown`:
    -   Create a custom C++ class `RustObject` that inherits from `verona::rt::VCown<RustObject>`.
    -   Store the Rust pointer and dtor in this class.
    -   In the C++ destructor of `RustObject`, call the Rust `dtor`.
    -   *Correction*: `verona::cpp::make_cown` creates a `cown_ptr`. We need to attach the cleanup logic to the *data* inside the cown.
    -   **Refined Approach**:
        -   Define `struct RustBox { size_t ptr; void (*dtor)(size_t); ~RustBox() { dtor(ptr); } }`.
        -   `make_cown` creates a `cown_ptr<RustBox>`.
        -   When the cown is collected, `~RustBox` runs, calling the Rust dtor.

### 3. FFI Bridge (`src/verona_stubs.rs`)
-   Update `#[cxx::bridge]` to expose `make_cown` and `cown_clone`.
-   Define `VoidCown` as an opaque C++ type.

## Phase 2: `when` and Callbacks

**Goal**: Allow executing Rust closures on Cowns.

### 1. Trampoline Mechanism (Inspired by `boxcars`)
-   Instead of boxing the callback on the heap for every call, we can pass the closure directly if it's `Copy`, or box it if it captures state.
-   For Phase 1/2 start, we will stick to `Box<FnOnce>` for simplicity, but implement the *trampoline* pattern to execute it.
-   **Rust**:
    -   `when<F>(cown, f)`:
        -   Box the closure `f`.
        -   Pass raw pointer to closure + `invoke_trampoline` function to C++.
-   **C++**:
    -   `when(cown) << [closure_ptr, invoke_fn](auto) { invoke_fn(closure_ptr, cown_data); }`
    -   This ensures the closure runs on the Verona thread.

### 2. Accessing Data
-   The trampoline function needs to:
    1.  Cast the `cown_data` (void*) back to `*mut T`.
    2.  Cast the `closure_ptr` back to `Box<F>`.
    3.  Call the closure with `&mut T`.

## Phase 3: Verification & Async Integration

### 1. Tests
-   **Basic**: Create `Cown<i32>`, clone it, drop it. Verify destructor runs (using a side-effect like `Drop` flag).
-   **Concurrency**: Run `when` on multiple threads.
-   **Async**: Spawn an async task that awaits a timer, then accesses a Cown.

### 2. Async Integration
-   Ensure `when` callbacks run on the same scheduler as the `Executor`.
-   (Already confirmed: `verona-rs` uses the global scheduler, so this should work "out of the box").

## Step-by-Step Tasks

1.  [ ] **Setup C++ Infrastructure**: Define `RustBox` and `VoidCown` in `libverona`.
2.  [ ] **Implement Rust `Cown<T>`**: Create `src/cown.rs` with `Box<T>` logic.
3.  [ ] **Update FFI**: Add `make_cown` to `verona_stubs.rs`.
4.  [ ] **Test Memory Management**: Verify `Drop` works correctly across FFI.
5.  [ ] **Implement `when`**: Add single-cown `when` support.
6.  [ ] **Implement `when2`**: Add two-cown `when` support.
