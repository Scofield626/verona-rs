use std::marker::PhantomData;
use std::sync::Mutex;

/// A concurrent owner (cown) that holds data of type T
///
/// Cowns provide safe concurrent access to mutable data through the `when()` API.
/// Multiple cowns can be acquired atomically to prevent deadlocks.
pub struct Cown<T: Send + 'static> {
    inner: cxx::UniquePtr<crate::verona_stubs::ffi::VoidCown>,
    // PhantomData<Mutex<T>> correctly models that we own T and provide
    // mutual exclusion (like a Mutex), so we are Send/Sync if T is Send.
    // However, since UniquePtr<VoidCown> is not automatically Send, we need
    // unsafe impls below. This marker is still useful for documentation and
    // variance (though T is invariant in Mutex).
    _marker: PhantomData<Mutex<T>>,
}

/// Trampoline function to drop the Rust data when the C++ cown is destroyed.
extern "C" fn drop_trampoline<T>(ptr: usize) {
    unsafe {
        // Reconstruct the Box and let it drop
        let _ = Box::from_raw(ptr as *mut T);
    }
}

impl<T: Send + 'static> Cown<T> {
    /// Create a new cown containing the given value
    ///
    /// # Example
    /// ```
    /// use verust::cown::Cown;
    /// let counter = Cown::new(0i32);
    /// let message = Cown::new(String::from("Hello"));
    /// ```
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        let raw_ptr = Box::into_raw(boxed) as usize;

        let inner =
            unsafe { crate::verona_stubs::ffi::make_cown(raw_ptr, drop_trampoline::<T> as usize) };

        Cown {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<T: Send + 'static> Clone for Cown<T> {
    fn clone(&self) -> Self {
        Cown {
            inner: crate::verona_stubs::ffi::cown_clone(&self.inner),
            _marker: PhantomData,
        }
    }
}

impl<T: Send + 'static> Drop for Cown<T> {
    fn drop(&mut self) {
        // UniquePtr handles cleanup automatically via cxx,
        // which triggers ~VoidCown -> ~RustBox -> drop_trampoline
    }
}

// Safety: Cown<T> is thread-safe because:
// - The underlying cown_ptr is reference-counted and thread-safe
// - The Verona runtime handles all synchronization
// - T is required to be Send
unsafe impl<T: Send + 'static> Send for Cown<T> {}

// Safety: Cown<T> can be shared between threads because the reference counting
// is atomic and the Verona runtime ensures safe concurrent access
unsafe impl<T: Send + 'static> Sync for Cown<T> {}
