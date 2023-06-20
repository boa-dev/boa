//! Boa's **`boa_gc`** crate implements a garbage collector.
//!
//! # Crate Overview
//! **`boa_gc`** is a mark-sweep garbage collector that implements a [`Trace`] and [`Finalize`] trait
//! for garbage collected values.
//!
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
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
    clippy::undocumented_unsafe_blocks
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]

extern crate self as boa_gc;

mod cell;
mod pointers;
mod trace;

pub(crate) mod internals;

use boa_profiler::Profiler;
use internals::{EphemeronBox, ErasedEphemeronBox, ErasedWeakMapBox, WeakMapBox};
use pointers::RawWeakMap;
use std::{
    cell::{Cell, RefCell},
    mem,
    ptr::NonNull,
};

pub use crate::trace::{Finalize, Trace};
pub use boa_macros::{Finalize, Trace};
pub use cell::{GcRef, GcRefCell, GcRefMut};
pub use internals::GcBox;
pub use pointers::{Ephemeron, Gc, WeakGc, WeakMap};

type GcPointer = NonNull<GcBox<dyn Trace>>;
type EphemeronPointer = NonNull<dyn ErasedEphemeronBox>;
type ErasedWeakMapBoxPointer = NonNull<dyn ErasedWeakMapBox>;

thread_local!(static GC_DROPPING: Cell<bool> = Cell::new(false));
thread_local!(static BOA_GC: RefCell<BoaGc> = RefCell::new( BoaGc {
    config: GcConfig::default(),
    runtime: GcRuntimeData::default(),
    strong_start: Cell::new(None),
    weak_start: Cell::new(None),
    weak_map_start: Cell::new(None),
}));

#[derive(Debug, Clone, Copy)]
struct GcConfig {
    threshold: usize,
    used_space_percentage: usize,
}

// Setting the defaults to an arbitrary value currently.
//
// TODO: Add a configure later
impl Default for GcConfig {
    fn default() -> Self {
        Self {
            threshold: 1024,
            used_space_percentage: 80,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct GcRuntimeData {
    collections: usize,
    bytes_allocated: usize,
}

#[derive(Debug)]
struct BoaGc {
    config: GcConfig,
    runtime: GcRuntimeData,
    strong_start: Cell<Option<GcPointer>>,
    weak_start: Cell<Option<EphemeronPointer>>,
    weak_map_start: Cell<Option<ErasedWeakMapBoxPointer>>,
}

impl Drop for BoaGc {
    fn drop(&mut self) {
        Collector::dump(self);
    }
}

// Whether or not the thread is currently in the sweep phase of garbage collection.
// During this phase, attempts to dereference a `Gc<T>` pointer will trigger a panic.
/// `DropGuard` flags whether the Collector is currently running `Collector::sweep()` or `Collector::dump()`
///
/// While the `DropGuard` is active, all `GcBox`s must not be dereferenced or accessed as it could cause Undefined Behavior
#[derive(Debug, Clone)]
struct DropGuard;

impl DropGuard {
    fn new() -> Self {
        GC_DROPPING.with(|dropping| dropping.set(true));
        Self
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        GC_DROPPING.with(|dropping| dropping.set(false));
    }
}

/// Returns `true` if it is safe for a type to run [`Finalize::finalize`].
#[must_use]
#[inline]
pub fn finalizer_safe() -> bool {
    GC_DROPPING.with(|dropping| !dropping.get())
}

/// The Allocator handles allocation of garbage collected values.
///
/// The allocator can trigger a garbage collection.
#[derive(Debug, Clone, Copy)]
struct Allocator;

impl Allocator {
    /// Allocate a new garbage collected value to the Garbage Collector's heap.
    fn alloc_gc<T: Trace>(value: GcBox<T>) -> NonNull<GcBox<T>> {
        let _timer = Profiler::global().start_event("New GcBox", "BoaAlloc");
        let element_size = mem::size_of_val::<GcBox<T>>(&value);
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            Self::manage_state(&mut gc);
            value.header.next.set(gc.strong_start.take());
            // Safety: value cannot be a null pointer, since `Box` cannot return null pointers.
            let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(value))) };
            let erased: NonNull<GcBox<dyn Trace>> = ptr;

            gc.strong_start.set(Some(erased));
            gc.runtime.bytes_allocated += element_size;

            ptr
        })
    }

    fn alloc_ephemeron<K: Trace + ?Sized, V: Trace>(
        value: EphemeronBox<K, V>,
    ) -> NonNull<EphemeronBox<K, V>> {
        let _timer = Profiler::global().start_event("New EphemeronBox", "BoaAlloc");
        let element_size = mem::size_of_val::<EphemeronBox<K, V>>(&value);
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            Self::manage_state(&mut gc);
            value.header.next.set(gc.weak_start.take());
            // Safety: value cannot be a null pointer, since `Box` cannot return null pointers.
            let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(value))) };
            let erased: NonNull<dyn ErasedEphemeronBox> = ptr;

            gc.weak_start.set(Some(erased));
            gc.runtime.bytes_allocated += element_size;

            ptr
        })
    }

    fn alloc_weak_map<K: Trace, V: Trace + Clone>() -> WeakMap<K, V> {
        let _timer = Profiler::global().start_event("New WeakMap", "BoaAlloc");

        let weak_map = WeakMap {
            inner: Gc::new(GcRefCell::new(RawWeakMap::new())),
        };
        let weak = WeakGc::new(&weak_map.inner);

        BOA_GC.with(|st| {
            let gc = st.borrow_mut();

            let weak_box = WeakMapBox {
                map: weak,
                next: Cell::new(gc.weak_map_start.take()),
            };

            // Safety: value cannot be a null pointer, since `Box` cannot return null pointers.
            let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(weak_box))) };
            let erased: ErasedWeakMapBoxPointer = ptr;

            gc.weak_map_start.set(Some(erased));

            weak_map
        })
    }

    fn manage_state(gc: &mut BoaGc) {
        if gc.runtime.bytes_allocated > gc.config.threshold {
            Collector::collect(gc);

            if gc.runtime.bytes_allocated
                > gc.config.threshold / 100 * gc.config.used_space_percentage
            {
                gc.config.threshold =
                    gc.runtime.bytes_allocated / gc.config.used_space_percentage * 100;
            }
        }
    }
}

