use std::any::Any;
use std::marker::PhantomData;

/// A concurrent owner (cown) that holds data of type T
///
/// Cowns provide safe concurrent access to mutable data through the `when()` API.
/// Multiple cowns can be acquired atomically to prevent deadlocks.
pub struct Cown<T: Send + 'static> {
    inner: cxx::UniquePtr<crate::verona_stubs::ffi::VoidCown>,
    _phantom: PhantomData<T>,
}

impl<T: Send + 'static> Cown<T> {
    /// Create a new cown containing the given value
    ///
    /// # Example
    /// ```
    /// let counter = Cown::new(0i32);
    /// let message = Cown::new(String::from("Hello"));
    /// ```
    pub fn new(value: T) -> Self {
        // Wrap the value in Box<dyn Any + Send> for type erasure
        let boxed: Box<dyn Any + Send> = Box::new(value);
        // Convert to raw pointer and then to usize for FFI
        let raw_ptr = Box::into_raw(boxed) as *mut () as usize;
        let inner = crate::verona_stubs::ffi::make_cown_any(raw_ptr);
        Cown {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T: Send + 'static> Clone for Cown<T> {
    fn clone(&self) -> Self {
        Cown {
            inner: crate::verona_stubs::ffi::cown_clone(&self.inner),
            _phantom: PhantomData,
        }
    }
}

impl<T: Send + 'static> Drop for Cown<T> {
    fn drop(&mut self) {
        // UniquePtr handles cleanup automatically via cxx
    }
}

// Safety: Cown<T> is thread-safe because:
// - The underlying cown_ptr<void*> is reference-counted and thread-safe
// - The Verona runtime handles all synchronization
// - T is required to be Send
unsafe impl<T: Send + 'static> Send for Cown<T> {}

// Safety: Cown<T> can be shared between threads because the reference counting
// is atomic and the Verona runtime ensures safe concurrent access
unsafe impl<T: Send + 'static> Sync for Cown<T> {}
