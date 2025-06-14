use std::rc::Rc;

use boa_ast::Position;

mod builder;

use boa_gc::{Finalize, Trace};
pub(crate) use builder::SourceMapBuilder;

// TODO: The line number increments slower than column,
//       maybe we can take advantage of this, for memory optimization?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) start_pc: u32,
    pub(crate) position: Option<Position>,
}

impl Entry {
    pub(crate) fn position(&self) -> Option<Position> {
        self.position
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Inner {
    entries: Box<[Entry]>,
}

/// Bytecode to source code mapping.
#[derive(Debug, Default, Clone, PartialEq, Eq, Finalize, Trace)]
// SAFETY: Nothing in Inner needs tracing, so this is safe.
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct SourceMap {
    inner: Rc<Inner>,
}

impl SourceMap {
    pub(crate) fn new(entries: Box<[Entry]>) -> Self {
        Self {
            inner: Rc::new(Inner { entries }),
        }
    }

    pub(crate) fn entries(&self) -> &[Entry] {
        &self.inner.entries
    }

    pub(crate) fn find(&self, pc: u32) -> Option<Position> {
        // TODO: Optimized the search, maybe using binary-search?
        self.entries()
            .iter()
            .find(|entry| entry.start_pc >= pc)
            .and_then(Entry::position)
    }
}
