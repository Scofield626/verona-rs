use crate::verona_stubs;
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    ffi::c_void,
    future::Future,
    sync::{Arc, Mutex},
    task::Context,
};

#[derive(Clone, Copy)]
pub struct Executor {}

pub struct Task {
    future: Mutex<BoxFuture<'static, ()>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // FIXME: This creates a new task from a verona perspective
        verona_stubs::verona_schedule_task(arc_self.clone());
    }
}

impl Executor {
    pub fn new(threads: usize) -> Executor {
        verona_stubs::verona_runtime_init(threads);
        Executor {}
    }

    pub fn shutdown(&self) {
        verona_stubs::verona_runtime_shutdown();
    }

    pub fn run(&self) {
        verona_stubs::verona_scheduler_run();
    }

    pub fn spawn(future: impl Future<Output = ()> + 'static + Send) {
        let boxed_future = future.boxed();
        let boxed_task = Arc::new(Task {
            future: Mutex::new(boxed_future),
        });
        verona_stubs::verona_schedule_task(boxed_task);
    }
}

#[no_mangle]
pub extern "C" fn poll_future_in_rust(task: *mut c_void) {
    let result = std::panic::catch_unwind(|| unsafe {
        let raw_ptr = task as *mut Task;
        let boxed_task = Arc::from_raw(raw_ptr);

        let is_pending = {
            let mut boxed_future = boxed_task.future.lock().unwrap();
            let waker = waker_ref(&boxed_task);
            let context = &mut Context::from_waker(&waker);
            boxed_future.as_mut().poll(context).is_pending()
        };

        if is_pending {
            std::mem::forget(boxed_task);
            println!("Task is not finished yet");
        } else {
            println!("Task is finished");
        }
    });

    if result.is_err() {
        eprintln!("Panic caught in poll_future_in_rust");
    }
}
