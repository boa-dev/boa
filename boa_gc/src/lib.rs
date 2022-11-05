//! Garbage collector for the Boa JavaScript engine.

#![allow(
    clippy::let_unit_value,
    clippy::should_implement_trait,
    clippy::match_like_matches_macro,
    clippy::new_ret_no_self,
    // Putting the below on the allow list for now, but these should eventually be addressed
    clippy::missing_safety_doc,
    clippy::explicit_auto_deref,
    clippy::borrow_deref_ref,
)]

use boa_profiler::Profiler;
use std::cell::{Cell as StdCell, RefCell as StdRefCell};
use std::mem;
use std::ptr::NonNull;

mod gc_box;
mod internals;
mod pointers;
pub mod trace;

pub use boa_gc_macros::{Finalize, Trace};

pub use crate::trace::{Finalize, Trace};
pub(crate) use gc_box::GcBox;
pub use internals::{Ephemeron, GcCell as Cell, GcCellRef as Ref, GcCellRefMut as RefMut};
pub use pointers::{Gc, WeakGc, WeakPair};

pub type GcPointer = NonNull<GcBox<dyn Trace>>;

// TODO: Determine if thread local variables are the correct approach vs an initialized structure
thread_local!(pub static EPHEMERON_QUEUE: StdCell<Option<Vec<GcPointer>>> = StdCell::new(None));
thread_local!(pub static GC_DROPPING: StdCell<bool> = StdCell::new(false));
thread_local!(static BOA_GC: StdRefCell<BoaGc> = StdRefCell::new( BoaGc {
    config: GcConfig::default(),
    runtime: GcRuntimeData::default(),
    adult_start: StdCell::new(None),
    youth_start: StdCell::new(None),
}));

struct GcConfig {
    youth_threshold: usize,
    youth_threshold_base: usize,
    adult_threshold: usize,
    growth_ratio: f64,
    youth_promo_age: u8,
}

// Setting the defaults to an arbitrary value currently.
//
// TODO: Add a configure later
impl Default for GcConfig {
    fn default() -> Self {
        Self {
            youth_threshold: 4096,
            youth_threshold_base: 4096,
            adult_threshold: 16384,
            growth_ratio: 0.7,
            youth_promo_age: 3,
        }
    }
}

#[derive(Default)]
struct GcRuntimeData {
    collections: usize,
    total_bytes_allocated: usize,
    youth_bytes: usize,
    adult_bytes: usize,
}

struct BoaGc {
    config: GcConfig,
    runtime: GcRuntimeData,
    adult_start: StdCell<Option<GcPointer>>,
    youth_start: StdCell<Option<GcPointer>>,
}

impl Drop for BoaGc {
    fn drop(&mut self) {
        unsafe {
            Collector::dump(self);
        }
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

/// The GcAllocater handles initialization and allocation of garbage collected values.
///
/// The allocator can trigger a garbage collection
pub struct BoaAlloc;

impl BoaAlloc {
    pub fn new<T: Trace>(value: T) -> Gc<T> {
        let _timer = Profiler::global().start_event("New Pointer", "BoaAlloc");
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            unsafe {
                Self::manage_state(&mut gc);
            }

            let gc_box = GcBox::new(value);

            let element_size = mem::size_of_val::<GcBox<T>>(&gc_box);
            let element_pointer = Box::into_raw(Box::from(gc_box));

            unsafe {
                let old_start = gc.youth_start.take();
                (*element_pointer).set_header_pointer(old_start);
                gc.youth_start
                    .set(Some(NonNull::new_unchecked(element_pointer)));

                gc.runtime.total_bytes_allocated += element_size;
                gc.runtime.youth_bytes += element_size;

                Gc::new(NonNull::new_unchecked(element_pointer))
            }
        })
    }

