//! This module implements the global `Weak*` objects.

//! Boa's implementation of JavaScript's `WeakRef` object.

mod weak_ref;

pub(crate) use weak_ref::WeakRef;
