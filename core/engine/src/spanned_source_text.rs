use boa_ast::LinearSpan;
use boa_gc::{Finalize, Gc, Trace};

#[derive(Trace, Finalize)]
struct Inner {
    #[unsafe_ignore_trace]
    source_text: boa_ast::SourceText,
}
impl Inner {
    fn new(source_text: boa_ast::SourceText) -> Self {
        Self { source_text }
    }
}

#[derive(Trace, Finalize, Clone)]
pub(crate) struct SourceText {
    source_text: Gc<Inner>,
}

impl SourceText {
    #[must_use]
    pub(crate) fn new(source_text: boa_ast::SourceText) -> Self {
        Self {
            source_text: Gc::new(Inner::new(source_text)),
        }
    }

    fn inner(&self) -> &boa_ast::SourceText {
        &self.source_text.source_text
    }
}

/// Contains pointer to source code and span of the object.
#[derive(Trace, Finalize, Clone)]
pub struct SpannedSourceText {
    source_text: SourceText,
    #[unsafe_ignore_trace]
    span: LinearSpan,
}
impl SpannedSourceText {
    pub(crate) fn new(source_text: SourceText, span: LinearSpan) -> Self {
        Self { source_text, span }
    }

    /// Test if the span is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }

    /// Gets inner code points.
    #[must_use]
    pub fn to_code_points(&self) -> &[u16] {
        self.source_text
            .inner()
            .get_code_points_from_span(self.span)
    }
}

impl std::fmt::Debug for SpannedSourceText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextSpanned").finish()
    }
}
impl std::fmt::Debug for SourceText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextInner").finish()
    }
}