struct Unreachables {
    strong: Vec<NonNull<GcBox<dyn Trace>>>,
    weak: Vec<NonNull<dyn ErasedEphemeronBox>>,
}

/// This collector currently functions in four main phases
///
/// Mark -> Finalize -> Mark -> Sweep
///
/// 1. Mark nodes as reachable.
/// 2. Finalize the unreachable nodes.
/// 3. Mark again because `Finalize::finalize` can potentially resurrect dead nodes.
/// 4. Sweep and drop all dead nodes.
///
/// A better approach in a more concurrent structure may be to reorder.
///
/// Mark -> Sweep -> Finalize
struct Collector;

impl Collector {
    /// Run a collection on the full heap.
    fn collect(gc: &mut BoaGc) {
        let _timer = Profiler::global().start_event("Gc Full Collection", "gc");
        gc.runtime.collections += 1;

        Self::trace_non_roots(gc);

        let unreachables = Self::mark_heap(&gc.strong_start, &gc.weak_start, &gc.weak_map_start);

        // Only finalize if there are any unreachable nodes.
        if !unreachables.strong.is_empty() || !unreachables.weak.is_empty() {
            // Finalize all the unreachable nodes.
            // SAFETY: All passed pointers are valid, since we won't deallocate until `Self::sweep`.
            unsafe { Self::finalize(unreachables) };

            let _final_unreachables =
                Self::mark_heap(&gc.strong_start, &gc.weak_start, &gc.weak_map_start);
        }

        // SAFETY: The head of our linked list is always valid per the invariants of our GC.
        unsafe {
            Self::sweep(
                &gc.strong_start,
                &gc.weak_start,
                &mut gc.runtime.bytes_allocated,
            );
        }

        // Weak maps have to be cleared after the sweep, since the process dereferences GcBoxes.
        let mut weak_map = &gc.weak_map_start;
        while let Some(w) = weak_map.get() {
            // SAFETY: The caller must ensure the validity of every node of `heap_start`.
            let node_ref = unsafe { w.as_ref() };

            if node_ref.is_live() {
                node_ref.clear_dead_entries();
                weak_map = node_ref.next();
            } else {
                weak_map.set(node_ref.next().take());

                // SAFETY:
                // The `Allocator` must always ensure its start node is a valid, non-null pointer that
                // was allocated by `Box::from_raw(Box::new(..))`.
                let _unmarked_node = unsafe { Box::from_raw(w.as_ptr()) };
            }
        }
    }

    fn trace_non_roots(gc: &mut BoaGc) {
        // Count all the handles located in GC heap.
        // Then, we can find whether there is a reference from other places, and they are the roots.
        let mut strong = &gc.strong_start;
        while let Some(node) = strong.get() {
            // SAFETY: node must be valid as this phase cannot drop any node.
            let node_ref = unsafe { node.as_ref() };
            node_ref.value().trace_non_roots();
            strong = &node_ref.header.next;
        }

        let mut weak = &gc.weak_start;
        while let Some(eph) = weak.get() {
            // SAFETY: node must be valid as this phase cannot drop any node.
            let eph_ref = unsafe { eph.as_ref() };
            eph_ref.trace_non_roots();
            weak = &eph_ref.header().next;
        }
    }

