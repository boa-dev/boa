use std::{
    any::TypeId,
    borrow::{Cow, ToOwned},
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

use crate::GcErasedPointer;

/// A queue used to trace [`crate::Gc<T>`] non-recursively.
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct Tracer {
    queue: VecDeque<GcErasedPointer>,
}

impl Tracer {
    pub(crate) fn new() -> Self {
        Self {
            queue: VecDeque::default(),
        }
    }

    pub(crate) fn enqueue(&mut self, node: GcErasedPointer) {
        self.queue.push_back(node);
    }

    pub(crate) fn next(&mut self) -> Option<GcErasedPointer> {
        self.queue.pop_front()
    }

    pub(crate) fn is_empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

/// Substitute for the [`Drop`] trait for garbage collected types.
pub trait Finalize {
    /// Cleanup logic for a type.
    fn finalize(&self) {}
}

/// The Trace trait, which needs to be implemented on garbage-collected objects.
///
/// # Safety
///
/// - An incorrect implementation of the trait can result in heap overflows, data corruption,
/// use-after-free, or Undefined Behaviour in general.
///
/// - Calling any of the functions marked as `unsafe` outside of the context of the garbage collector
/// can result in Undefined Behaviour.
pub unsafe trait Trace: Finalize {
    /// Marks all contained `Gc`s.
    ///
    /// # Safety
    ///
    /// See [`Trace`].
    unsafe fn trace(&self, tracer: &mut Tracer);

    /// Trace handles located in GC heap, and mark them as non root.
    ///
    /// # Safety
    ///
    /// See [`Trace`].
    unsafe fn trace_non_roots(&self);

    /// Runs [`Finalize::finalize`] on this object and all
    /// contained subobjects.
    fn run_finalizer(&self);
}

/// Utility macro to define an empty implementation of [`Trace`].
///
/// Use this for marking types as not containing any `Trace` types.
#[macro_export]
macro_rules! empty_trace {
    () => {
        #[inline]
        unsafe fn trace(&self, _tracer: &mut $crate::Tracer) {}
        #[inline]
        unsafe fn trace_non_roots(&self) {}
        #[inline]
        fn run_finalizer(&self) {
            $crate::Finalize::finalize(self)
        }
    };
}

/// Utility macro to manually implement [`Trace`] on a type.
///
/// You define a `this` parameter name and pass in a body, which should call `mark` on every
/// traceable element inside the body. The mark implementation will automatically delegate to the
/// correct method on the argument.
///
/// # Safety
///
/// Misusing the `mark` function may result in Undefined Behaviour.
#[macro_export]
macro_rules! custom_trace {
    ($this:ident, $marker:ident, $body:expr) => {
        #[inline]
        unsafe fn trace(&self, tracer: &mut $crate::Tracer) {
            let mut $marker = |it: &dyn $crate::Trace| {
                // SAFETY: The implementor must ensure that `trace` is correctly implemented.
                unsafe {
                    $crate::Trace::trace(it, tracer);
                }
            };
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn trace_non_roots(&self) {
            fn $marker<T: $crate::Trace + ?Sized>(it: &T) {
                // SAFETY: The implementor must ensure that `trace` is correctly implemented.
                unsafe {
                    $crate::Trace::trace_non_roots(it);
                }
            }
            let $this = self;
            $body
        }
        #[inline]
        fn run_finalizer(&self) {
            fn $marker<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::run_finalizer(it);
            }
            $crate::Finalize::finalize(self);
            let $this = self;
            $body
        }
    };
}

impl<T: ?Sized> Finalize for &'static T {}
// SAFETY: 'static references don't need to be traced, since they live indefinitely.
unsafe impl<T: ?Sized> Trace for &'static T {
    empty_trace!();
}

macro_rules! simple_empty_finalize_trace {
    ($($T:ty),*) => {
        $(
            impl Finalize for $T {}

            // SAFETY:
            // Primitive types and string types don't have inner nodes that need to be marked.
            unsafe impl Trace for $T { empty_trace!(); }
        )*
    }
}

simple_empty_finalize_trace![
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
    Box<str>,
    Rc<str>,
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
    NonZeroU128
];

#[cfg(target_has_atomic = "8")]
simple_empty_finalize_trace![atomic::AtomicBool, atomic::AtomicI8, atomic::AtomicU8];

#[cfg(target_has_atomic = "16")]
simple_empty_finalize_trace![atomic::AtomicI16, atomic::AtomicU16];

#[cfg(target_has_atomic = "32")]
simple_empty_finalize_trace![atomic::AtomicI32, atomic::AtomicU32];

#[cfg(target_has_atomic = "64")]
simple_empty_finalize_trace![atomic::AtomicI64, atomic::AtomicU64];

#[cfg(target_has_atomic = "ptr")]
simple_empty_finalize_trace![atomic::AtomicIsize, atomic::AtomicUsize];

impl<T: Trace, const N: usize> Finalize for [T; N] {}
// SAFETY:
// All elements inside the array are correctly marked.
unsafe impl<T: Trace, const N: usize> Trace for [T; N] {
    custom_trace!(this, mark, {
        for v in this {
            mark(v);
        }
    });
}

macro_rules! fn_finalize_trace_one {
    ($ty:ty $(,$args:ident)*) => {
        impl<Ret $(,$args)*> Finalize for $ty {}
        // SAFETY:
        // Function pointers don't have inner nodes that need to be marked.
        unsafe impl<Ret $(,$args)*> Trace for $ty { empty_trace!(); }
    }
}
macro_rules! fn_finalize_trace_group {
    () => {
        fn_finalize_trace_one!(extern "Rust" fn () -> Ret);
        fn_finalize_trace_one!(extern "C" fn () -> Ret);
        fn_finalize_trace_one!(unsafe extern "Rust" fn () -> Ret);
        fn_finalize_trace_one!(unsafe extern "C" fn () -> Ret);
    };
    ($($args:ident),*) => {
        fn_finalize_trace_one!(extern "Rust" fn ($($args),*) -> Ret, $($args),*);
        fn_finalize_trace_one!(extern "C" fn ($($args),*) -> Ret, $($args),*);
        fn_finalize_trace_one!(extern "C" fn ($($args),*, ...) -> Ret, $($args),*);
        fn_finalize_trace_one!(unsafe extern "Rust" fn ($($args),*) -> Ret, $($args),*);
        fn_finalize_trace_one!(unsafe extern "C" fn ($($args),*) -> Ret, $($args),*);
        fn_finalize_trace_one!(unsafe extern "C" fn ($($args),*, ...) -> Ret, $($args),*);
    }
}

macro_rules! tuple_finalize_trace {
    () => {}; // This case is handled above, by simple_finalize_empty_trace!().
    ($($args:ident),*) => {
        impl<$($args),*> Finalize for ($($args,)*) {}
        // SAFETY:
        // All elements inside the tuple are correctly marked.
        unsafe impl<$($args: $crate::Trace),*> Trace for ($($args,)*) {
            custom_trace!(this, mark, {
                #[allow(non_snake_case, unused_unsafe, unused_mut)]
                let mut avoid_lints = |&($(ref $args,)*): &($($args,)*)| {
                    // SAFETY: The implementor must ensure a correct implementation.
                    unsafe { $(mark($args);)* }
                };
                avoid_lints(this)
            });
        }
    }
}

macro_rules! type_arg_tuple_based_finalize_trace_impls {
    ($(($($args:ident),*);)*) => {
        $(
            fn_finalize_trace_group!($($args),*);
            tuple_finalize_trace!($($args),*);
        )*
    }
}

type_arg_tuple_based_finalize_trace_impls![
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

impl<T: Trace + ?Sized> Finalize for Box<T> {}
// SAFETY: The inner value of the `Box` is correctly marked.
unsafe impl<T: Trace + ?Sized> Trace for Box<T> {
    #[inline]
    unsafe fn trace(&self, tracer: &mut Tracer) {
        // SAFETY: The implementor must ensure that `trace` is correctly implemented.
        unsafe {
            Trace::trace(&**self, tracer);
        }
    }
    #[inline]
    unsafe fn trace_non_roots(&self) {
        // SAFETY: The implementor must ensure that `trace_non_roots` is correctly implemented.
        unsafe {
            Trace::trace_non_roots(&**self);
        }
    }
    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self);
        Trace::run_finalizer(&**self);
    }
}

