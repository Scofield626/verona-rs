use crate::executor::Task;
use std::ffi::c_void;
use std::sync::Arc;

use std::sync::atomic::{AtomicBool, Ordering};

static RUNTIME_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[link(name = "verona")]
extern "C" {
    fn runtime_init(threads: usize);
    fn runtime_shutdown();
    fn scheduler_run();
    fn schedule_task(task: *mut c_void);
}

pub fn verona_runtime_init(threads: usize) {
    if !RUNTIME_INITIALIZED.swap(true, Ordering::SeqCst) {
        unsafe {
            runtime_init(threads);
        }
    }
}

pub fn verona_runtime_shutdown() {
    unsafe {
        runtime_shutdown();
    }
}

pub fn verona_scheduler_run() {
    unsafe {
        scheduler_run();
    }
}

pub fn verona_schedule_task(task: Arc<Task>) {
    if !RUNTIME_INITIALIZED.load(Ordering::SeqCst) {
        panic!("Verona runtime not initialized");
    }
    let task_ptr = Arc::into_raw(task) as *mut c_void;
    unsafe {
        schedule_task(task_ptr);
    }
}
