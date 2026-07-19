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

#[cfg(test)]
mod tests {
    use super::{JsObject, WeakJsObject};
    use boa_gc::force_collect;

    #[test]
    fn upgrade_while_referent_is_live() {
        let object = JsObject::with_null_proto();
        let weak = WeakJsObject::new(&object);

        assert!(weak.is_upgradable());
        let upgraded = weak.upgrade().expect("referent is still alive");
        assert_eq!(upgraded, object);
    }

    #[test]
    fn upgrade_returns_none_after_referent_is_collected() {
        let object = JsObject::with_null_proto();
        let weak = WeakJsObject::new(&object);

        // While the strong reference is alive the weak one can be upgraded, even across a collection.
        force_collect();
        assert!(weak.is_upgradable());
        assert!(weak.upgrade().is_some());

        // Once the last strong reference is gone the referent can be collected.
        drop(object);
        force_collect();

        assert!(!weak.is_upgradable());
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn clone_points_to_the_same_referent() {
        let object = JsObject::with_null_proto();
        let weak = WeakJsObject::new(&object);
        let cloned = weak.clone();

        assert_eq!(weak, cloned);
        assert_eq!(
            weak.upgrade().expect("live"),
            cloned.upgrade().expect("live")
        );
    }

    #[test]
    fn equality_is_by_referent_identity() {
        let a = JsObject::with_null_proto();
        let b = JsObject::with_null_proto();

        let weak_a1 = WeakJsObject::new(&a);
        let weak_a2 = WeakJsObject::new(&a);
        let weak_b = WeakJsObject::new(&b);

        assert_eq!(weak_a1, weak_a2);
        assert_ne!(weak_a1, weak_b);
    }

    #[test]
    fn built_from_reference_via_conversion() {
        let object = JsObject::with_null_proto();
        let weak: WeakJsObject = (&object).into();

        assert_eq!(weak.upgrade().expect("live"), object);
    }
}
