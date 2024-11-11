use boa_ast::{LinearSpan, SourceText};
use boa_gc::{Finalize, Gc, Trace};

#[derive(Trace, Finalize)]
pub(crate) struct SourceTextInner {
    #[unsafe_ignore_trace]
    source_text: SourceText,
}
impl SourceTextInner {
    #[must_use]
    pub(crate) fn new(code: &mut boa_ast::Script) -> Option<Gc<Self>> {
        code.take_source()
            .map(|source_text| Gc::new(Self { source_text }))
    }
}

/// Contains pointer to source code and span of the object.
#[derive(Trace, Finalize, Clone)]
pub struct SpannedSourceText {
    gc: Gc<SourceTextInner>,
    #[unsafe_ignore_trace]
    span: LinearSpan,
}
impl SpannedSourceText {
    pub(crate) fn new(gc: Gc<SourceTextInner>, span: LinearSpan) -> Self {
        Self { gc, span }
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
        self.gc.source_text.get_code_points_from_span(self.span)
    }
}

impl std::fmt::Debug for SpannedSourceText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextSpanned").finish()
    }
}
impl std::fmt::Debug for SourceTextInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextInner").finish()
    }
}
