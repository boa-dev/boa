use std::borrow::{Cow, ToOwned};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize,
};

use crate::GcPointer;

/// The Finalize trait, which needs to be implemented on
/// garbage-collected objects to define finalization logic.
pub trait Finalize {
    fn finalize(&self) {}
}

/// The Trace trait, which needs to be implemented on garbage-collected objects.
pub unsafe trait Trace: Finalize {
    /// Marks all contained `Gc`s.
    unsafe fn trace(&self);

    /// Checks if an ephemeron's key is marked.
    ///
    /// Note: value should always be implemented to return false
    unsafe fn is_marked_ephemeron(&self) -> bool;

    /// Returns true if a marked `Gc` is found
    unsafe fn weak_trace(&self, ephemeron_queue: &mut Vec<GcPointer>);

    /// Increments the root-count of all contained `Gc`s.
    unsafe fn root(&self);

    /// Decrements the root-count of all contained `Gc`s.
    unsafe fn unroot(&self);

    /// Runs Finalize::finalize() on this object and all
    /// contained subobjects
    fn run_finalizer(&self);
}

/// This rule implements the trace methods with empty implementations.
///
/// Use this for marking types as not containing any `Trace` types.
#[macro_export]
macro_rules! unsafe_empty_trace {
    () => {
        #[inline]
        unsafe fn trace(&self) {}
        #[inline]
        unsafe fn is_marked_ephemeron(&self) -> bool {
            false
        }
        #[inline]
        unsafe fn weak_trace(&self, _ephemeron_queue: &mut Vec<GcPointer>) {}
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

/// This rule implements the trace method.
///
/// You define a `this` parameter name and pass in a body, which should call `mark` on every
/// traceable element inside the body. The mark implementation will automatically delegate to the
/// correct method on the argument.
#[macro_export]
macro_rules! custom_trace {
    ($this:ident, $op:ident, $body:expr, $weak_body:expr) => {
        #[inline]
        unsafe fn trace(&self) {
            #[inline]
            unsafe fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::trace(it);
            }
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn is_marked_ephemeron(&self) -> bool {
            false
        }
        #[inline]
        unsafe fn weak_trace(&self, queue: &mut Vec<GcPointer>) {
            #[inline]
            unsafe fn mark<T: $crate::Trace + ?Sized>(it: &T, queue: &mut Vec<GcPointer>) {
                $crate::Trace::weak_trace(it, queue)
            }
            let $this = self;
            let $op = queue;
            $weak_body
        }
        #[inline]
        unsafe fn root(&self) {
            #[inline]
            unsafe fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::root(it);
            }
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn unroot(&self) {
            #[inline]
            unsafe fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::unroot(it);
            }
            let $this = self;
            $body
        }
        #[inline]
        fn run_finalizer(&self) {
            $crate::Finalize::finalize(self);
            #[inline]
            fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::run_finalizer(it);
            }
            let $this = self;
            $body
        }
    };
}

impl<T: ?Sized> Finalize for &'static T {}
unsafe impl<T: ?Sized> Trace for &'static T {
    unsafe_empty_trace!();
}

