use crate::executor::Task;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static RUNTIME_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type VeronaTask;
        fn poll_future(task: Box<VeronaTask>);
    }

    unsafe extern "C++" {
        include!("libverona/verona_bridge.h");

        fn runtime_init(threads: usize);
        fn runtime_shutdown();
        fn scheduler_run();
        fn schedule_task(task: Box<VeronaTask>);
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
