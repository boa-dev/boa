//! An implementation of a `CompletionRecord` for Boa's VM.

#![allow(clippy::inline_always)]

use super::Registers;
use crate::{Context, JsError, JsResult, JsValue};
use boa_gc::{custom_trace, Finalize, Trace};
use std::ops::ControlFlow;

/// An implementation of the ECMAScript's `CompletionRecord` [specification] for
/// Boa's VM output Completion and Result.
///
/// [specification]: https://tc39.es/ecma262/#sec-completion-record-specification-type
#[derive(Debug, Clone, Finalize)]
pub(crate) enum CompletionRecord {
    Normal(JsValue),
    Return(JsValue),
    Throw(JsError),
}

// SAFETY: this matches all possible variants and traces
// their inner contents, which makes this safe.
unsafe impl Trace for CompletionRecord {
    custom_trace!(this, mark, {
        match this {
            Self::Normal(v) => mark(v),
            Self::Return(r) => mark(r),
            Self::Throw(th) => mark(th),
        }
    });
}

// ---- `CompletionRecord` methods ----
impl CompletionRecord {
    pub(crate) const fn is_throw_completion(&self) -> bool {
        matches!(self, Self::Throw(_))
    }

    /// This function will consume the current `CompletionRecord` and return a `JsResult<JsValue>`
    // NOTE: rustc bug around evaluating destructors that prevents this from being a const function.
    // Related issue(s):
    //   - https://github.com/rust-lang/rust-clippy/issues/4041
    //   - https://github.com/rust-lang/rust/issues/60964
    //   - https://github.com/rust-lang/rust/issues/73255
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn consume(self) -> JsResult<JsValue> {
        match self {
            Self::Throw(error) => Err(error),
            Self::Normal(value) | Self::Return(value) => Ok(value),
        }
    }
}

pub(crate) trait IntoCompletionRecord {
    fn into_completion_record(
        self,
        context: &mut Context,
        registers: &mut Registers,
    ) -> ControlFlow<CompletionRecord>;
}

impl IntoCompletionRecord for () {
    #[inline(always)]
    fn into_completion_record(
        self,
        _: &mut Context,
        _: &mut Registers,
    ) -> ControlFlow<CompletionRecord> {
        ControlFlow::Continue(())
    }
}

impl IntoCompletionRecord for JsError {
    #[inline(always)]
    fn into_completion_record(
        self,
        context: &mut Context,
        registers: &mut Registers,
    ) -> ControlFlow<CompletionRecord> {
        context.handle_error(registers, self)
    }
}

impl IntoCompletionRecord for JsResult<()> {
    #[inline(always)]
    fn into_completion_record(
        self,
        context: &mut Context,
        registers: &mut Registers,
    ) -> ControlFlow<CompletionRecord> {
        match self {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => context.handle_error(registers, err),
        }
    }
}

impl IntoCompletionRecord for ControlFlow<CompletionRecord> {
    #[inline(always)]
    fn into_completion_record(
        self,
        _: &mut Context,
        _: &mut Registers,
    ) -> ControlFlow<CompletionRecord> {
        self
    }
}
