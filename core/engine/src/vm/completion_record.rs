//! An implementation of a `CompletionRecord` for Boa's VM.

#![allow(clippy::inline_always)]

use super::OpStatus;
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
        saved_pc: u32,
    ) -> ControlFlow<CompletionRecord, OpStatus>;
}

impl IntoCompletionRecord for () {
    #[inline(always)]
    fn into_completion_record(
        self,
        _: &mut Context,
        _: u32,
    ) -> ControlFlow<CompletionRecord, OpStatus> {
        ControlFlow::Continue(OpStatus::Finished)
    }
}

impl IntoCompletionRecord for JsError {
    #[inline(always)]
    fn into_completion_record(
        self,
        context: &mut Context,
        _: u32,
    ) -> ControlFlow<CompletionRecord, OpStatus> {
        context.handle_error(self)
    }
}

impl IntoCompletionRecord for JsResult<()> {
    #[inline(always)]
    fn into_completion_record(
        self,
        context: &mut Context,
        _: u32,
    ) -> ControlFlow<CompletionRecord, OpStatus> {
        match self {
            Ok(()) => ControlFlow::Continue(OpStatus::Finished),
            Err(err) => context.handle_error(err),
        }
    }
}

impl IntoCompletionRecord for JsResult<OpStatus> {
    #[inline(always)]
    fn into_completion_record(
        self,
        context: &mut Context,
        saved_pc: u32,
    ) -> ControlFlow<CompletionRecord, OpStatus> {
        match self {
            Ok(OpStatus::Finished) => ControlFlow::Continue(OpStatus::Finished),
            Ok(OpStatus::Pending) => {
                context.vm.frame_mut().pc = saved_pc;
                ControlFlow::Continue(OpStatus::Pending)
            }
            Err(err) => context.handle_error(err),
        }
    }
}

impl IntoCompletionRecord for ControlFlow<CompletionRecord> {
    #[inline(always)]
    fn into_completion_record(
        self,
        _: &mut Context,
        _: u32,
    ) -> ControlFlow<CompletionRecord, OpStatus> {
        match self {
            ControlFlow::Continue(()) => ControlFlow::Continue(OpStatus::Finished),
            ControlFlow::Break(completion_record) => ControlFlow::Break(completion_record),
        }
    }
}

impl IntoCompletionRecord for ControlFlow<CompletionRecord, OpStatus> {
    #[inline(always)]
    fn into_completion_record(
        self,
        context: &mut Context,
        saved_pc: u32,
    ) -> ControlFlow<CompletionRecord, OpStatus> {
        match self {
            ControlFlow::Continue(OpStatus::Finished) => ControlFlow::Continue(OpStatus::Finished),
            ControlFlow::Continue(OpStatus::Pending) => {
                context.vm.frame_mut().pc = saved_pc;
                ControlFlow::Continue(OpStatus::Pending)
            }
            ControlFlow::Break(completion_record) => ControlFlow::Break(completion_record),
        }
    }
}
