use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};

/// A smart pointer to a shared instance of type `T`.
///
/// The inner `T` can be borrowed immutably with `.borrow()` and mutably with
/// `.borrow_mut()`. The implementation guarantees the usual reference
/// guarantees.
#[derive(Debug)]
pub struct Shared<T>(Arc<Mutex<T>>);

impl<T> Shared<T> {
    /// Wraps `val` into a `Shared<T>` and returns it.
    pub fn new(val: T) -> Self {
        Shared(Arc::new(Mutex::new(val)))
    }

    /// Returns an immutable borrow to the inner value.
    ///
    /// If the inner value is presently mutably borrowed, this function blocks
    /// until that borrow is returned.
    pub fn borrow<'a>(&'a self) -> impl Deref<Target = T> + 'a {
        self.0.lock().expect("all okay")
    }

    /// Returns an mutable borrow to the inner value.
    ///
    /// If the inner value is presently borrowed, mutably or immutably, this
    /// function blocks until all borrows are returned.
    pub fn borrow_mut<'a>(&'a self) -> impl DerefMut<Target = T> + 'a {
        self.0.lock().expect("all okay")
    }
}

impl<T> Clone for Shared<T> {
    /// Returns a copy of the shared pointer.
    ///
    /// The value `T` itself is not copied; only the metadata associated with
    /// the smart pointer required for accurate book-keeping is copied.
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}
