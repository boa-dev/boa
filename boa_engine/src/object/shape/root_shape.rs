use boa_macros::{Finalize, Trace};

use super::SharedShape;

/// Represent the root shape that [`SharedShape`] transitions start from.
///
/// This is a wrapper around [`SharedShape`] that ensures that the shape the root shape.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RootShape {
    shape: SharedShape,
}

impl Default for RootShape {
    fn default() -> Self {
        Self {
            shape: SharedShape::root(),
        }
    }
}

impl RootShape {
    /// Gets the inner [`SharedShape`].
    pub(crate) const fn shape(&self) -> &SharedShape {
        &self.shape
    }
}
