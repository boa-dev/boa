//! Garbage collector for the Boa JavaScript engine.

pub use gc::{
    custom_trace, finalizer_safe, force_collect, unsafe_empty_trace, Finalize, Gc, GcCell as Cell,
    GcCellRef as Ref, GcCellRefMut as RefMut, Trace,
};
