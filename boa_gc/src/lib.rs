//! Garbage collector for the Boa JavaScript engine.

#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_docs
)]
#![allow(clippy::let_unit_value, clippy::module_name_repetitions)]

extern crate self as boa_gc;

use boa_profiler::Profiler;
use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr::NonNull;

mod trace;

pub(crate) mod internals;

mod cell;
mod pointers;

pub use crate::trace::{Finalize, Trace};
pub use boa_macros::{Finalize, Trace};
pub use cell::{GcCell, GcCellRef, GcCellRefMut};
pub use pointers::{Ephemeron, Gc, WeakGc};

use internals::GcBox;

type GcPointer = NonNull<GcBox<dyn Trace>>;

thread_local!(static EPHEMERON_QUEUE: Cell<Option<Vec<GcPointer>>> = Cell::new(None));
thread_local!(static GC_DROPPING: Cell<bool> = Cell::new(false));
thread_local!(static BOA_GC: RefCell<BoaGc> = RefCell::new( BoaGc {
    config: GcConfig::default(),
    runtime: GcRuntimeData::default(),
    adult_start: Cell::new(None),
}));

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

#[derive(Default)]
struct GcRuntimeData {
    collections: usize,
    bytes_allocated: usize,
}

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

struct DropGuard;

impl DropGuard {
    fn new() -> DropGuard {
        GC_DROPPING.with(|dropping| dropping.set(true));
        DropGuard
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        GC_DROPPING.with(|dropping| dropping.set(false));
    }
}

/// Returns `true` if it is safe for a type to run [`Finalize::finalize`].
#[must_use]
pub fn finalizer_safe() -> bool {
    GC_DROPPING.with(|dropping| !dropping.get())
}

/// The Allocator handles allocation of garbage collected values.
///
/// The allocator can trigger a garbage collection
struct Allocator;

impl Allocator {
    fn allocate<T: Trace>(value: GcBox<T>) -> NonNull<GcBox<T>> {
        let _timer = Profiler::global().start_event("New Pointer", "BoaAlloc");
        let element_size = mem::size_of_val::<GcBox<T>>(&value);
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            Self::manage_state(&mut gc);
            value.header.next.set(gc.adult_start.take());
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
    fn run_full_collection(gc: &mut BoaGc) {
        let _timer = Profiler::global().start_event("Gc Full Collection", "gc");
        gc.runtime.collections += 1;
        let unreachable_adults = unsafe { Self::mark_heap(&gc.adult_start) };

        // Check if any unreachable nodes were found and finalize
        if !unreachable_adults.is_empty() {
            unsafe { Self::finalize(unreachable_adults) };
        }

        let _final_unreachable_adults = unsafe { Self::mark_heap(&gc.adult_start) };

        unsafe {
            Self::sweep(&gc.adult_start, &mut gc.runtime.bytes_allocated);
        }
    }

    unsafe fn mark_heap(
        head: &Cell<Option<NonNull<GcBox<dyn Trace>>>>,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let _timer = Profiler::global().start_event("Gc Marking", "gc");
        // Walk the list, tracing and marking the nodes
        let mut finalize = Vec::new();
        let mut ephemeron_queue = Vec::new();
        let mut mark_head = head;
        while let Some(node) = mark_head.get() {
            let node_ref = unsafe { node.as_ref() };
            if node_ref.header.is_ephemeron() {
                ephemeron_queue.push(node);
            } else if node_ref.header.roots() > 0 {
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
            ephemeron_queue = unsafe { Self::mark_ephemerons(ephemeron_queue) };
        }

        // Any left over nodes in the ephemeron queue at this point are
        // unreachable and need to be notified/finalized.
        finalize.extend(ephemeron_queue);

        finalize
    }

    // Tracing Ephemerons/Weak is always requires tracing the inner nodes in case it ends up marking unmarked node
    //
    // Time complexity should be something like O(nd) where d is the longest chain of epehemerons
    unsafe fn mark_ephemerons(
        initial_queue: Vec<NonNull<GcBox<dyn Trace>>>,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let mut ephemeron_queue = initial_queue;
        loop {
            // iterate through ephemeron queue, sorting nodes by whether they
            // are reachable or unreachable<?>
            let (reachable, other): (Vec<_>, Vec<_>) =
                ephemeron_queue.into_iter().partition(|node| {
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

    unsafe fn finalize(finalize_vec: Vec<NonNull<GcBox<dyn Trace>>>) {
        let _timer = Profiler::global().start_event("Gc Finalization", "gc");
        for node in finalize_vec {
            // We double check that the unreachable nodes are actually unreachable
            // prior to finalization as they could have been marked by a different
            // trace after initially being added to the queue
            let node = unsafe { node.as_ref() };
            if !node.header.is_marked() {
                Trace::run_finalizer(&node.value);
            }
        }
    }

    unsafe fn sweep(
        heap_start: &Cell<Option<NonNull<GcBox<dyn Trace>>>>,
        total_allocated: &mut usize,
    ) {
        let _timer = Profiler::global().start_event("Gc Sweeping", "gc");
        let _guard = DropGuard::new();

        let mut sweep_head = heap_start;
        while let Some(node) = sweep_head.get() {
            let node_ref = unsafe { node.as_ref() };
            if node_ref.is_marked() {
                node_ref.header.unmark();
                sweep_head = &node_ref.header.next;
            } else if node_ref.header.is_ephemeron() && node_ref.header.roots() > 0 {
                // Keep the ephemeron box's alive if rooted, but note that it's pointer is no longer safe
                Trace::run_finalizer(&node_ref.value);
                sweep_head = &node_ref.header.next;
            } else {
                // Drops occur here
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
            // Drops every node
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
