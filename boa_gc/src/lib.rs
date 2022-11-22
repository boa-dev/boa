//! Garbage collector for the Boa JavaScript engine.

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
use internals::GcBox;
use std::{
    cell::{Cell, RefCell},
    mem,
    ptr::NonNull,
};

pub use crate::trace::{Finalize, Trace};
pub use boa_macros::{Finalize, Trace};
pub use cell::{GcCell, GcCellRef, GcCellRefMut};
pub use pointers::{Ephemeron, Gc, WeakGc};

type GcPointer = NonNull<GcBox<dyn Trace>>;

thread_local!(static EPHEMERON_QUEUE: Cell<Option<Vec<GcPointer>>> = Cell::new(None));
thread_local!(static GC_DROPPING: Cell<bool> = Cell::new(false));
thread_local!(static BOA_GC: RefCell<BoaGc> = RefCell::new( BoaGc {
    config: GcConfig::default(),
    runtime: GcRuntimeData::default(),
    adult_start: Cell::new(None),
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
    adult_start: Cell<Option<GcPointer>>,
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
    fn allocate<T: Trace>(value: GcBox<T>) -> NonNull<GcBox<T>> {
        let _timer = Profiler::global().start_event("New Pointer", "BoaAlloc");
        let element_size = mem::size_of_val::<GcBox<T>>(&value);
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            Self::manage_state(&mut gc);
            value.header.next.set(gc.adult_start.take());
            // Safety: Value Cannot be a null as it must be a GcBox<T>
            let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::from(value))) };

            gc.adult_start.set(Some(ptr));
            gc.runtime.bytes_allocated += element_size;

            ptr
        })
    }

    fn manage_state(gc: &mut BoaGc) {
        if gc.runtime.bytes_allocated > gc.config.threshold {
            Collector::run_full_collection(gc);

            if gc.runtime.bytes_allocated
                > gc.config.threshold / 100 * gc.config.used_space_percentage
            {
                gc.config.threshold =
                    gc.runtime.bytes_allocated / gc.config.used_space_percentage * 100;
            }
        }
    }
}

// This collector currently functions in four main phases
//
// Mark -> Finalize -> Mark -> Sweep
//
// Mark nodes as reachable then finalize the unreachable nodes. A remark phase
// then needs to be retriggered as finalization can potentially resurrect dead
// nodes.
//
// A better approach in a more concurrent structure may be to reorder.
//
// Mark -> Sweep -> Finalize
struct Collector;

impl Collector {
    /// Run a collection on the full heap.
    fn run_full_collection(gc: &mut BoaGc) {
        let _timer = Profiler::global().start_event("Gc Full Collection", "gc");
        gc.runtime.collections += 1;
        let unreachable_adults = Self::mark_heap(&gc.adult_start);

        // Check if any unreachable nodes were found and finalize
        if !unreachable_adults.is_empty() {
            // SAFETY: Please see `Collector::finalize()`
            unsafe { Self::finalize(unreachable_adults) };
        }

        let _final_unreachable_adults = Self::mark_heap(&gc.adult_start);

        // SAFETY: Please see `Collector::sweep()`
        unsafe {
            Self::sweep(&gc.adult_start, &mut gc.runtime.bytes_allocated);
        }
    }

    /// Walk the heap and mark any nodes deemed reachable
    fn mark_heap(head: &Cell<Option<NonNull<GcBox<dyn Trace>>>>) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let _timer = Profiler::global().start_event("Gc Marking", "gc");
        // Walk the list, tracing and marking the nodes
        let mut finalize = Vec::new();
        let mut ephemeron_queue = Vec::new();
        let mut mark_head = head;
        while let Some(node) = mark_head.get() {
            // SAFETY: node must be valid as it is coming directly from the heap.
            let node_ref = unsafe { node.as_ref() };
            if node_ref.header.is_ephemeron() {
                ephemeron_queue.push(node);
            } else if node_ref.header.roots() > 0 {
                // SAFETY: the reference to node must be valid as it is rooted. Passing
                // invalid references can result in Undefined Behavior
                unsafe {
                    node_ref.trace_inner();
                }
            } else {
                finalize.push(node);
            }
            mark_head = &node_ref.header.next;
        }

        // Ephemeron Evaluation
        if !ephemeron_queue.is_empty() {
            ephemeron_queue = Self::mark_ephemerons(ephemeron_queue);
        }

        // Any left over nodes in the ephemeron queue at this point are
        // unreachable and need to be notified/finalized.
        finalize.extend(ephemeron_queue);

