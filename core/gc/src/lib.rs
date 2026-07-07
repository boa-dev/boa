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

extern crate self as boa_gc;

#[cfg(not(feature = "oscars_backend"))]
mod cell;
#[cfg(not(feature = "oscars_backend"))]
mod pointers;
#[cfg(not(feature = "oscars_backend"))]
mod trace;

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
pub use pointers::{Ephemeron, Gc, GcErased, WeakGc, WeakMap};

#[cfg(feature = "oscars_backend")]
pub use oscars::null_collector_branded::{
    Ephemeron, Finalize, Gc, GcRefCell, MutationContext, Root, Trace, Tracer, WeakGc,
};

#[cfg(not(feature = "oscars_backend"))]
pub(crate) mod boa_allocator;

#[cfg(not(feature = "oscars_backend"))]
pub use boa_allocator::*;

#[cfg(all(test, not(feature = "oscars_backend")))]
mod test;
