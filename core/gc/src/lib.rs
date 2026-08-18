//! Boa's **`boa_gc`** crate implements a garbage collector.
//!
//! # Crate Overview
//! **`boa_gc`** is a mark-sweep garbage collector that implements a [`Trace`] and [`Finalize`] trait
//! for garbage collected values.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]
#![allow(missing_docs)]
#![cfg_attr(
    feature = "oscars_backend",
    allow(unused_crate_dependencies, unused_extern_crates)
)]

extern crate self as boa_gc;

#[cfg(not(feature = "oscars_backend"))]
mod cell;
#[cfg(not(feature = "oscars_backend"))]
mod pointers;
#[cfg(not(feature = "oscars_backend"))]
mod trace;

pub mod context;
pub use context::GcContext;

#[cfg(not(feature = "oscars_backend"))]
pub(crate) mod internals;

#[cfg(not(feature = "oscars_backend"))]
use internals::{EphemeronBox, ErasedEphemeronBox, ErasedWeakMapBox, WeakMapBox};
#[cfg(not(feature = "oscars_backend"))]
use pointers::{NonTraceable, RawWeakMap};
#[cfg(not(feature = "oscars_backend"))]
use std::{
    cell::{Cell, RefCell},
    mem,
    ptr::NonNull,
};

#[cfg(not(feature = "oscars_backend"))]
pub use crate::trace::{Finalize, Trace, Tracer};
pub use boa_macros::{Finalize, Trace};
#[cfg(not(feature = "oscars_backend"))]
pub use cell::{GcRef, GcRefCell, GcRefMut};
#[cfg(not(feature = "oscars_backend"))]
pub use internals::GcBox;
#[cfg(not(feature = "oscars_backend"))]
pub use pointers::{Ephemeron, Gc, GcErased, MutationContext, WeakGc, WeakMap};

#[cfg(feature = "oscars_backend")]
pub use oscars::collectors::mark_sweep_branded::{Finalize, Gc, GcRefCell, Root, Trace, Tracer};

#[cfg(feature = "oscars_backend")]
/// Re-export [`typeid::of`].
///
/// Computes a [`std::any::TypeId`] compatible value for `T` without requiring `T: 'static`.
/// oscars collectors use this to stamp `GcBox` at allocation, ensuring consistent
/// type comparisons.
///
/// Use this instead of `std::any::TypeId::of::<T>()` for types with non-`'static`
/// branded lifetimes (like `'gc` or `'id`).
pub use typeid::of as type_id_of;

#[cfg(feature = "oscars_backend")]
/// Type alias for Ephemeron
pub type Ephemeron<K, V> = oscars::collectors::mark_sweep_branded::Ephemeron<'static, K, V>;

#[cfg(feature = "oscars_backend")]
/// A token granting permission to allocate into the GC arena.
/// Lifetimes are `'static` for the null collector but should be forwarded for `mark_sweep_branded`.
pub type MutationContext<'a, 'b> =
    oscars::collectors::mark_sweep_branded::MutationContext<'static, 'static>;

#[cfg(feature = "oscars_backend")]
/// Type alias for `WeakGc`
pub type WeakGc<T> = oscars::collectors::mark_sweep_branded::WeakGc<'static, T>;

#[cfg(feature = "oscars_backend")]
pub use oscars::collectors::mark_sweep_branded::cell::{GcRef, GcRefMut};

#[cfg(feature = "oscars_backend")]
mod oscars_weak_map;

#[cfg(feature = "oscars_backend")]
pub use oscars_weak_map::WeakMap;

#[cfg(feature = "oscars_backend")]
#[must_use]
/// Returns whether finalizer is safe
pub fn finalizer_safe() -> bool {
    true
}

#[cfg(feature = "oscars_backend")]
/// Implements an empty `Trace` trait for the specified types
#[macro_export]
macro_rules! empty_trace {
    () => {
        #[inline]
        unsafe fn trace(&self, _tracer: &mut $crate::Tracer<'_>) {}
        #[inline]
        unsafe fn trace_non_roots(&self) {}
        #[inline]
        fn run_finalizer(&self) {
            $crate::Finalize::finalize(self);
        }
    };
    ($($T:ty),* $(,)?) => {
        $(
            unsafe impl $crate::Trace for $T {
                $crate::empty_trace!();
            }
        )*
    };
}

#[cfg(feature = "oscars_backend")]
/// Macro for custom trace
#[macro_export]
macro_rules! custom_trace {
    ($this:ident, $mark:ident, $body:expr) => {
        #[inline]
        unsafe fn trace(&self, tracer: &mut $crate::Tracer<'_>) {
            let mut $mark = |it: &dyn $crate::Trace| {
                // SAFETY: implementor must ensure trace is correctly implemented
                unsafe {
                    $crate::Trace::trace(it, tracer);
                }
            };
            let $this = self;
            // SAFETY: The implementor must ensure the trace body is safe
            unsafe { $body }
        }
        #[inline]
        unsafe fn trace_non_roots(&self) {
            #[allow(non_snake_case)]
            fn $mark<T: $crate::Trace + ?Sized>(_it: &T) {
                // SAFETY: implementor must ensure trace is correctly implemented
                unsafe {
                    $crate::Trace::trace_non_roots(_it);
                }
            }
            let $this = self;
            // SAFETY: The implementor must ensure the trace body is safe
            unsafe { $body }
        }
        #[inline]
        fn run_finalizer(&self) {
            $crate::Finalize::finalize(self);
        }
    };
}

#[cfg(not(feature = "oscars_backend"))]
pub(crate) mod boa_allocator;

#[cfg(not(feature = "oscars_backend"))]
pub use boa_allocator::*;

#[cfg(all(test, not(feature = "oscars_backend")))]
mod test;

#[cfg(feature = "oscars_backend")]
/// Forces a garbage collection
pub fn force_collect() {
    let mc = MutationContext::global();
    mc.collect();
    crate::context::GcContext::new().force_collect();
}
