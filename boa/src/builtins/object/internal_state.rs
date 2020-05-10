//! Implementations for storing normal rust structs inside any object as internal state.

use gc::{unsafe_empty_trace, Finalize, Trace};
use std::{
    any::Any,
    fmt::{self, Debug},
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// Wrapper around `Rc` to implement `Trace` and `Finalize`.
#[derive(Clone)]
pub struct InternalStateCell {
    /// The internal state.
    state: Rc<dyn InternalState + 'static>,
}

impl Finalize for InternalStateCell {}

unsafe impl Trace for InternalStateCell {
    unsafe_empty_trace!();
}

impl Deref for InternalStateCell {
    type Target = dyn InternalState;
    fn deref(&self) -> &Self::Target {
        Deref::deref(&self.state)
    }
}

impl DerefMut for InternalStateCell {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Rc::get_mut(&mut self.state).expect("failed to get mutable")
    }
}

/// The derived version would print 'InternalStateCell { state: ... }', this custom implementation
/// only prints the actual internal state.
impl Debug for InternalStateCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.state, f)
    }
}

impl InternalStateCell {
    /// Create new `InternalStateCell` from a value.
    pub fn new<T: 'static + InternalState>(value: T) -> Self {
        Self {
            state: Rc::new(value),
        }
    }

    /// Get a reference to the stored value and cast it to `T`.
    pub fn downcast_ref<T: Any + InternalState>(&self) -> Option<&T> {
        self.deref().downcast_ref::<T>()
    }

    /// Get a mutable reference to the stored value and cast it to `T`.
    pub fn downcast_mut<T: InternalState>(&mut self) -> Option<&mut T> {
        self.state.downcast_mut::<T>()
    }
}

/// This trait must be implemented by all structs used for internal state.
pub trait InternalState: Debug + Trace + Any {}