        finalize
    }

    // Tracing Ephemerons/Weak is always requires tracing the inner nodes in case it ends up marking unmarked node
    //
    // Time complexity should be something like O(nd) where d is the longest chain of epehemerons
    /// Mark any ephemerons that are deemed live and trace their fields.
    fn mark_ephemerons(
        initial_queue: Vec<NonNull<GcBox<dyn Trace>>>,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let mut ephemeron_queue = initial_queue;
        loop {
            // iterate through ephemeron queue, sorting nodes by whether they
            // are reachable or unreachable<?>
            let (reachable, other): (Vec<_>, Vec<_>) =
                ephemeron_queue.into_iter().partition(|node| {
                    // SAFETY: Any node on the eph_queue or the heap must be non null
                    let node = unsafe { node.as_ref() };
                    if node.value.is_marked_ephemeron() {
                        node.header.mark();
                        true
                    } else {
                        node.header.roots() > 0
                    }
                });
            // Replace the old queue with the unreachable<?>
            ephemeron_queue = other;

            // If reachable nodes is not empty, trace values. If it is empty,
            // break from the loop
            if reachable.is_empty() {
                break;
            }
            EPHEMERON_QUEUE.with(|state| state.set(Some(Vec::new())));
            // iterate through reachable nodes and trace their values,
            // enqueuing any ephemeron that is found during the trace
            for node in reachable {
                // TODO: deal with fetch ephemeron_queue
                // SAFETY: Node must be a valid pointer or else it would not be deemed reachable.
                unsafe {
                    node.as_ref().weak_trace_inner();
                }
            }

            EPHEMERON_QUEUE.with(|st| {
                if let Some(found_nodes) = st.take() {
                    ephemeron_queue.extend(found_nodes);
                }
            });
        }
        ephemeron_queue
    }

    /// # Safety
    ///
    /// Passing a vec with invalid pointers will result in Undefined Behaviour.
    unsafe fn finalize(finalize_vec: Vec<NonNull<GcBox<dyn Trace>>>) {
        let _timer = Profiler::global().start_event("Gc Finalization", "gc");
        for node in finalize_vec {
            // We double check that the unreachable nodes are actually unreachable
            // prior to finalization as they could have been marked by a different
            // trace after initially being added to the queue
            //
            // SAFETY: The caller must ensure all pointers inside `finalize_vec` are valid.
            let node = unsafe { node.as_ref() };
            if !node.header.is_marked() {
                Trace::run_finalizer(&node.value);
            }
        }
    }

    /// # Safety
    ///
    /// - Providing an invalid pointer in the `heap_start` or in any of the headers of each
    /// node will result in Undefined Behaviour.
    /// - Providing a list of pointers that weren't allocated by `Box::into_raw(Box::new(..))`
    /// will result in Undefined Behaviour.
    unsafe fn sweep(
        heap_start: &Cell<Option<NonNull<GcBox<dyn Trace>>>>,
        total_allocated: &mut usize,
    ) {
        let _timer = Profiler::global().start_event("Gc Sweeping", "gc");
        let _guard = DropGuard::new();

        let mut sweep_head = heap_start;
        while let Some(node) = sweep_head.get() {
            // SAFETY: The caller must ensure the validity of every node of `heap_start`.
            let node_ref = unsafe { node.as_ref() };
            if node_ref.is_marked() {
                node_ref.header.unmark();
                sweep_head = &node_ref.header.next;
            } else if node_ref.header.is_ephemeron() && node_ref.header.roots() > 0 {
                // Keep the ephemeron box's alive if rooted, but note that it's pointer is no longer safe
                Trace::run_finalizer(&node_ref.value);
                sweep_head = &node_ref.header.next;
            } else {
                // SAFETY: The algorithm ensures only unmarked/unreachable pointers are dropped.
                // The caller must ensure all pointers were allocated by `Box::into_raw(Box::new(..))`.
                let unmarked_node = unsafe { Box::from_raw(node.as_ptr()) };
                let unallocated_bytes = mem::size_of_val::<GcBox<_>>(&*unmarked_node);
                *total_allocated -= unallocated_bytes;
                sweep_head.set(unmarked_node.header.next.take());
            }
        }
    }

    // Clean up the heap when BoaGc is dropped
    fn dump(gc: &mut BoaGc) {
        // Not initializing a dropguard since this should only be invoked when BOA_GC is being dropped.
        let _guard = DropGuard::new();

        let sweep_head = &gc.adult_start;
        while let Some(node) = sweep_head.get() {
            // SAFETY:
            // The `Allocator` must always ensure its start node is a valid, non-null pointer that
            // was allocated by `Box::from_raw(Box::new(..))`.
            let unmarked_node = unsafe { Box::from_raw(node.as_ptr()) };
            sweep_head.set(unmarked_node.header.next.take());
        }
    }
}

/// Forcefully runs a garbage collection of all unaccessible nodes.
pub fn force_collect() {
    BOA_GC.with(|current| {
        let mut gc = current.borrow_mut();

        if gc.runtime.bytes_allocated > 0 {
            Collector::run_full_collection(&mut gc);
        }
    });
}

#[cfg(test)]
mod test;
