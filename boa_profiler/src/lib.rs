//! Profiler for the Boa JavaScript engine.

#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]

use std::fmt::{self, Debug};

#[cfg(feature = "profiler")]
use measureme::{EventId, Profiler as MeasuremeProfiler, TimingGuard};
#[cfg(feature = "profiler")]
use once_cell::sync::OnceCell;
#[cfg(feature = "profiler")]
use std::{
    path::Path,
    thread::{current, ThreadId},
};

/// Profiler for the Boa JavaScript engine.
#[cfg(feature = "profiler")]
pub struct Profiler {
    profiler: MeasuremeProfiler,
}

/// This static instance must never be public, and its only access must be done through the
/// `global()` and `drop()` methods. This is because `get_or_init` manages synchronization and the
/// case of an empty value.
#[cfg(feature = "profiler")]
static mut INSTANCE: OnceCell<Profiler> = OnceCell::new();

#[cfg(feature = "profiler")]
impl Profiler {
    /// Start a new profiled event.
    pub fn start_event(&self, label: &str, category: &str) -> TimingGuard<'_> {
        let kind = self.profiler.alloc_string(category);
        let id = EventId::from_label(self.profiler.alloc_string(label));
        let thread_id = Self::thread_id_to_u32(current().id());
        self.profiler
            .start_recording_interval_event(kind, id, thread_id)
    }

    fn default() -> Self {
        let profiler =
            MeasuremeProfiler::new(Path::new("./my_trace")).expect("must be able to create file");
        Self { profiler }
    }

    /// Return the global instance of the profiler.
    #[must_use]
    pub fn global() -> &'static Self {
        unsafe { INSTANCE.get_or_init(Self::default) }
    }

    /// Drop the global instance of the profiler.
    pub fn drop(&self) {
        // In order to drop the INSTANCE we need to get ownership of it, which isn't possible on a static unless you make it a mutable static
        // mutating statics is unsafe, so we need to wrap it as so.
        // This is actually safe though because init and drop are only called at the beginning and end of the application
        unsafe {
            INSTANCE
                .take()
                .expect("Could not take back profiler instance");
        }
    }

    // Sadly we need to use the unsafe method until this is resolved:
    // https://github.com/rust-lang/rust/issues/67939
    // Once `as_64()` is in stable we can do this:
    // https://github.com/rust-lang/rust/pull/68531/commits/ea42b1c5b85f649728e3a3b334489bac6dce890a
    // Until then our options are: use rust-nightly or use unsafe {}
    #[allow(clippy::cast_possible_truncation)]
    fn thread_id_to_u32(tid: ThreadId) -> u32 {
        unsafe { std::mem::transmute::<ThreadId, u64>(tid) as u32 }
    }
}

impl Debug for Profiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt("no debug implemented", f)
    }
}

/// An empty profiler that does nothing.
#[cfg(not(feature = "profiler"))]
#[derive(Copy, Clone)]
pub struct Profiler;

//#[allow(clippy::unused_unit, clippy::unused_self)]
#[cfg(not(feature = "profiler"))]
impl Profiler {
    /// Does nothing.
    #[allow(clippy::unused_unit)]
    pub const fn start_event(&self, _label: &str, _category: &str) -> () {}

    /// Does nothing.
    pub const fn drop(&self) {}

    /// Does nothing.
    #[must_use]
    pub const fn global() -> Self {
        Self
    }
}
