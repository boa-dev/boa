//! Garbage collector for the Boa JavaScript engine.

pub use gc::{
    custom_trace, finalizer_safe, force_collect, unsafe_empty_trace, Finalize, Gc, GcCell as Cell,
    GcCellRef as Ref, GcCellRefMut as RefMut, Trace,
};

use std::ptr::NonNull;

mod heap_box;

use heap_box::HeapBox;

struct GcConfig {
    threshold: usize,
    growth_ratio: f64,

}

struct GcRuntimeData {
    byte_allocated: usize,
}

struct BoaGc {
    config: GcConfig,
    runtime: GcRuntimeData, 
    heap_start: Option<NonNull<HeapBox<dyn Trace>>>,
}


/// The GcAllocater allocates a garbage collected value to heap.
pub struct GcAllocater<T: Trace>;

impl BoaAllocater<T: Trace> {
    pub fn new_gc(value: T) -> Gc<T> {

    }
}


pub struct Collector;


impl Collector {
    pub(crate) fn run_collection(st: &mut GcRuntimeData) {

    }
}

