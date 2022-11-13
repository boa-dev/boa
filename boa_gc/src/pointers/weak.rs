use crate::{Ephemeron, Finalize, Gc, Trace};

/// A weak reference to a [`Gc`].
///
/// This type allows keeping references to [`Gc`] managed values without keeping them alive for
/// garbage collections. However, this also means [`WeakGc::value`] can return `None` at any moment.
#[derive(Debug, Trace, Finalize)]
#[repr(transparent)]
pub struct WeakGc<T: Trace + ?Sized + 'static> {
    inner: Ephemeron<T, ()>,
}

impl<T: Trace + ?Sized> WeakGc<T> {
    /// Creates a new weak pointer for a garbage collected value.
    pub fn new(value: &Gc<T>) -> Self {
        Self {
            inner: Ephemeron::new(value, ()),
        }
    }
}

impl<T: Trace + ?Sized> WeakGc<T> {
    #[inline]
    /// Gets the value of this weak pointer, or `None` if the value was already garbage collected.
    pub fn value(&self) -> Option<&T> {
        self.inner.key()
    }

    #[inline]
    /// Upgrade returns a `Gc` pointer for the internal value if valid, or None if the value was already garbage collected.
    pub fn upgrade(&self) -> Option<Gc<T>> {
        self.inner.upgrade_key()
    }
}

impl<T: Trace + ?Sized> Clone for WeakGc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Trace + ?Sized> From<Ephemeron<T, ()>> for WeakGc<T> {
    fn from(inner: Ephemeron<T, ()>) -> Self {
        Self { inner }
    }
}
