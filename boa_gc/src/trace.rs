use std::{
    borrow::{Cow, ToOwned},
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
        AtomicU64, AtomicU8, AtomicUsize,
    },
};

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
    unsafe fn trace(&self);

    /// Marks all contained weak references of a `Gc`.
    ///
    /// # Safety
    ///
    /// See [`Trace`].
    unsafe fn weak_trace(&self);

    /// Increments the root-count of all contained `Gc`s.
    ///
    /// # Safety
    ///
    /// See [`Trace`].
    unsafe fn root(&self);

    /// Decrements the root-count of all contained `Gc`s.
    ///
    /// # Safety
    ///
    /// See [`Trace`].
    unsafe fn unroot(&self);

    /// Checks if an ephemeron's key is marked.
    #[doc(hidden)]
    fn is_marked_ephemeron(&self) -> bool {
        false
    }

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
        unsafe fn trace(&self) {}
        #[inline]
        unsafe fn weak_trace(&self) {}
        #[inline]
        unsafe fn root(&self) {}
        #[inline]
        unsafe fn unroot(&self) {}
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
    ($this:ident, $body:expr) => {
        #[inline]
        unsafe fn trace(&self) {
            #[inline]
            fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                // SAFETY: The implementor must ensure that `trace` is correctly implemented.
                unsafe {
                    $crate::Trace::trace(it);
                }
            }
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn weak_trace(&self) {
            #[inline]
            fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                // SAFETY: The implementor must ensure that `weak_trace` is correctly implemented.
                unsafe {
                    $crate::Trace::weak_trace(it);
                }
            }
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn root(&self) {
            #[inline]
            fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                // SAFETY: The implementor must ensure that `root` is correctly implemented.
                unsafe {
                    $crate::Trace::root(it);
                }
            }
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn unroot(&self) {
            #[inline]
            fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                // SAFETY: The implementor must ensure that `unroot` is correctly implemented.
                unsafe {
                    $crate::Trace::unroot(it);
                }
            }
            let $this = self;
            $body
        }
        #[inline]
        fn run_finalizer(&self) {
            #[inline]
            fn mark<T: $crate::Trace + ?Sized>(it: &T) {
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
    NonZeroU128,
    AtomicBool,
    AtomicIsize,
    AtomicUsize,
    AtomicI8,
    AtomicU8,
    AtomicI16,
    AtomicU16,
    AtomicI32,
    AtomicU32,
    AtomicI64,
    AtomicU64
];

impl<T: Trace, const N: usize> Finalize for [T; N] {}
// SAFETY:
// All elements inside the array are correctly marked.
unsafe impl<T: Trace, const N: usize> Trace for [T; N] {
    custom_trace!(this, {
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
            custom_trace!(this, {
                #[allow(non_snake_case, unused_unsafe)]
                fn avoid_lints<$($args: $crate::Trace),*>(&($(ref $args,)*): &($($args,)*)) {
                    // SAFETY: The implementor must ensure a correct implementation.
                    unsafe { $(mark($args);)* }
                }
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
    custom_trace!(this, {
        mark(&**this);
    });
}

impl<T: Trace> Finalize for Box<[T]> {}
// SAFETY: All the inner elements of the `Box` array are correctly marked.
unsafe impl<T: Trace> Trace for Box<[T]> {
    custom_trace!(this, {
        for e in this.iter() {
            mark(e);
        }
    });
}

impl<T: Trace> Finalize for Vec<T> {}
// SAFETY: All the inner elements of the `Vec` are correctly marked.
unsafe impl<T: Trace> Trace for Vec<T> {
    custom_trace!(this, {
        for e in this {
            mark(e);
        }
    });
}

impl<T: Trace> Finalize for Option<T> {}
// SAFETY: The inner value of the `Option` is correctly marked.
unsafe impl<T: Trace> Trace for Option<T> {
    custom_trace!(this, {
        if let Some(ref v) = *this {
            mark(v);
        }
    });
}

impl<T: Trace, E: Trace> Finalize for Result<T, E> {}
// SAFETY: Both inner values of the `Result` are correctly marked.
unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    custom_trace!(this, {
        match *this {
            Ok(ref v) => mark(v),
            Err(ref v) => mark(v),
        }
    });
}

impl<T: Ord + Trace> Finalize for BinaryHeap<T> {}
// SAFETY: All the elements of the `BinaryHeap` are correctly marked.
unsafe impl<T: Ord + Trace> Trace for BinaryHeap<T> {
    custom_trace!(this, {
        for v in this.iter() {
            mark(v);
        }
    });
}

impl<K: Trace, V: Trace> Finalize for BTreeMap<K, V> {}
// SAFETY: All the elements of the `BTreeMap` are correctly marked.
unsafe impl<K: Trace, V: Trace> Trace for BTreeMap<K, V> {
    custom_trace!(this, {
        for (k, v) in this {
            mark(k);
            mark(v);
        }
    });
}

impl<T: Trace> Finalize for BTreeSet<T> {}
// SAFETY: All the elements of the `BTreeSet` are correctly marked.
unsafe impl<T: Trace> Trace for BTreeSet<T> {
    custom_trace!(this, {
        for v in this {
            mark(v);
        }
    });
}

impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Finalize for HashMap<K, V, S> {}
// SAFETY: All the elements of the `HashMap` are correctly marked.
unsafe impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Trace for HashMap<K, V, S> {
    custom_trace!(this, {
        for (k, v) in this.iter() {
            mark(k);
            mark(v);
        }
    });
}

impl<T: Eq + Hash + Trace, S: BuildHasher> Finalize for HashSet<T, S> {}
// SAFETY: All the elements of the `HashSet` are correctly marked.
unsafe impl<T: Eq + Hash + Trace, S: BuildHasher> Trace for HashSet<T, S> {
    custom_trace!(this, {
        for v in this.iter() {
            mark(v);
        }
    });
}

impl<T: Eq + Hash + Trace> Finalize for LinkedList<T> {}
// SAFETY: All the elements of the `LinkedList` are correctly marked.
unsafe impl<T: Eq + Hash + Trace> Trace for LinkedList<T> {
    custom_trace!(this, {
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
    custom_trace!(this, {
        for v in this.iter() {
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
    custom_trace!(this, {
        if let Cow::Owned(ref v) = this {
            mark(v);
        }
    });
}