macro_rules! simple_empty_finalize_trace {
    ($($T:ty),*) => {
        $(
            impl Finalize for $T {}
            unsafe impl Trace for $T { unsafe_empty_trace!(); }
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
unsafe impl<T: Trace, const N: usize> Trace for [T; N] {
    custom_trace!(
        this,
        queue,
        {
            for v in this {
                mark(v);
            }
        },
        {
            for v in this {
                mark(v, queue);
            }
        }
    );
}

macro_rules! fn_finalize_trace_one {
    ($ty:ty $(,$args:ident)*) => {
        impl<Ret $(,$args)*> Finalize for $ty {}
        unsafe impl<Ret $(,$args)*> Trace for $ty { unsafe_empty_trace!(); }
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
        unsafe impl<$($args: $crate::Trace),*> Trace for ($($args,)*) {
            custom_trace!(this, queue, {
                #[allow(non_snake_case, unused_unsafe)]
                fn avoid_lints<$($args: $crate::Trace),*>(&($(ref $args,)*): &($($args,)*)) {
                    unsafe { $(mark($args);)* }
                }
                avoid_lints(this)
            }, {
                #[allow(non_snake_case, unused_unsafe)]
                fn avoid_lints<$($args: $crate::Trace),*>(&($(ref $args,)*): &($($args,)*), queue: &mut Vec<GcPointer>) {
                    unsafe { $(mark($args, queue);)* }
                }
                avoid_lints(this, queue)
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

impl<T: Trace + ?Sized> Finalize for Rc<T> {}
unsafe impl<T: Trace + ?Sized> Trace for Rc<T> {
    custom_trace!(
        this,
        queue,
        {
            mark(&**this);
        },
        mark(&**this, queue)
    );
}

impl<T: Trace> Finalize for Rc<[T]> {}
unsafe impl<T: Trace> Trace for Rc<[T]> {
    custom_trace!(
        this,
        queue,
        {
            for e in this.iter() {
                mark(e);
            }
        },
        {
            for e in this.iter() {
                mark(e, queue);
            }
        }
    );
}

impl<T: Trace + ?Sized> Finalize for Box<T> {}
unsafe impl<T: Trace + ?Sized> Trace for Box<T> {
    custom_trace!(
        this,
        queue,
        {
            mark(&**this);
        },
        mark(&**this, queue)
    );
}

impl<T: Trace> Finalize for Box<[T]> {}
unsafe impl<T: Trace> Trace for Box<[T]> {
    custom_trace!(
        this,
        queue,
        {
            for e in this.iter() {
                mark(e);
            }
        },
        {
            for e in this.iter() {
                mark(e, queue);
            }
        }
    );
}

impl<T: Trace> Finalize for Vec<T> {}
unsafe impl<T: Trace> Trace for Vec<T> {
    custom_trace!(
        this,
        queue,
        {
            for e in this {
                mark(e);
            }
        },
        {
            for e in this {
                mark(e, queue);
            }
        }
    );
}

impl<T: Trace> Finalize for Option<T> {}
unsafe impl<T: Trace> Trace for Option<T> {
    custom_trace!(
        this,
        queue,
        {
            if let Some(ref v) = *this {
                mark(v);
            }
        },
        {
            if let Some(ref v) = *this {
                mark(v, queue)
            }
        }
    );
}

impl<T: Trace, E: Trace> Finalize for Result<T, E> {}
unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    custom_trace!(
        this,
        queue,
        {
            match *this {
                Ok(ref v) => mark(v),
                Err(ref v) => mark(v),
            }
        },
        {
            let marked = match *this {
                Ok(ref v) => mark(v, queue),
                Err(ref v) => mark(v, queue),
            };
            marked
        }
    );
}

impl<T: Ord + Trace> Finalize for BinaryHeap<T> {}
unsafe impl<T: Ord + Trace> Trace for BinaryHeap<T> {
    custom_trace!(
        this,
        queue,
        {
            for v in this.iter() {
                mark(v);
            }
        },
        {
            for e in this.iter() {
                mark(e, queue);
            }
        }
    );
}

impl<K: Trace, V: Trace> Finalize for BTreeMap<K, V> {}
unsafe impl<K: Trace, V: Trace> Trace for BTreeMap<K, V> {
    custom_trace!(
        this,
        queue,
        {
            for (k, v) in this {
                mark(k);
                mark(v);
            }
        },
        {
            for (k, v) in this {
                mark(k, queue);
                mark(v, queue);
            }
        }
    );
}

impl<T: Trace> Finalize for BTreeSet<T> {}
unsafe impl<T: Trace> Trace for BTreeSet<T> {
    custom_trace!(
        this,
        queue,
        {
            for v in this {
                mark(v);
            }
        },
        {
            for v in this {
                mark(v, queue);
            }
        }
    );
}

impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Finalize for HashMap<K, V, S> {}
unsafe impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Trace for HashMap<K, V, S> {
    custom_trace!(
        this,
        queue,
        {
            for (k, v) in this.iter() {
                mark(k);
                mark(v);
            }
        },
        {
            for (k, v) in this.iter() {
                mark(k, queue);
                mark(v, queue);
            }
        }
    );
}

impl<T: Eq + Hash + Trace, S: BuildHasher> Finalize for HashSet<T, S> {}
unsafe impl<T: Eq + Hash + Trace, S: BuildHasher> Trace for HashSet<T, S> {
    custom_trace!(
        this,
        queue,
        {
            for v in this.iter() {
                mark(v);
            }
        },
        {
            for v in this.iter() {
                mark(v, queue);
            }
        }
    );
}

impl<T: Eq + Hash + Trace> Finalize for LinkedList<T> {}
unsafe impl<T: Eq + Hash + Trace> Trace for LinkedList<T> {
    custom_trace!(
        this,
        queue,
        {
            for v in this.iter() {
                mark(v);
            }
        },
        {
            for v in this.iter() {
                mark(v, queue);
            }
        }
    );
}

impl<T> Finalize for PhantomData<T> {}
unsafe impl<T> Trace for PhantomData<T> {
    unsafe_empty_trace!();
}

impl<T: Trace> Finalize for VecDeque<T> {}
unsafe impl<T: Trace> Trace for VecDeque<T> {
    custom_trace!(
        this,
        queue,
        {
            for v in this.iter() {
                mark(v);
            }
        },
        {
            for v in this.iter() {
                mark(v, queue);
            }
        }
    );
}

impl<'a, T: ToOwned + Trace + ?Sized> Finalize for Cow<'a, T> {}
unsafe impl<'a, T: ToOwned + Trace + ?Sized> Trace for Cow<'a, T>
where
    T::Owned: Trace,
{
    custom_trace!(
        this,
        queue,
        {
            if let Cow::Owned(ref v) = this {
                mark(v);
            }
        },
        {
            if let Cow::Owned(ref v) = this {
                mark(v, queue)
            }
        }
    );
}
