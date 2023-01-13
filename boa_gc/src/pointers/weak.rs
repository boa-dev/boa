use crate::{Ephemeron, Finalize, Gc, Trace};

/// A weak reference to a [`Gc`].
///
/// This type allows keeping references to [`Gc`] managed values without keeping them alive for
/// garbage collections. However, this also means [`WeakGc::upgrade`] could return `None` at any moment.
#[derive(Debug, Trace, Finalize)]
#[repr(transparent)]
pub struct WeakGc<T: Trace + ?Sized + 'static> {
    inner: Ephemeron<T, Gc<T>>,
}

impl<T: Trace> WeakGc<T> {
    /// Creates a new weak pointer for a garbage collected value.
    pub fn new(value: &Gc<T>) -> Self {
        Self {
            inner: Ephemeron::new(value, value.clone()),
        }
    }

    /// Upgrade returns a `Gc` pointer for the internal value if valid, or None if the value was already garbage collected.
    pub fn upgrade(&self) -> Option<Gc<T>> {
        self.inner.value()
    }
}

impl<T: Trace> Clone for WeakGc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Trace> From<Ephemeron<T, Gc<T>>> for WeakGc<T> {
    fn from(inner: Ephemeron<T, Gc<T>>) -> Self {
        Self { inner }
    }
}