impl<T: Trace> Finalize for Box<[T]> {}
// SAFETY: All the inner elements of the `Box` array are correctly marked.
unsafe impl<T: Trace> Trace for Box<[T]> {
    custom_trace!(this, mark, {
        for e in &**this {
            mark(e);
        }
    });
}

impl<T: Trace> Finalize for Vec<T> {}
// SAFETY: All the inner elements of the `Vec` are correctly marked.
unsafe impl<T: Trace> Trace for Vec<T> {
    custom_trace!(this, mark, {
        for e in this {
            mark(e);
        }
    });
}

#[cfg(feature = "thin-vec")]
impl<T: Trace> Finalize for thin_vec::ThinVec<T> {}

#[cfg(feature = "thin-vec")]
// SAFETY: All the inner elements of the `Vec` are correctly marked.
unsafe impl<T: Trace> Trace for thin_vec::ThinVec<T> {
    custom_trace!(this, mark, {
        for e in this {
            mark(e);
        }
    });
}

impl<T: Trace> Finalize for Option<T> {}
// SAFETY: The inner value of the `Option` is correctly marked.
unsafe impl<T: Trace> Trace for Option<T> {
    custom_trace!(this, mark, {
        if let Some(ref v) = *this {
            mark(v);
        }
    });
}

impl<T: Trace, E: Trace> Finalize for Result<T, E> {}
// SAFETY: Both inner values of the `Result` are correctly marked.
unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    custom_trace!(this, mark, {
        match *this {
            Ok(ref v) => mark(v),
            Err(ref v) => mark(v),
        }
    });
}

