use boa_macros::{Finalize, Trace};

use super::SharedShape;

/// This is a wrapper around [`SharedShape`] that ensures it's root shape.
///
/// Represent the root shape that [`SharedShape`] transitions start from.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RootShape {
    shape: SharedShape,
}

impl Default for RootShape {
    #[inline]
    fn default() -> Self {
        Self::new_in(&unsafe { boa_gc::MutationContext::global() })
    }
}

impl RootShape {
    /// Create a new root shape using the given context.
    #[inline]
    pub(crate) fn new_in(mc: &boa_gc::MutationContext<'static, '_>) -> Self {
        Self {
            shape: SharedShape::root_in(mc),
        }
    }
    /// Gets the inner [`SharedShape`].
    #[must_use]
    pub const fn shape(&self) -> &SharedShape {
        &self.shape
    }
}
