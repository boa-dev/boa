//! Garbage collector for the Boa JavaScript engine.

#![allow(
    clippy::let_unit_value,
    clippy::should_implement_trait,
    clippy::match_like_matches_macro,
    clippy::new_ret_no_self,
    clippy::needless_bool,
    // Putting the below on the allow list for now, but these should eventually be addressed
    clippy::missing_safety_doc,
    clippy::explicit_auto_deref,
    clippy::borrow_deref_ref,
)]

extern crate self as boa_gc;

use boa_profiler::Profiler;
use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr::NonNull;

pub mod trace;

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
    adult_threshold: usize,
    growth_ratio: f64,
}

// Setting the defaults to an arbitrary value currently.
//
// TODO: Add a configure later
impl Default for GcConfig {
    fn default() -> Self {
        Self {
            adult_threshold: 1024,
            growth_ratio: 0.8,
        }
    }
}

#[derive(Default)]
struct GcRuntimeData {
    collections: usize,
    total_bytes_allocated: usize,
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

pub fn finalizer_safe() -> bool {
    GC_DROPPING.with(|dropping| !dropping.get())
}

/// The Allocator handles allocation of garbage collected values.
///
/// The allocator can trigger a garbage collection
struct Allocator;

impl Allocator {
    fn new<T: Trace>(value: GcBox<T>) -> NonNull<GcBox<T>> {
        let _timer = Profiler::global().start_event("New Pointer", "BoaAlloc");
        let element_size = mem::size_of_val::<GcBox<T>>(&value);
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            Self::manage_state(&mut gc);
            value.header.next.set(gc.adult_start.take());
            let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::from(value))) };

            gc.adult_start.set(Some(ptr));
            gc.runtime.total_bytes_allocated += element_size;

            ptr
        })
    }

    fn manage_state(gc: &mut BoaGc) {
        if gc.runtime.total_bytes_allocated > gc.config.adult_threshold {
            Collector::run_full_collection(gc);

            if gc.runtime.total_bytes_allocated as f64
                > gc.config.adult_threshold as f64 * gc.config.growth_ratio
            {
                gc.config.adult_threshold =
                    (gc.runtime.total_bytes_allocated as f64 / gc.config.growth_ratio) as usize
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
pub struct Collector;

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
            Self::sweep(&gc.adult_start, &mut gc.runtime.total_bytes_allocated);
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
            if (*node.as_ptr()).header.is_ephemeron() {
                ephemeron_queue.push(node);
            } else if (*node.as_ptr()).header.roots() > 0 {
                (*node.as_ptr()).trace_inner();
            } else {
                finalize.push(node)
            }
            mark_head = &(*node.as_ptr()).header.next;
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
    unsafe fn mark_ephemerons(
        initial_queue: Vec<NonNull<GcBox<dyn Trace>>>,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let mut ephemeron_queue = initial_queue;
        loop {
            // iterate through ephemeron queue, sorting nodes by whether they
            // are reachable or unreachable<?>
            let (reachable, other): (Vec<_>, Vec<_>) =
                ephemeron_queue.into_iter().partition(|node| {
                    if node.as_ref().value.is_marked_ephemeron() {
                        node.as_ref().header.mark();
                        true
                    } else if node.as_ref().header.roots() > 0 {
                        true
                    } else {
                        false
                    }
                });
            // Replace the old queue with the unreachable<?>
            ephemeron_queue = other;

            // If reachable nodes is not empty, trace values. If it is empty,
            // break from the loop
            if !reachable.is_empty() {
                EPHEMERON_QUEUE.with(|state| state.set(Some(Vec::new())));
                // iterate through reachable nodes and trace their values,
                // enqueuing any ephemeron that is found during the trace
                for node in reachable {
                    // TODO: deal with fetch ephemeron_queue
                    (*node.as_ptr()).weak_trace_inner()
                }

                EPHEMERON_QUEUE.with(|st| {
                    if let Some(found_nodes) = st.take() {
                        ephemeron_queue.extend(found_nodes)
                    }
                })
            } else {
                break;
            }
        }
        ephemeron_queue
    }

    unsafe fn finalize(finalize_vec: Vec<NonNull<GcBox<dyn Trace>>>) {
        let _timer = Profiler::global().start_event("Gc Finalization", "gc");
        for node in finalize_vec {
            // We double check that the unreachable nodes are actually unreachable
            // prior to finalization as they could have been marked by a different
            // trace after initially being added to the queue
            if !(*node.as_ptr()).header.is_marked() {
                Trace::run_finalizer(&(*node.as_ptr()).value)
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
            if (*node.as_ptr()).is_marked() {
                (*node.as_ptr()).header.unmark();
                sweep_head = &(*node.as_ptr()).header.next;
            } else if (*node.as_ptr()).header.is_ephemeron() && (*node.as_ptr()).header.roots() > 0
            {
                // Keep the ephemeron box's alive if rooted, but note that it's pointer is no longer safe
                Trace::run_finalizer(&(*node.as_ptr()).value);
                sweep_head = &(*node.as_ptr()).header.next;
            } else {
                // Drops occur here
                let unmarked_node = Box::from_raw(node.as_ptr());
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

// A utility function that forces runs through Collector method based off the state.
//
// Note:
//  - This method is meant solely for testing purposes only
//  - `force_collect` will not extend threshold
pub fn force_collect() {
    BOA_GC.with(|current| {
        let mut gc = current.borrow_mut();

        if gc.runtime.total_bytes_allocated > 0 {
            Collector::run_full_collection(&mut *gc)
        }
    })
}

#[cfg(test)]
mod test;
