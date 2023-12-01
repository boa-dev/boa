use crate::{Ephemeron, Finalize, Gc, Trace};
use std::hash::{Hash, Hasher};

/// A weak reference to a [`Gc`].
///
/// This type allows keeping references to [`Gc`] managed values without keeping them alive for
/// garbage collections. However, this also means [`WeakGc::upgrade`] could return `None` at any moment.
#[derive(Debug, Trace, Finalize)]
#[repr(transparent)]
pub struct WeakGc<T: Trace + 'static> {
    inner: Ephemeron<T, ()>,
}

impl<T: Trace> WeakGc<T> {
    /// Creates a new weak pointer for a garbage collected value.
    #[inline]
    #[must_use]
    pub fn new(value: &Gc<T>) -> Self {
        Self {
            inner: Ephemeron::new(value, ()),
        }
    }

    /// Upgrade returns a `Gc` pointer for the internal value if the pointer is still live, or `None`
    /// if the value was already garbage collected.
    #[inline]
    #[must_use]
    pub fn upgrade(&self) -> Option<Gc<T>> {
        self.inner.key()
    }

    /// Check if the [`WeakGc`] can be upgraded.
    #[inline]
    #[must_use]
    pub fn is_upgradable(&self) -> bool {
        self.inner.has_value()
    }

    #[must_use]
    pub(crate) const fn inner(&self) -> &Ephemeron<T, ()> {
        &self.inner
    }
}

impl<T: Trace> Clone for WeakGc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Trace> From<Ephemeron<T, ()>> for WeakGc<T> {
    fn from(inner: Ephemeron<T, ()>) -> Self {
        Self { inner }
    }
}

impl<T: Trace> PartialEq for WeakGc<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.upgrade(), other.upgrade()) {
            (Some(a), Some(b)) => std::ptr::eq(a.as_ref(), b.as_ref()),
            _ => false,
        }
    }
}

impl<T: Trace> Eq for WeakGc<T> {}

impl<T: Trace> Hash for WeakGc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(obj) = self.upgrade() {
            std::ptr::hash(obj.as_ref(), state);
        } else {
            std::ptr::hash(self, state);
        }
    }
}