    pub fn new_cell<T: Trace>(value: T) -> Gc<Cell<T>> {
        let _timer = Profiler::global().start_event("New Cell", "BoaAlloc");
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            // Manage state preps the internal state for allocation and
            // triggers a collection if the state dictates it.
            unsafe {
                Self::manage_state(&mut gc);
            }

            let gc_box = GcBox::new(Cell::new(value));
            let element_size = mem::size_of_val::<GcBox<Cell<T>>>(&gc_box);
            let element_pointer = Box::into_raw(Box::from(gc_box));

            unsafe {
                let old_start = gc.youth_start.take();
                (*element_pointer).set_header_pointer(old_start);
                gc.youth_start
                    .set(Some(NonNull::new_unchecked(element_pointer)));

                gc.runtime.youth_bytes += element_size;
                gc.runtime.total_bytes_allocated += element_size;

                Gc::new(NonNull::new_unchecked(element_pointer))
            }
        })
    }

    pub fn new_weak_pair<K: Trace, V: Trace>(key: NonNull<GcBox<K>>, value: V) -> WeakPair<K, V> {
        let _timer = Profiler::global().start_event("New Weak Pair", "BoaAlloc");
        BOA_GC.with(|internals| {
            let mut gc = internals.borrow_mut();

            unsafe {
                Self::manage_state(&mut gc);
                let ephem = Ephemeron::new_pair(key, value);
                let gc_box = GcBox::new_weak(ephem);

                let element_size = mem::size_of_val::<GcBox<_>>(&gc_box);
                let element_pointer = Box::into_raw(Box::from(gc_box));

                let old_start = gc.youth_start.take();
                (*element_pointer).set_header_pointer(old_start);
                gc.youth_start
                    .set(Some(NonNull::new_unchecked(element_pointer)));

                gc.runtime.total_bytes_allocated += element_size;

                WeakPair::new(NonNull::new_unchecked(element_pointer))
            }
        })
    }

    pub fn new_weak_ref<T: Trace>(value: NonNull<GcBox<T>>) -> WeakGc<T> {
        let _timer = Profiler::global().start_event("New Weak Pointer", "BoaAlloc");
        BOA_GC.with(|state| {
            let mut gc = state.borrow_mut();

            unsafe {
                Self::manage_state(&mut gc);

                let ephemeron = Ephemeron::new(value);
                let gc_box = GcBox::new_weak(ephemeron);

                let element_size = mem::size_of_val::<GcBox<_>>(&gc_box);
                let element_pointer = Box::into_raw(Box::from(gc_box));

                let old_start = gc.youth_start.take();
                (*element_pointer).set_header_pointer(old_start);
                gc.youth_start
                    .set(Some(NonNull::new_unchecked(element_pointer)));

                gc.runtime.total_bytes_allocated += element_size;

                WeakGc::new(NonNull::new_unchecked(element_pointer))
            }
        })
    }

    // Possibility here for `new_weak` that takes any value and creates a new WeakGc

    pub(crate) unsafe fn promote_to_medium(
        promotions: Vec<NonNull<GcBox<dyn Trace>>>,
        gc: &mut BoaGc,
    ) {
        let _timer = Profiler::global().start_event("Gc Promoting", "gc");
        for node in promotions {
            (*node.as_ptr()).set_header_pointer(gc.adult_start.take());
            let allocation_bytes = mem::size_of_val::<GcBox<_>>(&(*node.as_ptr()));
            gc.runtime.youth_bytes -= allocation_bytes;
            gc.runtime.adult_bytes += allocation_bytes;
            gc.adult_start.set(Some(node));
        }
    }

    unsafe fn manage_state(gc: &mut BoaGc) {
        if gc.runtime.adult_bytes > gc.config.adult_threshold {
            Collector::run_full_collection(gc);

            if gc.runtime.adult_bytes as f64
                > gc.config.adult_threshold as f64 * gc.config.growth_ratio
            {
                gc.config.adult_threshold =
                    (gc.runtime.adult_bytes as f64 / gc.config.growth_ratio) as usize
            }
        } else if gc.runtime.youth_bytes > gc.config.youth_threshold {
            Collector::run_youth_collection(gc);

            // If we are constrained on the top of the stack,
            // increase the size of capacity, so a garbage collection
            // isn't triggered on every allocation
            if gc.runtime.youth_bytes > gc.config.youth_threshold {
                gc.config.youth_threshold =
                    (gc.runtime.youth_bytes as f64 / gc.config.growth_ratio) as usize
            }

            // The young object threshold should only be raised in cases of high laod. It
            // should retract back to base when the load lessens
            if gc.runtime.youth_bytes < gc.config.youth_threshold_base
                && gc.config.youth_threshold != gc.config.youth_threshold_base
            {
                gc.config.youth_threshold = gc.config.youth_threshold_base
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
// A better appraoch in a more concurrent structure may be to reorder.
//
// Mark -> Sweep -> Finalize
pub struct Collector;

impl Collector {
    pub(crate) unsafe fn run_youth_collection(gc: &mut BoaGc) {
        let _timer = Profiler::global().start_event("Gc Youth Collection", "gc");
        gc.runtime.collections += 1;
        let unreachable_nodes = Self::mark_heap(&gc.youth_start);

        if !unreachable_nodes.is_empty() {
            Self::finalize(unreachable_nodes);
        }
        // The returned unreachable vector must be filled with nodes that are for certain dead (these will be removed during the sweep)
        let _finalized_unreachable_nodes = Self::mark_heap(&gc.youth_start);
        let promotion_candidates = Self::sweep_with_promotions(
            &gc.youth_start,
            &mut gc.runtime.youth_bytes,
            &mut gc.runtime.total_bytes_allocated,
            &gc.config.youth_promo_age,
        );
        // Check if there are any candidates for promotion
        if !promotion_candidates.is_empty() {
            BoaAlloc::promote_to_medium(promotion_candidates, gc);
        }
    }

    pub(crate) unsafe fn run_full_collection(gc: &mut BoaGc) {
        let _timer = Profiler::global().start_event("Gc Full Collection", "gc");
        gc.runtime.collections += 1;
        let unreachable_adults = Self::mark_heap(&gc.adult_start);
        let unreachable_youths = Self::mark_heap(&gc.youth_start);

        // Check if any unreachable nodes were found and finalize
        if !unreachable_adults.is_empty() {
            Self::finalize(unreachable_adults);
        }
        if !unreachable_youths.is_empty() {
            Self::finalize(unreachable_youths);
        }

        let _final_unreachable_adults = Self::mark_heap(&gc.adult_start);
        let _final_unreachable_youths = Self::mark_heap(&gc.youth_start);

        // Sweep both without promoting any values
        Self::sweep(
            &gc.adult_start,
            &mut gc.runtime.adult_bytes,
            &mut gc.runtime.total_bytes_allocated,
        );
        Self::sweep(
            &gc.youth_start,
            &mut gc.runtime.youth_bytes,
            &mut gc.runtime.total_bytes_allocated,
        );
    }

    pub(crate) unsafe fn mark_heap(
        head: &StdCell<Option<NonNull<GcBox<dyn Trace>>>>,
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
            let mut reachable_nodes = Vec::new();
            let mut other_nodes = Vec::new();
            // iterate through ephemeron queue, sorting nodes by whether they
            // are reachable or unreachable<?>
            for node in ephemeron_queue {
                if (*node.as_ptr()).value.is_marked_ephemeron() {
                    (*node.as_ptr()).header.mark();
                    reachable_nodes.push(node);
                } else {
                    other_nodes.push(node);
                }
            }
            // Replace the old queue with the unreachable<?>
            ephemeron_queue = other_nodes;

            // If reachable nodes is not empty, trace values. If it is empty,
            // break from the loop
            if !reachable_nodes.is_empty() {
                EPHEMERON_QUEUE.with(|state| state.set(Some(Vec::new())));
                // iterate through reachable nodes and trace their values,
                // enqueuing any ephemeron that is found during the trace
                for node in reachable_nodes {
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

    unsafe fn sweep_with_promotions(
        heap_start: &StdCell<Option<NonNull<GcBox<dyn Trace>>>>,
        heap_bytes: &mut usize,
        total_bytes: &mut usize,
        promotion_age: &u8,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let _timer = Profiler::global().start_event("Gc Sweeping", "gc");
        let _guard = DropGuard::new();

        let mut promotions = Vec::new();
        let mut sweep_head = heap_start;
        while let Some(node) = sweep_head.get() {
            if (*node.as_ptr()).is_marked() {
                (*node.as_ptr()).header.unmark();
                (*node.as_ptr()).header.inc_age();
                if (*node.as_ptr()).header.age() >= *promotion_age {
                    sweep_head.set((*node.as_ptr()).header.next.take());
                    promotions.push(node)
                } else {
                    sweep_head = &(*node.as_ptr()).header.next;
                }
            } else {
                // Drops occur here
                let unmarked_node = Box::from_raw(node.as_ptr());
                let unallocated_bytes = mem::size_of_val::<GcBox<_>>(&*unmarked_node);
                *heap_bytes -= unallocated_bytes;
                *total_bytes -= unallocated_bytes;
                sweep_head.set(unmarked_node.header.next.take());
            }
        }

        promotions
    }

    unsafe fn sweep(
        heap_start: &StdCell<Option<NonNull<GcBox<dyn Trace>>>>,
        bytes_allocated: &mut usize,
        total_allocated: &mut usize,
    ) {
        let _timer = Profiler::global().start_event("Gc Sweeping", "gc");
        let _guard = DropGuard::new();

        let mut sweep_head = heap_start;
        while let Some(node) = sweep_head.get() {
            if (*node.as_ptr()).is_marked() {
                (*node.as_ptr()).header.unmark();
                (*node.as_ptr()).header.inc_age();
                sweep_head = &(*node.as_ptr()).header.next;
            } else {
                // Drops occur here
                let unmarked_node = Box::from_raw(node.as_ptr());
                let unallocated_bytes = mem::size_of_val::<GcBox<_>>(&*unmarked_node);
                *bytes_allocated -= unallocated_bytes;
                *total_allocated -= unallocated_bytes;
                sweep_head.set(unmarked_node.header.next.take());
            }
        }
    }

    // Clean up the heap when BoaGc is dropped
    unsafe fn dump(gc: &mut BoaGc) {
        Self::drop_heap(&gc.youth_start);
        Self::drop_heap(&gc.adult_start);
    }

    unsafe fn drop_heap(heap_start: &StdCell<Option<NonNull<GcBox<dyn Trace>>>>) {
        // Not initializing a dropguard since this should only be invoked when BOA_GC is being dropped.

        let sweep_head = heap_start;
        while let Some(node) = sweep_head.get() {
            // Drops every node
            let unmarked_node = Box::from_raw(node.as_ptr());
            sweep_head.set(unmarked_node.header.next.take());

            mem::forget(unmarked_node)
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

        unsafe {
            if gc.runtime.adult_bytes > 0 {
                Collector::run_full_collection(&mut *gc)
            } else {
                Collector::run_youth_collection(&mut *gc)
            }
        }
    })
}

pub struct GcTester;

impl GcTester {
    pub fn assert_collections(o: usize) {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert_eq!(gc.runtime.collections, o);
        })
    }

    pub fn assert_collection_floor(floor: usize) {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert!(gc.runtime.collections > floor);
        })
    }

    pub fn assert_youth_bytes_allocated() {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert!(gc.runtime.youth_bytes > 0);
        })
    }

    pub fn assert_empty_gc() {
        BOA_GC.with(|current| {
            let gc = current.borrow();

            assert!(gc.adult_start.get().is_none());
            assert!(gc.runtime.adult_bytes == 0);
            assert!(gc.youth_start.get().is_none());
            assert!(gc.runtime.youth_bytes == 0);
        })
    }

    pub fn assert_adult_bytes_allocated() {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert!(gc.runtime.adult_bytes > 0);
        })
    }
}
