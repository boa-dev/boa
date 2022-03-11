//! Garbage collector for the Boa JavaScript engine.

pub use gc::{
    custom_trace, force_collect, unsafe_empty_trace, Finalize, Gc, GcCell as Cell,
    GcCellRef as Ref, GcCellRefMut as RefMut, Trace,
};
