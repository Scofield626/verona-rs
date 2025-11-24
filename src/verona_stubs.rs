use crate::executor::Task;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static RUNTIME_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type VeronaTask;
        fn poll_future(task: Box<VeronaTask>);

        // Cown callback handling
        type CownCallback;
        fn invoke_cown_callback(cb: Box<CownCallback>, data_ptr: usize);
    }

    unsafe extern "C++" {
        include!("libverona/verona_bridge.h");

        // Runtime functions
        fn runtime_init(threads: usize);
        fn runtime_shutdown();
        fn scheduler_run();
        fn schedule_task(task: Box<VeronaTask>);

        // Cown operations - type-erased via void*
        type VoidCown;

        /// Create a cown holding arbitrary Rust data (passed as usize pointer)
        fn make_cown_any(data_ptr: usize) -> UniquePtr<VoidCown>;

        /// Clone a cown reference (increments reference count)
        fn cown_clone(cown: &VoidCown) -> UniquePtr<VoidCown>;

        /// Schedule a when() clause on a single cown
        fn when_cown(cown: &VoidCown, callback: Box<CownCallback>);

        /// Schedule a when() clause on two cowns
        fn when_cown2(cown1: &VoidCown, cown2: &VoidCown, callback: Box<CownCallback>);
    }
}

pub struct VeronaTask {
    pub task: Arc<Task>,
}

pub fn verona_runtime_init(threads: usize) {
    if !RUNTIME_INITIALIZED.swap(true, Ordering::SeqCst) {
        ffi::runtime_init(threads);
    }
}

pub fn verona_runtime_shutdown() {
    ffi::runtime_shutdown();
}

pub fn verona_scheduler_run() {
    ffi::scheduler_run();
}

pub fn verona_schedule_task(task: Arc<Task>) {
    if !RUNTIME_INITIALIZED.load(Ordering::SeqCst) {
        panic!("Verona runtime not initialized");
    }
    let verona_task = Box::new(VeronaTask { task });
    ffi::schedule_task(verona_task);
}

#[allow(clippy::boxed_local)]
pub fn poll_future(task: Box<VeronaTask>) {
    crate::executor::poll_future_in_rust(task.task.clone());
}

// Cown callback support
pub struct CownCallback {
    // Placeholder for now - will be filled in Phase 2
}

#[allow(clippy::boxed_local)]
pub fn invoke_cown_callback(_cb: Box<CownCallback>, _data_ptr: usize) {
    // Placeholder for Phase 2 when() implementation
    // This will invoke the user's closure with the acquired cown data
    eprintln!("invoke_cown_callback called (not yet implemented)");
}