    /// Walk the heap and mark any nodes deemed reachable
    fn mark_heap(
        mut strong: &Cell<Option<NonNull<GcBox<dyn Trace>>>>,
        mut weak: &Cell<Option<NonNull<dyn ErasedEphemeronBox>>>,
        mut weak_map: &Cell<Option<ErasedWeakMapBoxPointer>>,
    ) -> Unreachables {
        let _timer = Profiler::global().start_event("Gc Marking", "gc");

        // Walk the list, tracing and marking the nodes
        let mut strong_dead = Vec::new();
        let mut pending_ephemerons = Vec::new();

        // === Preliminary mark phase ===
        //
        // 0. Get the naive list of possibly dead nodes.
        while let Some(node) = strong.get() {
            // SAFETY: node must be valid as this phase cannot drop any node.
            let node_ref = unsafe { node.as_ref() };
            if node_ref.get_non_root_count() < node_ref.get_ref_count() {
                // SAFETY: the gc heap object should be alive if there is a root.
                unsafe {
                    node_ref.mark_and_trace();
                }
            } else if !node_ref.is_marked() {
                strong_dead.push(node);
            }
            strong = &node_ref.header.next;
        }

        // 0.1. Early return if there are no ephemerons in the GC
        if weak.get().is_none() {
            strong_dead.retain_mut(|node| {
                // SAFETY: node must be valid as this phase cannot drop any node.
                unsafe { !node.as_ref().is_marked() }
            });
            return Unreachables {
                strong: strong_dead,
                weak: Vec::new(),
            };
        }

        // === Weak mark phase ===
        //
        //
        // 1. Get the naive list of ephemerons that are supposedly dead or their key is dead and
        // trace all the ephemerons that have roots and their keys are live. Also remove from
        // this list the ephemerons that are marked but their value is dead.
        while let Some(eph) = weak.get() {
            // SAFETY: node must be valid as this phase cannot drop any node.
            let eph_ref = unsafe { eph.as_ref() };
            let header = eph_ref.header();
            if header.get_non_root_count() < header.get_ref_count() {
                header.mark();
            }
            // SAFETY: the garbage collector ensures `eph_ref` always points to valid data.
            if unsafe { !eph_ref.trace() } {
                pending_ephemerons.push(eph);
            }
            weak = &header.next;
        }

        // 2. Trace all the weak pointers in the live weak maps to make sure they do not get swept.
        while let Some(w) = weak_map.get() {
            // SAFETY: node must be valid as this phase cannot drop any node.
            let node_ref = unsafe { w.as_ref() };

            // SAFETY: The garbage collector ensures that all nodes are valid.
            unsafe { node_ref.trace() };

            weak_map = node_ref.next();
        }

        // 3. Iterate through all pending ephemerons, removing the ones which have been successfully
        // traced. If there are no changes in the pending ephemerons list, it means that there are no
        // more reachable ephemerons from the remaining ephemeron values.
        let mut previous_len = pending_ephemerons.len();
        loop {
            pending_ephemerons.retain_mut(|eph| {
                // SAFETY: node must be valid as this phase cannot drop any node.
                let eph_ref = unsafe { eph.as_ref() };
                // SAFETY: the garbage collector ensures `eph_ref` always points to valid data.
                unsafe { !eph_ref.trace() }
            });

            if previous_len == pending_ephemerons.len() {
                break;
            }

            previous_len = pending_ephemerons.len();
        }

        // 4. The remaining list should contain the ephemerons that are either unreachable or its key
        // is dead. Cleanup the strong pointers since this procedure could have marked some more strong
        // pointers.
        strong_dead.retain_mut(|node| {
            // SAFETY: node must be valid as this phase cannot drop any node.
            unsafe { !node.as_ref().is_marked() }
        });

        Unreachables {
            strong: strong_dead,
            weak: pending_ephemerons,
        }
    }

    /// # Safety
    ///
    /// Passing a `strong` or a `weak` vec with invalid pointers will result in Undefined Behaviour.
    unsafe fn finalize(unreachables: Unreachables) {
        let _timer = Profiler::global().start_event("Gc Finalization", "gc");
        for node in unreachables.strong {
            // SAFETY: The caller must ensure all pointers inside `unreachables.strong` are valid.
            let node = unsafe { node.as_ref() };
            Trace::run_finalizer(node.value());
        }
        for node in unreachables.weak {
            // SAFETY: The caller must ensure all pointers inside `unreachables.weak` are valid.
            let node = unsafe { node.as_ref() };
            node.finalize_and_clear();
        }
    }

