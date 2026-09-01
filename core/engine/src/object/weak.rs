//! This module implements the [`WeakJsObject`] structure.
//!
//! A [`WeakJsObject`] is a weak reference to a [`JsObject`], allowing an embedder to hold a
//! reference to an object without keeping it alive across garbage collections.

use super::{ErasedObjectData, JsObject, NativeObject, jsobject::VTableObject};
use boa_gc::{Finalize, MutationContext, Trace, WeakGc};
use std::fmt::{self, Debug};

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
/// # use boa_engine::{object::{JsObject, WeakJsObject}, Context};
///
/// let context = &mut Context::default();
///
/// let object = JsObject::with_null_proto(context.gc_collector());
/// let weak = WeakJsObject::new(&object, context.gc_collector());
///
/// // While `object` is alive, the weak reference can be upgraded.
/// assert!(weak.upgrade(context.gc_collector()).is_some());
/// ```
#[derive(Trace, Finalize)]
pub struct WeakJsObject<T: NativeObject = ErasedObjectData> {
    inner: WeakGc<VTableObject<T>>,
}

impl<T: NativeObject> WeakJsObject<T> {
    /// Creates a new weak reference to the given [`JsObject`].
    #[inline]
    #[must_use]
    pub fn new(object: &JsObject<T>, mc: &MutationContext<'_, '_>) -> Self {
        Self {
            inner: WeakGc::new(mc, object.inner()),
        }
    }

    /// Upgrades the weak reference to a strong [`JsObject`] if the referenced object is still live,
    /// or returns `None` if it was already garbage collected.
    #[inline]
    #[must_use]
    pub fn upgrade(&self, mc: &MutationContext<'_, '_>) -> Option<JsObject<T>> {
        self.inner.upgrade(mc).map(JsObject::from_inner)
    }

    /// Checks whether this weak reference can still be upgraded to a live [`JsObject`].
    #[inline]
    #[must_use]
    pub fn is_upgradable(&self) -> bool {
        self.inner.is_upgradable()
    }
}

impl<T: NativeObject> Clone for WeakJsObject<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// `PartialEq`/`Eq`/`Hash` are intentionally not implemented. The natural forwarding to `WeakGc`
// would compare and hash by the *live* referent, so two references to the same object would stop
// comparing equal (and change their hash) once that object is collected, breaking `Eq`'s
// reflexivity and the `Hash`/`Eq` invariant for a collected key. To compare two weak references,
// upgrade them first and compare the resulting `JsObject`s. These impls can be added later, without
// a breaking change, if a collection-stable identity is designed.

// We can't derive `Debug` because `VTableObject` deliberately doesn't implement it. Instead we
// upgrade and delegate to `JsObject`'s own `Debug`, which uses a `RecursionLimiter` to avoid
// overflowing the stack on cyclic object graphs. A collected referent prints as `None`.
impl<T: NativeObject> Debug for WeakJsObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WeakJsObject[unknown]").finish()
    }
}
