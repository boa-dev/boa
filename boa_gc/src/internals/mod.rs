pub(crate) mod borrow_flag;
pub mod cell;
pub mod cell_ref;
pub mod ephemeron;

pub use cell::GcCell;
pub use cell_ref::{GcCellRef, GcCellRefMut};
pub use ephemeron::Ephemeron;
