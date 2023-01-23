mod ephemeron_box;
mod gc_box;

pub(crate) use self::ephemeron_box::{EphemeronBox, ErasedEphemeronBox};
pub use self::gc_box::GcBox;
