//! Runtime support for the ECMAScript Explicit Resource Management proposal.
//!
//! This module provides the core data structures and placeholder algorithms
//! needed to implement the `using` and `await using` declarations, as well
//! as the `DisposableStack` / `AsyncDisposableStack` built-in objects.
//!
//! **Current status:** Structural scaffolding only. Disposal logic is not yet
//! implemented; the bytecompiler does not yet emit resource-tracking opcodes.
//!
//! More information:
//!  - [TC39 Proposal][proposal]
//!  - [Spec text – DisposeResources][spec]
//!
//! [proposal]: https://github.com/tc39/proposal-explicit-resource-management
//! [spec]: https://tc39.es/proposal-explicit-resource-management/#sec-disposeresources

use crate::{Context, JsValue};
use boa_gc::{Finalize, Trace};

// ---------------------------------------------------------------------------
// DisposableResourceHint
// ---------------------------------------------------------------------------

/// The disposal hint associated with a disposable resource.
///
/// Corresponds to the `[[Hint]]` field of the `DisposableResource` Record in
/// the spec.
///
/// More information:
///  - [Spec – DisposableResource Records][spec]
///
/// [spec]: https://tc39.es/proposal-explicit-resource-management/#sec-disposableresource-records
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub(crate) enum DisposableResourceHint {
    /// Synchronous disposal via `Symbol.dispose`.
    SyncDispose,
    /// Asynchronous disposal via `Symbol.asyncDispose`.
    AsyncDispose,
}

// ---------------------------------------------------------------------------
// DisposableResource
// ---------------------------------------------------------------------------

/// A single disposable resource tracked by the runtime.
///
/// Corresponds to the **`DisposableResource` Record** defined in the
/// Explicit Resource Management proposal.
///
/// More information:
///  - [Spec – DisposableResource Records][spec]
///
/// [spec]: https://tc39.es/proposal-explicit-resource-management/#sec-disposableresource-records
#[derive(Debug, Clone, Trace, Finalize)]
pub(crate) struct DisposableResource {
    /// `[[ResourceValue]]` – the value that was bound by the `using` declaration.
    value: JsValue,

    /// `[[DisposeMethod]]` – the dispose function (obtained from `Symbol.dispose`
    /// or `Symbol.asyncDispose` on the value).
    dispose_method: JsValue,

    /// `[[Hint]]` – whether this resource uses sync or async disposal.
    #[unsafe_ignore_trace]
    #[allow(dead_code)]
    hint: DisposableResourceHint,
}

impl DisposableResource {
    /// Creates a new `DisposableResource`.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        value: JsValue,
        dispose_method: JsValue,
        hint: DisposableResourceHint,
    ) -> Self {
        Self {
            value,
            dispose_method,
            hint,
        }
    }

    /// Returns the resource value.
    #[inline]
    #[must_use]
    pub(crate) const fn value(&self) -> &JsValue {
        &self.value
    }

    /// Returns the dispose method.
    #[inline]
    #[must_use]
    pub(crate) const fn dispose_method(&self) -> &JsValue {
        &self.dispose_method
    }

    /// Returns the disposal hint.
    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn hint(&self) -> &DisposableResourceHint {
        &self.hint
    }
}

// ---------------------------------------------------------------------------
// DisposableResourceStack
// ---------------------------------------------------------------------------

/// A stack of [`DisposableResource`]s associated with a lexical scope.
///
/// When a `using` or `await using` declaration is evaluated, the runtime
/// pushes a resource onto the stack. When the scope exits (normally or
/// abruptly), the resources are disposed in reverse order via
/// [`dispose_resources`].
///
/// Corresponds to the `[[DisposeCapability]]` field on certain Environment
/// Records in the proposal spec.
///
/// More information:
///  - [Spec – DisposeResources][spec]
///
/// [spec]: https://tc39.es/proposal-explicit-resource-management/#sec-disposeresources
#[derive(Debug, Clone, Default, Trace, Finalize)]
pub(crate) struct DisposableResourceStack {
    /// Resources in declaration order. Disposal happens in *reverse* order.
    resources: Vec<DisposableResource>,
}

impl DisposableResourceStack {
    /// Creates an empty resource stack.
    #[inline]
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Pushes a resource onto the stack.
    #[inline]
    pub(crate) fn push(&mut self, resource: DisposableResource) {
        self.resources.push(resource);
    }

    /// Returns `true` if there are no tracked resources.
    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Returns the number of tracked resources.
    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.resources.len()
    }

    /// Returns an iterator over the resources in *reverse* (disposal) order.
    #[inline]
    pub(crate) fn drain_reversed(&mut self) -> impl Iterator<Item = DisposableResource> + '_ {
        self.resources.drain(..).rev()
    }
}

// ---------------------------------------------------------------------------
// dispose_resources (placeholder)
// ---------------------------------------------------------------------------

/// `DisposeResources ( disposeCapability, completion )`
///
/// Performs synchronous disposal of all resources tracked by the given
/// `DisposableResourceStack`, in reverse order.
///
/// # Current status
///
/// **This is a placeholder.** The actual algorithm is defined in:
/// <https://tc39.es/proposal-explicit-resource-management/#sec-disposeresources>
///
/// The full implementation will:
/// 1. Iterate resources in reverse order.
/// 2. Call each resource's `[[DisposeMethod]]`.
/// 3. Collect any thrown errors into a `SuppressedError`.
/// 4. Return the original completion (or a `SuppressedError` if disposal failed).
///
/// # Parameters
/// - `stack`: The disposal stack for the exiting scope.
///
/// # Returns
/// Currently a no-op that simply drains the stack.
pub(crate) fn dispose_resources(context: &mut Context, stack: &mut DisposableResourceStack) {
    // TODO: Implement full spec-compliant disposal per
    // https://tc39.es/proposal-explicit-resource-management/#sec-disposeresources
    //
    // Steps (from spec):
    //   1. Assert: disposable is a DisposableResource Record.
    //   2. Let result be Completion(Call(disposable.[[DisposeMethod]], disposable.[[ResourceValue]])).
    //   3. If result is a throw completion, then
    //      a. If completion is a throw completion, then
    //         i. Set result to ThrowCompletion(a newly created SuppressedError ...).
    //      b. Set completion to result.
    //   4. Return completion.

    // Drain resources in reverse order (dispose newest first).
    for resource in stack.drain_reversed() {
        if let Some(function) = resource.dispose_method().as_callable() {
            // Call the dispose method; ignore errors for now
            // (SuppressedError aggregation will be implemented later).
            drop(function.call(resource.value(), &[], context));
        }
    }
}

// TODO:
// - Hook into using declarations in bytecompiler (emit AddDisposableResource opcode)
// - Implement DisposeResources algorithm (call dispose methods, handle SuppressedError)
// - Add support for async disposal (await using)
// - Implement DisposableStack / AsyncDisposableStack builtins
// - Wire DisposableResourceStack into CallFrame or environment records
