//! Garbage collector for the Boa JavaScript engine.

use std::cell::{Cell as StdCell, RefCell as StdRefCell};
use std::mem;
use std::ptr::NonNull;

pub use boa_gc_macros::{Trace, Finalize};

/// `gc_derive` is a general derive prelude import 
pub mod derive_prelude {
    pub use boa_gc_macros::{Trace, Finalize};
    pub use crate::GcPointer;
}

mod gc_box;
mod internals;
pub mod trace;
pub mod pointers;

pub(crate) use gc_box::GcBox;
pub use internals::{GcCell, GcCellRef};
use pointers::Gc;
pub use crate::trace::{Finalize, Trace};

pub type GcPointer = NonNull<GcBox<dyn Trace>>;

thread_local!(pub static GC_DROPPING: StdCell<bool> = StdCell::new(false));
thread_local!(static BOA_GC: StdRefCell<BoaGc> = StdRefCell::new( BoaGc {
    config: GcConfig::default(),
    runtime: GcRuntimeData::default(),
    heap_start: StdCell::new(None),
    stack: StdCell::new(Vec::new()),
}));

struct GcConfig {
    threshold: usize,
    growth_ratio: f64,
    stack_base_capacity: usize,
    stack_soft_cap: usize,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            threshold: 100,
            growth_ratio: 0.7,
            stack_base_capacity: 255,
            stack_soft_cap: 255,
        }
    }
}

struct GcRuntimeData {
    collections: usize,
    heap_bytes_allocated: usize,
    stack_allocations: usize,
}

impl Default for GcRuntimeData {
    fn default() -> Self {
        Self {
            collections: 0,
            heap_bytes_allocated: 0,
            stack_allocations: 0,
        }
    }
}

struct BoaGc {
    config: GcConfig,
    runtime: GcRuntimeData,
    heap_start: StdCell<Option<GcPointer>>,
    stack: StdCell<Vec<GcPointer>>,
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
pub struct GcAlloc;

impl GcAlloc {
    pub fn new<T: Trace>(value: T) -> Gc<T> {
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            unsafe {
                Self::manage_state::<T>(&mut *gc);
            }

            let stack_element = Box::into_raw(Box::from(GcBox::new(value)));
            unsafe {
                let mut stack = gc.stack.take();
                stack.push(NonNull::new_unchecked(stack_element));
                gc.stack.set(stack);
                gc.runtime.stack_allocations += 1;

                Gc::new(NonNull::new_unchecked(stack_element))
            }
        })
    }

    pub fn new_cell<T: Trace>(value: T) -> Gc<GcCell<T>> {
        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            // Manage state preps the internal state for allocation and
            // triggers a collection if the state dictates it.
            unsafe {
                Self::manage_state::<T>(&mut *gc);
            }

            let cell = GcCell::new(value);
            let stack_element = Box::into_raw(Box::from(GcBox::new(cell)));
            unsafe {
                let mut stack = gc.stack.take();
                stack.push(NonNull::new_unchecked(stack_element));
                gc.stack.set(stack);
                gc.runtime.stack_allocations += 1;

                Gc::new(NonNull::new_unchecked(stack_element))
            }
        })
    }

    pub fn new_weak_pair<K: Trace, V: Trace>(key: K, value: V) {
        todo!()
    }

    pub fn new_weak_cell<T: Trace>(value: T) {
        todo!()
    }

    pub(crate) unsafe fn promote_allocs<T: Trace>(
        promotions: Vec<NonNull<GcBox<dyn Trace>>>,
        gc: &mut BoaGc,
    ) {
        for node in promotions {
            (*node.as_ptr()).promote(gc.heap_start.take());
            gc.heap_start.set(Some(node));
            gc.runtime.heap_bytes_allocated += mem::size_of::<GcBox<T>>();
        }
    }

    unsafe fn manage_state<T: Trace>(gc: &mut BoaGc) {
        if gc.runtime.heap_bytes_allocated > gc.config.threshold {
            Collector::run_full_collection::<T>(gc);

            if gc.runtime.heap_bytes_allocated as f64
                > gc.config.threshold as f64 * gc.config.growth_ratio
            {
                gc.config.threshold =
                    (gc.runtime.heap_bytes_allocated as f64 / gc.config.growth_ratio) as usize
            }
        } else {
            if gc.runtime.stack_allocations > gc.config.stack_soft_cap {
                Collector::run_stack_collection::<T>(gc);

                // If we are constrained on the top of the stack,
                // increase the size of capacity, so a garbage collection
                // isn't triggered on every allocation
                if gc.runtime.stack_allocations > gc.config.stack_soft_cap {
                    gc.config.stack_soft_cap += 5
                }

                // If the soft cap was increased but the allocation has lowered below
                // the initial base, then reset to the original base
                if gc.runtime.stack_allocations < gc.config.stack_base_capacity
                    && gc.config.stack_base_capacity != gc.config.stack_soft_cap
                {
                    gc.config.stack_soft_cap = gc.config.stack_base_capacity
                }
            }
        }
    }
}

