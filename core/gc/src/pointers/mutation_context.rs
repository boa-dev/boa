use std::marker::PhantomData;

/// Context required to safely allocate or mutate the Gc heap
#[derive(Copy, Clone, Debug)]
pub struct MutationContext<'gc, 'a> {
    _marker: PhantomData<&'a &'gc ()>,
}

impl MutationContext<'_, '_> {
    /// Creates a temporary dummy context
    ///
    /// # Safety
    /// Bypasses lifetime branding, use only as a bridge during Gc migration.
    #[must_use]
    pub unsafe fn dummy() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