impl<T: Ord + Trace> Finalize for BinaryHeap<T> {}
// SAFETY: All the elements of the `BinaryHeap` are correctly marked.
unsafe impl<T: Ord + Trace> Trace for BinaryHeap<T> {
    custom_trace!(this, mark, {
        for v in this {
            mark(v);
        }
    });
}

impl<K: Trace, V: Trace> Finalize for BTreeMap<K, V> {}
// SAFETY: All the elements of the `BTreeMap` are correctly marked.
unsafe impl<K: Trace, V: Trace> Trace for BTreeMap<K, V> {
    custom_trace!(this, mark, {
        for (k, v) in this {
            mark(k);
            mark(v);
        }
    });
}

impl<T: Trace> Finalize for BTreeSet<T> {}
// SAFETY: All the elements of the `BTreeSet` are correctly marked.
unsafe impl<T: Trace> Trace for BTreeSet<T> {
    custom_trace!(this, mark, {
        for v in this {
            mark(v);
        }
    });
}

impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Finalize
    for hashbrown::hash_map::HashMap<K, V, S>
{
}
// SAFETY: All the elements of the `HashMap` are correctly marked.
unsafe impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Trace
    for hashbrown::hash_map::HashMap<K, V, S>
{
    custom_trace!(this, mark, {
        for (k, v) in this {
            mark(k);
            mark(v);
        }
    });
}

impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Finalize for HashMap<K, V, S> {}
// SAFETY: All the elements of the `HashMap` are correctly marked.
unsafe impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Trace for HashMap<K, V, S> {
    custom_trace!(this, mark, {
        for (k, v) in this {
            mark(k);
            mark(v);
        }
    });
}

impl<T: Eq + Hash + Trace, S: BuildHasher> Finalize for HashSet<T, S> {}
// SAFETY: All the elements of the `HashSet` are correctly marked.
unsafe impl<T: Eq + Hash + Trace, S: BuildHasher> Trace for HashSet<T, S> {
    custom_trace!(this, mark, {
        for v in this {
            mark(v);
        }
    });
}

impl<T: Eq + Hash + Trace> Finalize for LinkedList<T> {}
// SAFETY: All the elements of the `LinkedList` are correctly marked.
unsafe impl<T: Eq + Hash + Trace> Trace for LinkedList<T> {
    custom_trace!(this, mark, {
        #[allow(clippy::explicit_iter_loop)]
        for v in this.iter() {
            mark(v);
        }
    });
}

impl<T> Finalize for PhantomData<T> {}
// SAFETY: A `PhantomData` doesn't have inner data that needs to be marked.
unsafe impl<T> Trace for PhantomData<T> {
    empty_trace!();
}

impl<T: Trace> Finalize for VecDeque<T> {}
// SAFETY: All the elements of the `VecDeque` are correctly marked.
unsafe impl<T: Trace> Trace for VecDeque<T> {
    custom_trace!(this, mark, {
        for v in this {
            mark(v);
        }
    });
}

impl<T: ToOwned + Trace + ?Sized> Finalize for Cow<'static, T> {}
// SAFETY: 'static references don't need to be traced, since they live indefinitely, and the owned
// variant is correctly marked.
unsafe impl<T: ToOwned + Trace + ?Sized> Trace for Cow<'static, T>
where
    T::Owned: Trace,
{
    custom_trace!(this, mark, {
        if let Cow::Owned(ref v) = this {
            mark(v);
        }
    });
}

impl<T: Trace> Finalize for Cell<Option<T>> {}
// SAFETY: Taking and setting is done in a single action, and recursive traces should find a `None`
// value instead of the original `T`, making this safe.
unsafe impl<T: Trace> Trace for Cell<Option<T>> {
    custom_trace!(this, mark, {
        if let Some(v) = this.take() {
            mark(&v);
            this.set(Some(v));
        }
    });
}

#[cfg(feature = "icu")]
mod icu {
    use icu_locid::{LanguageIdentifier, Locale};

    use crate::{Finalize, Trace};

    impl Finalize for LanguageIdentifier {}

    // SAFETY: `LanguageIdentifier` doesn't have any traceable data.
    unsafe impl Trace for LanguageIdentifier {
        empty_trace!();
    }

    impl Finalize for Locale {}

    // SAFETY: `LanguageIdentifier` doesn't have any traceable data.
    unsafe impl Trace for Locale {
        empty_trace!();
    }
}
