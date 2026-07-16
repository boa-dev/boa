//! This module implements the [`WeakJsObject`] structure.
//!
//! A [`WeakJsObject`] is a weak reference to a [`JsObject`], allowing an embedder to hold a
//! reference to an object without keeping it alive across garbage collections.

use super::{ErasedObjectData, JsObject, NativeObject, jsobject::VTableObject};
use boa_gc::{Finalize, Trace, WeakGc};
use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
};

/// A weak reference to a [`JsObject`].
///
/// This is the object-level counterpart of [`boa_gc::WeakGc`]. It lets embedders keep a handle to a
/// [`JsObject`] without preventing it from being collected. Because the referenced object may be
/// collected at any point, [`WeakJsObject::upgrade`] returns an `Option<JsObject<T>>` that is `None`
/// once the object is gone.
///
/// # Examples
///
/// ```
/// # use boa_engine::object::{JsObject, WeakJsObject};
/// let object = JsObject::with_null_proto();
/// let weak = WeakJsObject::new(&object);
///
/// // While `object` is alive, the weak reference can be upgraded.
/// assert!(weak.upgrade().is_some());
/// ```
#[derive(Trace, Finalize)]
pub struct WeakJsObject<T: NativeObject = ErasedObjectData> {
    inner: WeakGc<VTableObject<T>>,
}

impl<T: NativeObject> WeakJsObject<T> {
    /// Creates a new weak reference to the given [`JsObject`].
    #[inline]
    #[must_use]
    pub fn new(object: &JsObject<T>) -> Self {
        Self {
            inner: WeakGc::new(object.inner()),
        }
    }

    /// Upgrades the weak reference to a strong [`JsObject`] if the referenced object is still live,
    /// or returns `None` if it was already garbage collected.
    #[inline]
    #[must_use]
    pub fn upgrade(&self) -> Option<JsObject<T>> {
        self.inner.upgrade().map(JsObject::from_inner)
    }

    /// Checks whether this weak reference can still be upgraded to a live [`JsObject`].
    #[inline]
    #[must_use]
    pub fn is_upgradable(&self) -> bool {
        self.inner.is_upgradable()
    }
}

impl<T: NativeObject> From<&JsObject<T>> for WeakJsObject<T> {
    fn from(object: &JsObject<T>) -> Self {
        Self::new(object)
    }
}

impl<T: NativeObject> Clone for WeakJsObject<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: NativeObject> PartialEq for WeakJsObject<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: NativeObject> Eq for WeakJsObject<T> {}

impl<T: NativeObject> Hash for WeakJsObject<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

// `VTableObject` deliberately does not implement `Debug` to avoid recursing into the object graph
// (which could overflow the stack), so we cannot derive `Debug` here. We provide a minimal,
// non-recursive implementation instead.
impl<T: NativeObject> Debug for WeakJsObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeakJsObject").finish_non_exhaustive()
    }
}
