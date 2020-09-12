//! This module represents the main way to interact with the garbacge collector.

// This is because `rust-gc` unsafe_empty_trace has a `unsafe_`
// when it should be `empty_trace`.
#![allow(clippy::unsafe_removed_from_name)]

pub use crate::object::GcObject;
pub use ::gc::{
    custom_trace, force_collect, unsafe_empty_trace as empty_trace, Finalize, GcCellRef as Ref,
    GcCellRefMut as RefMut, Trace,
};
