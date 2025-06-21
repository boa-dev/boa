use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use boa_ast::Position;

mod builder;

use boa_gc::{Finalize, Trace};
use boa_string::JsString;
pub(crate) use builder::SourceMapBuilder;

// TODO: The line number increments slower than column,
//       maybe we can take advantage of this, for memory optimization?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) start_pc: u32,
    pub(crate) position: Option<Position>,
}

impl Entry {
    pub(crate) const fn position(&self) -> Option<Position> {
        self.position
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Inner {
    entries: Box<[Entry]>,
    file_path: Option<PathBuf>,
    function_name: JsString,
}

/// Bytecode to source code mapping.
#[derive(Debug, Default, Clone, PartialEq, Eq, Finalize, Trace)]
// SAFETY: Nothing in Inner needs tracing, so this is safe.
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct SourceMap {
    inner: Rc<Inner>,
}

impl SourceMap {
    pub(crate) fn new(
        file_path: Option<PathBuf>,
        entries: Box<[Entry]>,
        function_name: JsString,
    ) -> Self {
        Self {
            inner: Rc::new(Inner {
                entries,
                file_path,
                function_name,
            }),
        }
    }

    pub(crate) fn entries(&self) -> &[Entry] {
        &self.inner.entries
    }

    pub(crate) fn find(&self, pc: u32) -> Option<Position> {
        for entry in self.entries().iter().rev() {
            if entry.start_pc + 1 < pc {
                return entry.position();
            }
        }

        None
    }

    pub(crate) fn file_path(&self) -> Option<&Path> {
        self.inner.file_path.as_deref()
    }

    pub(crate) fn function_name(&self) -> &JsString {
        &self.inner.function_name
    }
}
