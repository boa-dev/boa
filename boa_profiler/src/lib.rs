//! The **`boa_profiler`** crate is a code profiler for Boa.
//!
//! # Crate Overview
//! This crate provides a code profiler for Boa. For more information, please
//! see Boa's page on [profiling][profiler-md]
//!
//! # About Boa
//! Boa is an open-source, experimental ECMAScript Engine written in Rust for lexing, parsing and executing ECMAScript/JavaScript. Currently, Boa
//! supports some of the [language][boa-conformance]. More information can be viewed at [Boa's website][boa-web].
//!
//! Try out the most recent release with Boa's live demo [playground][boa-playground].  
//!
//! # Boa Crates
//!  - **`boa_ast`** - Boa's ECMAScript Abstract Syntax Tree.
//!  - **`boa_engine`** - Boa's implementation of ECMAScript builtin objects and execution.
//!  - **`boa_gc`** - Boa's garbage collector
//!  - **`boa_interner`** - Boa's string interner
//!  - **`boa_parser`** - Boa's lexer and parser
//!  - **`boa_profiler`** - Boa's code profiler
//!  - **`boa_unicode`** - Boa's Unicode identifier
//!
//! [profiler-md]: https://github.com/boa-dev/boa/blob/main/docs/profiling.md
//! [boa-conformance]: https://boa-dev.github.io/boa/test262/
//! [boa-web]: https://boa-dev.github.io/
//! [boa-playground]: https://boa-dev.github.io/boa/playground/

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
use measureme::{EventId, Profiler as MeasuremeProfiler, StringId, TimingGuard};
#[cfg(feature = "profiler")]
use once_cell::sync::OnceCell;
#[cfg(feature = "profiler")]
use rustc_hash::FxHashMap;
#[cfg(feature = "profiler")]
use std::collections::hash_map::Entry;
#[cfg(feature = "profiler")]
use std::sync::RwLock;
#[cfg(feature = "profiler")]
use std::{
    path::Path,
    thread::{current, ThreadId},
};

/// Profiler for the Boa JavaScript engine.
#[cfg(feature = "profiler")]
pub struct Profiler {
    profiler: MeasuremeProfiler,
    string_cache: RwLock<FxHashMap<String, StringId>>,
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
        let kind = self.get_or_alloc_string(category);
        let id = EventId::from_label(self.get_or_alloc_string(label));
        let thread_id = Self::thread_id_to_u32(current().id());
        self.profiler
            .start_recording_interval_event(kind, id, thread_id)
    }

    fn get_or_alloc_string(&self, s: &str) -> StringId {
        {
            // Check the cache only with the read lock first.
            let cache = self
                .string_cache
                .read()
                .expect("Some writer panicked while holding an exclusive lock.");
            if let Some(id) = cache.get(s) {
                return *id;
            }
        }
        let mut cache = self
            .string_cache
            .write()
            .expect("Some writer panicked while holding an exclusive lock.");
        let entry = cache.entry(s.into());
        match entry {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = self.profiler.alloc_string(s);
                *entry.insert(id)
            }
        }
    }

    fn default() -> Self {
        let profiler =
            MeasuremeProfiler::new(Path::new("./my_trace")).expect("must be able to create file");
        Self {
            profiler,
            string_cache: RwLock::new(FxHashMap::default()),
        }
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
