use boa_gc::{empty_trace, Finalize, Trace};
use boa_interner::Sym;

/// Private runtime environment.
#[derive(Clone, Debug, Finalize)]
pub(crate) struct PrivateEnvironment {
    /// The unique identifier of the private names.
    id: usize,

    /// The `[[Description]]` internal slot of the private names.
    descriptions: Vec<Sym>,
}

// Safety: PrivateEnvironment does not contain any objects that need to be traced.
unsafe impl Trace for PrivateEnvironment {
    empty_trace!();
}

impl PrivateEnvironment {
    /// Creates a new `PrivateEnvironment`.
    pub(crate) fn new(id: usize, descriptions: Vec<Sym>) -> Self {
        Self { id, descriptions }
    }

    /// Gets the id of this private environment.
    pub(crate) const fn id(&self) -> usize {
        self.id
    }

    /// Gets the descriptions of this private environment.
    pub(crate) fn descriptions(&self) -> &[Sym] {
        &self.descriptions
    }
}