    /// # Safety
    ///
    /// - Providing an invalid pointer in the `heap_start` or in any of the headers of each
    /// node will result in Undefined Behaviour.
    /// - Providing a list of pointers that weren't allocated by `Box::into_raw(Box::new(..))`
    /// will result in Undefined Behaviour.
    unsafe fn sweep(
        mut strong: &Cell<Option<NonNull<GcBox<dyn Trace>>>>,
        mut weak: &Cell<Option<NonNull<dyn ErasedEphemeronBox>>>,
        total_allocated: &mut usize,
    ) {
        let _timer = Profiler::global().start_event("Gc Sweeping", "gc");
        let _guard = DropGuard::new();

        while let Some(node) = strong.get() {
            // SAFETY: The caller must ensure the validity of every node of `heap_start`.
            let node_ref = unsafe { node.as_ref() };
            if node_ref.is_marked() {
                node_ref.header.unmark();
                node_ref.reset_non_root_count();
                strong = &node_ref.header.next;
            } else {
                // SAFETY: The algorithm ensures only unmarked/unreachable pointers are dropped.
                // The caller must ensure all pointers were allocated by `Box::into_raw(Box::new(..))`.
                let unmarked_node = unsafe { Box::from_raw(node.as_ptr()) };
                let unallocated_bytes = mem::size_of_val(&*unmarked_node);
                *total_allocated -= unallocated_bytes;
                strong.set(unmarked_node.header.next.take());
            }
        }

        while let Some(eph) = weak.get() {
            // SAFETY: The caller must ensure the validity of every node of `heap_start`.
            let eph_ref = unsafe { eph.as_ref() };
            let header = eph_ref.header();
            if header.is_marked() {
                header.unmark();
                header.reset_non_root_count();
                weak = &header.next;
            } else {
                // SAFETY: The algorithm ensures only unmarked/unreachable pointers are dropped.
                // The caller must ensure all pointers were allocated by `Box::into_raw(Box::new(..))`.
                let unmarked_eph = unsafe { Box::from_raw(eph.as_ptr()) };
                let unallocated_bytes = mem::size_of_val(&*unmarked_eph);
                *total_allocated -= unallocated_bytes;
                weak.set(unmarked_eph.header().next.take());
            }
        }
    }

    // Clean up the heap when BoaGc is dropped
    fn dump(gc: &mut BoaGc) {
        // Weak maps have to be dropped first, since the process dereferences GcBoxes.
        // This can be done without initializing a dropguard since no GcBox's are being dropped.
        let weak_map_head = &gc.weak_map_start;
        while let Some(node) = weak_map_head.get() {
            // SAFETY:
            // The `Allocator` must always ensure its start node is a valid, non-null pointer that
            // was allocated by `Box::from_raw(Box::new(..))`.
            let unmarked_node = unsafe { Box::from_raw(node.as_ptr()) };
            weak_map_head.set(unmarked_node.next().take());
        }

        // Not initializing a dropguard since this should only be invoked when BOA_GC is being dropped.
        let _guard = DropGuard::new();

        let strong_head = &gc.strong_start;
        while let Some(node) = strong_head.get() {
            // SAFETY:
            // The `Allocator` must always ensure its start node is a valid, non-null pointer that
            // was allocated by `Box::from_raw(Box::new(..))`.
            let unmarked_node = unsafe { Box::from_raw(node.as_ptr()) };
            strong_head.set(unmarked_node.header.next.take());
        }

        let eph_head = &gc.weak_start;
        while let Some(node) = eph_head.get() {
            // SAFETY:
            // The `Allocator` must always ensure its start node is a valid, non-null pointer that
            // was allocated by `Box::from_raw(Box::new(..))`.
            let unmarked_node = unsafe { Box::from_raw(node.as_ptr()) };
            eph_head.set(unmarked_node.header().next.take());
        }
    }
}

/// Forcefully runs a garbage collection of all unaccessible nodes.
pub fn force_collect() {
    BOA_GC.with(|current| {
        let mut gc = current.borrow_mut();

        if gc.runtime.bytes_allocated > 0 {
            Collector::collect(&mut gc);
        }
    });
}

#[cfg(test)]
mod test;

/// Returns `true` is any weak maps are currently allocated.
#[cfg(test)]
#[must_use]
pub fn has_weak_maps() -> bool {
    BOA_GC.with(|current| {
        let gc = current.borrow();

        gc.weak_map_start.get().is_some()
    })
}
