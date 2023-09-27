use std::any::TypeId;

use boa_gc::{GcRef, GcRefCell, GcRefMut};
use boa_macros::{Finalize, Trace};
use rustc_hash::FxHashMap;

use crate::object::NativeObject;

/// Map used to store the host defined objects.
#[doc(hidden)]
type HostDefinedMap = FxHashMap<TypeId, Box<dyn NativeObject>>;

/// This represents a `ECMASCript` specification \[`HostDefined`\] field.
///
/// This allows storing types which are mapped by their [`TypeId`].
#[derive(Default, Trace, Finalize)]
#[allow(missing_debug_implementations)]
pub struct HostDefined {
    state: GcRefCell<HostDefinedMap>,
}

impl HostDefined {
    /// Insert a type into the [`HostDefined`].
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[track_caller]
    pub fn insert_default<T: NativeObject + Default>(&self) -> Option<Box<dyn NativeObject>> {
        self.state
            .borrow_mut()
            .insert(TypeId::of::<T>(), Box::<T>::default())
    }

    /// Insert a type into the [`HostDefined`].
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[track_caller]
    pub fn insert<T: NativeObject>(&self, value: T) -> Option<Box<dyn NativeObject>> {
        self.state
            .borrow_mut()
            .insert(TypeId::of::<T>(), Box::new(value))
    }

    /// Check if the [`HostDefined`] has type T.
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed mutably.
    #[track_caller]
    pub fn has<T: NativeObject>(&self) -> bool {
        self.state.borrow().contains_key(&TypeId::of::<T>())
    }

    /// Remove type T from [`HostDefined`], if it exists.
    ///
    /// Returns [`Some`] with the object if it exits, [`None`] otherwise.
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[track_caller]
    pub fn remove<T: NativeObject>(&self) -> Option<Box<dyn NativeObject>> {
        self.state.borrow_mut().remove(&TypeId::of::<T>())
    }

    /// Get type T from [`HostDefined`], if it exits.
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[track_caller]
    pub fn get<T: NativeObject>(&self) -> Option<GcRef<'_, T>> {
        let state = self.state.borrow();

        state
            .get(&TypeId::of::<T>())
            .map(Box::as_ref)
            .and_then(<dyn NativeObject>::downcast_ref::<T>)?;

        Some(GcRef::map(state, |state| {
            state
                .get(&TypeId::of::<T>())
                .map(Box::as_ref)
                .and_then(<dyn NativeObject>::downcast_ref::<T>)
                .expect("Should not fail")
        }))
    }

    /// Get type T from [`HostDefined`], if it exits.
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[track_caller]
    pub fn get_mut<T: NativeObject>(&self) -> Option<GcRefMut<'_, HostDefinedMap, T>> {
        let mut state = self.state.borrow_mut();

        state
            .get_mut(&TypeId::of::<T>())
            .map(Box::as_mut)
            .and_then(<dyn NativeObject>::downcast_mut::<T>)?;

        Some(GcRefMut::map(
            state,
            |state: &mut FxHashMap<TypeId, Box<dyn NativeObject>>| {
                state
                    .get_mut(&TypeId::of::<T>())
                    .map(Box::as_mut)
                    .and_then(<dyn NativeObject>::downcast_mut::<T>)
                    .expect("Should not fail")
            },
        ))
    }

    /// Clears all the objects.
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[track_caller]
    pub fn clear(&self) {
        self.state.borrow_mut().clear();
    }
}
