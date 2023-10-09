use std::any::TypeId;

use boa_gc::{GcRef, GcRefCell, GcRefMut};
use boa_macros::{Finalize, Trace};
use rustc_hash::FxHashMap;

use crate::object::NativeObject;

/// This represents a `ECMASCript` specification \[`HostDefined`\] field.
///
/// This allows storing types which are mapped by their [`TypeId`].
#[derive(Default, Trace, Finalize)]
#[allow(missing_debug_implementations)]
pub struct HostDefined {
    env: FxHashMap<TypeId, GcRefCell<Box<dyn NativeObject>>>,
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn downcast_boxed_native_object_unchecked<T: NativeObject>(
    obj: Box<dyn NativeObject>,
) -> Box<T> {
    let raw: *mut dyn NativeObject = Box::into_raw(obj);
    Box::from_raw(raw as *mut T)
}

impl HostDefined {
    /// Insert a type into the [`HostDefined`].
    #[track_caller]
    pub fn insert_default<T: NativeObject + Default>(&mut self) -> Option<Box<T>> {
        self.env
            .insert(TypeId::of::<T>(), GcRefCell::new(Box::new(T::default())))
            .map(|obj| unsafe { downcast_boxed_native_object_unchecked(obj.into_inner()) })
    }

    /// Insert a type into the [`HostDefined`].
    #[track_caller]
    pub fn insert<T: NativeObject>(&mut self, value: T) -> Option<Box<T>> {
        self.env
            .insert(TypeId::of::<T>(), GcRefCell::new(Box::new(value)))
            .map(|obj| unsafe { downcast_boxed_native_object_unchecked(obj.into_inner()) })
    }

    /// Check if the [`HostDefined`] has type T.
    #[track_caller]
    pub fn has<T: NativeObject>(&self) -> bool {
        self.env.contains_key(&TypeId::of::<T>())
    }

    /// Remove type T from [`HostDefined`], if it exists.
    ///
    /// Returns [`Some`] with the object if it exits, [`None`] otherwise.
    #[track_caller]
    pub fn remove<T: NativeObject>(&mut self) -> Option<Box<T>> {
        self.env
            .remove(&TypeId::of::<T>())
            .map(|obj| unsafe { downcast_boxed_native_object_unchecked(obj.into_inner()) })
    }

    /// Get type T from [`HostDefined`], if it exists.
    ///
    /// # Panics
    ///
    /// Panics if T's entry in [`HostDefined`] is mutably borrowed.
    #[track_caller]
    pub fn get<T: NativeObject>(&self) -> Option<GcRef<'_, T>> {
        let entry = self.env.get(&TypeId::of::<T>())?;

        GcRef::try_map(entry.borrow(), |obj| obj.as_ref().downcast_ref::<T>())
    }

    /// Get type T from [`HostDefined`], if it exists.
    ///
    /// # Panics
    ///
    /// Panics if T's entry in [`HostDefined`] is borrowed.
    #[track_caller]
    pub fn get_mut<T: NativeObject>(&self) -> Option<GcRefMut<'_, Box<dyn NativeObject>, T>> {
        let entry = self.env.get(&TypeId::of::<T>())?;

        GcRefMut::try_map(entry.borrow_mut(), |obj| obj.as_mut().downcast_mut::<T>())
    }

    /// Clears all the objects.
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[track_caller]
    pub fn clear(&mut self) {
        self.env.clear();
    }
}