pub struct Collector;

impl Collector {
    pub(crate) unsafe fn run_stack_collection<T: Trace>(gc: &mut BoaGc) {
        gc.runtime.collections += 1;
        let stack = gc.stack.take();
        let unreachable_nodes = Self::mark_stack(&stack);
        Self::finalize(unreachable_nodes);
        let _finalized = Self::mark_stack(&stack);
        let promotions = Self::stack_sweep(gc, stack);
        GcAlloc::promote_allocs::<T>(promotions, gc);
    }

    pub(crate) unsafe fn run_full_collection<T: Trace>(gc: &mut BoaGc) {
        gc.runtime.collections += 1;
        let old_stack = gc.stack.take();
        let mut unreachable = Self::mark_heap(&gc.heap_start);
        let stack_unreachable = Self::mark_stack(&old_stack);
        unreachable.extend(stack_unreachable);
        Self::finalize(unreachable);
        let _heap_finalized = Self::mark_heap(&gc.heap_start);
        let _sweep_finalized = Self::mark_stack(&old_stack);
        Self::heap_sweep(gc);
        let promotions = Self::stack_sweep(gc, old_stack);
        GcAlloc::promote_allocs::<T>(promotions, gc);
    }

    pub(crate) unsafe fn mark_heap(
        head: &StdCell<Option<NonNull<GcBox<dyn Trace>>>>,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        // Walk the list, tracing and marking the nodes
        let mut finalize = Vec::new();
        let mut ephemeron_queue = Vec::new();
        let mut mark_head = head;
        while let Some(node) = mark_head.get() {
            if (*node.as_ptr()).header.is_ephemeron() {
                ephemeron_queue.push(node);
            } else {
                if (*node.as_ptr()).header.roots() > 0 {
                    (*node.as_ptr()).trace_inner();
                } else {
                    finalize.push(node)
                }
            }
            mark_head = &(*node.as_ptr()).header.next;
        }

        // Ephemeron Evaluation
        if !ephemeron_queue.is_empty() {
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
                    // iterate through reachable nodes and trace their values,
                    // enqueuing any ephemeron that is found during the trace
                    for node in reachable_nodes {
                        (*node.as_ptr()).weak_trace_inner(&mut ephemeron_queue)
                    }
                } else {
                    break;
                }
            }
        }

        // Any left over nodes in the ephemeron queue at this point are
        // unreachable and need to be notified/finalized.
        finalize.extend(ephemeron_queue);

        finalize
    }

    pub(crate) unsafe fn mark_stack(
        stack: &Vec<NonNull<GcBox<dyn Trace>>>,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let mut finalize = Vec::new();

        for node in stack {
            if (*node.as_ptr()).header.roots() > 0 {
                (*node.as_ptr()).header.mark()
            } else {
                finalize.push(*node)
            }
        }

        finalize
    }

    unsafe fn finalize(finalize_vec: Vec<NonNull<GcBox<dyn Trace>>>) {
        for node in finalize_vec {
            // We double check that the unreachable nodes are actually unreachable
            // prior to finalization as they could have been marked by a different
            // trace after initially being added to the queue
            if !(*node.as_ptr()).header.is_marked() {
                Trace::run_finalizer(&(*node.as_ptr()).value)
            }
        }
    }

    unsafe fn stack_sweep(
        gc: &mut BoaGc,
        old_stack: Vec<NonNull<GcBox<dyn Trace>>>,
    ) -> Vec<NonNull<GcBox<dyn Trace>>> {
        let _guard = DropGuard::new();

        let mut new_stack = Vec::new();
        let mut promotions = Vec::new();

        for node in old_stack {
            if (*node.as_ptr()).header.is_marked() {
                (*node.as_ptr()).header.unmark();
                (*node.as_ptr()).header.inc_age();
                if (*node.as_ptr()).header.age() > 10 {
                    promotions.push(node);
                } else {
                    new_stack.push(node)
                }
            } else {
                gc.runtime.stack_allocations -= 1;
            }
        }

        gc.stack.set(new_stack);
        promotions
    }

    unsafe fn heap_sweep(gc: &mut BoaGc) {
        let _guard = DropGuard::new();

        let mut sweep_head = &gc.heap_start;
        while let Some(node) = sweep_head.get() {
            if (*node.as_ptr()).header.is_marked() {
                (*node.as_ptr()).header.unmark();
                sweep_head = &(*node.as_ptr()).header.next;
            } else {
                let unmarked_node = Box::from_raw(node.as_ptr());
                gc.runtime.heap_bytes_allocated -= mem::size_of_val::<GcBox<_>>(&*unmarked_node);
                sweep_head.set(unmarked_node.header.next.take());
            }
        }
    }
}
