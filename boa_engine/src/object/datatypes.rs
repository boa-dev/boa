use std::{
    any::TypeId,
    borrow::Cow,
    cell::Cell,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic,
};

use boa_gc::{Ephemeron, Gc, GcRefCell, Trace, WeakGc, WeakMap};
pub use boa_macros::JsData;

use super::internal_methods::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS};

pub trait JsData {
    #[doc(hidden)]
    fn internal_methods(&self) -> &'static InternalObjectMethods
    where
        Self: Sized,
    {
        &ORDINARY_INTERNAL_METHODS
    }
}

macro_rules! default_impls {
    ($($T:ty),*$(,)?) => {
        $(
            impl JsData for $T {}
        )*
    }
}

default_impls![
    (),
    bool,
    isize,
    usize,
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    i128,
    u128,
    f32,
    f64,
    char,
    TypeId,
    String,
    Path,
    PathBuf,
    NonZeroIsize,
    NonZeroUsize,
    NonZeroI8,
    NonZeroU8,
    NonZeroI16,
    NonZeroU16,
    NonZeroI32,
    NonZeroU32,
    NonZeroI64,
    NonZeroU64,
    NonZeroI128,
    NonZeroU128,
];

#[cfg(target_has_atomic = "8")]
default_impls![atomic::AtomicBool, atomic::AtomicI8, atomic::AtomicU8];

#[cfg(target_has_atomic = "16")]
default_impls![atomic::AtomicI16, atomic::AtomicU16];

#[cfg(target_has_atomic = "32")]
default_impls![atomic::AtomicI32, atomic::AtomicU32];

#[cfg(target_has_atomic = "64")]
default_impls![atomic::AtomicI64, atomic::AtomicU64];

#[cfg(target_has_atomic = "ptr")]
default_impls![atomic::AtomicIsize, atomic::AtomicUsize];

impl<T, const N: usize> JsData for [T; N] {}

macro_rules! fn_one {
    ($ty:ty $(,$args:ident)*) => {
        impl<Ret $(,$args)*> JsData for $ty {}
    }
}

macro_rules! fn_impls {
    () => {
        fn_one!(extern "Rust" fn () -> Ret);
        fn_one!(extern "C" fn () -> Ret);
        fn_one!(unsafe extern "Rust" fn () -> Ret);
        fn_one!(unsafe extern "C" fn () -> Ret);
    };
    ($($args:ident),*) => {
        fn_one!(extern "Rust" fn ($($args),*) -> Ret, $($args),*);
        fn_one!(extern "C" fn ($($args),*) -> Ret, $($args),*);
        fn_one!(extern "C" fn ($($args),*, ...) -> Ret, $($args),*);
        fn_one!(unsafe extern "Rust" fn ($($args),*) -> Ret, $($args),*);
        fn_one!(unsafe extern "C" fn ($($args),*) -> Ret, $($args),*);
        fn_one!(unsafe extern "C" fn ($($args),*, ...) -> Ret, $($args),*);
    }
}

macro_rules! tuple_impls {
    () => {}; // This case is handled above, by default_impls!().
    ($($args:ident),*) => {
        impl<$($args),*> JsData for ($($args,)*) {}
    }
}

macro_rules! type_arg_tuple_based_impls {
    ($(($($args:ident),*);)*) => {
        $(
            fn_impls!($($args),*);
            tuple_impls!($($args),*);
        )*
    }
}

type_arg_tuple_based_impls![
    ();
    (A);
    (A, B);
    (A, B, C);
    (A, B, C, D);
    (A, B, C, D, E);
    (A, B, C, D, E, F);
    (A, B, C, D, E, F, G);
    (A, B, C, D, E, F, G, H);
    (A, B, C, D, E, F, G, H, I);
    (A, B, C, D, E, F, G, H, I, J);
    (A, B, C, D, E, F, G, H, I, J, K);
    (A, B, C, D, E, F, G, H, I, J, K, L);
];

impl<T: ?Sized> JsData for Box<T> {}

impl<T: ?Sized> JsData for Rc<T> {}

impl<T> JsData for Vec<T> {}

impl<T> JsData for thin_vec::ThinVec<T> {}

impl<T> JsData for Option<T> {}

impl<T, E> JsData for Result<T, E> {}

impl<T: Ord> JsData for BinaryHeap<T> {}

impl<K, V> JsData for BTreeMap<K, V> {}

impl<T> JsData for BTreeSet<T> {}

impl<K: Eq + Hash, V, S: BuildHasher> JsData for hashbrown::hash_map::HashMap<K, V, S> {}

impl<K: Eq + Hash, V, S: BuildHasher> JsData for HashMap<K, V, S> {}

impl<T: Eq + Hash, S: BuildHasher> JsData for HashSet<T, S> {}

impl<T: Eq + Hash> JsData for LinkedList<T> {}

impl<T> JsData for PhantomData<T> {}

impl<T> JsData for VecDeque<T> {}

impl<T: ToOwned + ?Sized> JsData for Cow<'static, T> {}

impl<T> JsData for Cell<Option<T>> {}

#[cfg(feature = "intl")]
default_impls!(icu_locid::Locale);

impl<T: Trace + ?Sized> JsData for Gc<T> {}

impl<T: Trace + ?Sized> JsData for WeakGc<T> {}

impl<T: Trace + ?Sized, V: Trace> JsData for Ephemeron<T, V> {}

impl<T: Trace + ?Sized> JsData for GcRefCell<T> {}

impl<K: Trace + ?Sized, V: Trace> JsData for WeakMap<K, V> {}
