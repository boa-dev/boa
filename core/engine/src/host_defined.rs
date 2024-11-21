use std::any::TypeId;

use boa_macros::{Finalize, Trace};
use hashbrown::hash_map::HashMap;

use crate::object::NativeObject;

/// This represents a `ECMASCript` specification \[`HostDefined`\] field.
///
/// This allows storing types which are mapped by their [`TypeId`].
#[derive(Default, Trace, Finalize)]
#[allow(missing_debug_implementations)]
pub struct HostDefined {
    // INVARIANT: All key-value pairs `(id, obj)` satisfy:
    //  `id == TypeId::of::<T>() && obj.is::<T>()`
    // for some type `T : NativeObject`.
    types: HashMap<TypeId, Box<dyn NativeObject>>,
}

// TODO: Track https://github.com/rust-lang/rust/issues/65991 and
// https://github.com/rust-lang/rust/issues/90850 to remove this
// when those are stabilized.
fn downcast_boxed_native_object_unchecked<T: NativeObject>(obj: Box<dyn NativeObject>) -> Box<T> {
    let raw: *mut dyn NativeObject = Box::into_raw(obj);

    // SAFETY: We know that `obj` is of type `T` (due to the INVARIANT of `HostDefined`).
    // See `HostDefined::insert`, `HostDefined::insert_default` and `HostDefined::remove`.
    unsafe { Box::from_raw(raw.cast::<T>()) }
}

impl HostDefined {
    /// Insert a type into the [`HostDefined`].
    #[track_caller]
    pub fn insert_default<T: NativeObject + Default>(&mut self) -> Option<Box<T>> {
        self.types
            .insert(TypeId::of::<T>(), Box::<T>::default())
            .map(downcast_boxed_native_object_unchecked)
    }

    /// Insert a type into the [`HostDefined`].
    #[track_caller]
    pub fn insert<T: NativeObject>(&mut self, value: T) -> Option<Box<T>> {
        self.types
            .insert(TypeId::of::<T>(), Box::new(value))
            .map(downcast_boxed_native_object_unchecked)
    }

    /// Check if the [`HostDefined`] has type T.
    #[must_use]
    #[track_caller]
    pub fn has<T: NativeObject>(&self) -> bool {
        self.types.contains_key(&TypeId::of::<T>())
    }

    /// Remove type T from [`HostDefined`], if it exists.
    ///
    /// Returns [`Some`] with the object if it exits, [`None`] otherwise.
    #[track_caller]
    pub fn remove<T: NativeObject>(&mut self) -> Option<Box<T>> {
        self.types
            .remove(&TypeId::of::<T>())
            .map(downcast_boxed_native_object_unchecked)
    }

    /// Get type T from [`HostDefined`], if it exists.
    #[track_caller]
    pub fn get<T: NativeObject>(&self) -> Option<&T> {
        self.types
            .get(&TypeId::of::<T>())
            .map(Box::as_ref)
            .and_then(<dyn NativeObject>::downcast_ref::<T>)
    }

    /// Get type T from [`HostDefined`], if it exists.
    #[track_caller]
    pub fn get_mut<T: NativeObject>(&mut self) -> Option<&mut T> {
        self.types
            .get_mut(&TypeId::of::<T>())
            .map(Box::as_mut)
            .and_then(<dyn NativeObject>::downcast_mut::<T>)
    }

    /// Get a tuple of types from [`HostDefined`], returning `None` for the types that are not on the map.
    #[track_caller]
    pub fn get_many_mut<T, const SIZE: usize>(&mut self) -> T::NativeTupleMutRef<'_>
    where
        T: NativeTuple<SIZE>,
    {
        let ids = T::as_type_ids();
        let refs: [&TypeId; SIZE] = std::array::from_fn(|i| &ids[i]);

        T::mut_ref_from_anys(self.types.get_many_mut(refs))
    }

    /// Clears all the objects.
    #[track_caller]
    pub fn clear(&mut self) {
        self.types.clear();
    }
}

/// This trait represents a tuple of [`NativeObject`]s capable of being
/// used in [`HostDefined`].
///
/// This allows accessing multiple types from [`HostDefined`] at once.
pub trait NativeTuple<const SIZE: usize> {
    type NativeTupleMutRef<'a>;

    fn as_type_ids() -> [TypeId; SIZE];

    fn mut_ref_from_anys(
        anys: [Option<&'_ mut Box<dyn NativeObject>>; SIZE],
    ) -> Self::NativeTupleMutRef<'_>;
}

macro_rules! impl_native_tuple {
    ($size:literal $(,$name:ident)* ) => {
        impl<$($name: NativeObject,)*> NativeTuple<$size> for ($($name,)*) {
            type NativeTupleMutRef<'a> = ($(Option<&'a mut $name>,)*);

            fn as_type_ids() -> [TypeId; $size] {
                [$(TypeId::of::<$name>(),)*]
            }

            #[allow(unused_variables, unused_mut, clippy::unused_unit)]
            fn mut_ref_from_anys(
                anys: [Option<&'_ mut Box<dyn NativeObject>>; $size],
            ) -> Self::NativeTupleMutRef<'_> {
                let mut anys = anys.into_iter();
                ($(
                    anys.next().flatten().and_then(|v| v.downcast_mut::<$name>()),
                )*)
            }
        }
    }
}

impl_native_tuple!(0);
impl_native_tuple!(1, A);
impl_native_tuple!(2, A, B);
impl_native_tuple!(3, A, B, C);
impl_native_tuple!(4, A, B, C, D);
impl_native_tuple!(5, A, B, C, D, E);
impl_native_tuple!(6, A, B, C, D, E, F);
impl_native_tuple!(7, A, B, C, D, E, F, G);
impl_native_tuple!(8, A, B, C, D, E, F, G, H);
impl_native_tuple!(9, A, B, C, D, E, F, G, H, I);
impl_native_tuple!(10, A, B, C, D, E, F, G, H, I, J);
impl_native_tuple!(11, A, B, C, D, E, F, G, H, I, J, K);
impl_native_tuple!(12, A, B, C, D, E, F, G, H, I, J, K, L);
