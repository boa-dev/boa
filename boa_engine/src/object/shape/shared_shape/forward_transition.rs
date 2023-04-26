use boa_gc::{Finalize, Gc, GcRefCell, Trace, WeakGc};
use rustc_hash::FxHashMap;

use crate::object::JsPrototype;

use super::{Inner as SharedShapeInner, TransitionKey};

/// Maps transition key type to a [`SharedShapeInner`] transition.
type TransitionMap<T> = FxHashMap<T, WeakGc<SharedShapeInner>>;

/// The internal representation of [`ForwardTransition`].
#[derive(Default, Debug, Trace, Finalize)]
struct Inner {
    properties: Option<Box<TransitionMap<TransitionKey>>>,
    prototypes: Option<Box<TransitionMap<JsPrototype>>>,
}

/// Holds a forward reference to a previously created transition.
///
/// The reference is weak, therefore it can be garbage collected if it is not in use.
#[derive(Default, Debug, Trace, Finalize)]
pub(super) struct ForwardTransition {
    inner: GcRefCell<Inner>,
}

impl ForwardTransition {
    /// Insert a property transition.
    pub(super) fn insert_property(&self, key: TransitionKey, value: &Gc<SharedShapeInner>) {
        let mut this = self.inner.borrow_mut();
        let properties = this.properties.get_or_insert_with(Box::default);
        properties.insert(key, WeakGc::new(value));
    }

    /// Insert a prototype transition.
    pub(super) fn insert_prototype(&self, key: JsPrototype, value: &Gc<SharedShapeInner>) {
        let mut this = self.inner.borrow_mut();
        let prototypes = this.prototypes.get_or_insert_with(Box::default);
        prototypes.insert(key, WeakGc::new(value));
    }

    /// Get a property transition, return [`None`] otherwise.
    pub(super) fn get_property(&self, key: &TransitionKey) -> Option<WeakGc<SharedShapeInner>> {
        let this = self.inner.borrow();
        let Some(transitions) = this.properties.as_ref() else {
            return None;
        };
        transitions.get(key).cloned()
    }

    /// Get a prototype transition, return [`None`] otherwise.
    pub(super) fn get_prototype(&self, key: &JsPrototype) -> Option<WeakGc<SharedShapeInner>> {
        let this = self.inner.borrow();
        let Some(transitions) = this.prototypes.as_ref() else {
            return None;
        };
        transitions.get(key).cloned()
    }
}
