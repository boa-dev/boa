//! Wrapper methods on [`Context`] for VM access.
//!
//! These methods encapsulate access to `Context`'s internal `Vm` field,
//! ensuring external code doesn't depend on Context's internal structure.

use crate::{
    JsError, JsResult, JsValue,
    builtins::promise::PromiseCapability,
    object::JsObject,
    vm::{CallFrame, Stack, shadow_stack::ShadowStack},
};

use super::Context;

// === Register Access ===

impl Context {
    /// Get a register value by index.
    #[track_caller]
    #[inline]
    pub(crate) fn get_register(&self, index: usize) -> &JsValue {
        self.vm.get_register(index)
    }

    /// Set a register value by index.
    #[track_caller]
    #[inline]
    pub(crate) fn set_register(&mut self, index: usize, value: JsValue) {
        self.vm.set_register(index, value);
    }

    /// Take the value from a register, replacing it with `undefined`.
    #[track_caller]
    #[inline]
    pub(crate) fn take_register(&mut self, index: usize) -> JsValue {
        self.vm.take_register(index)
    }
}

// === Frame Access ===

impl Context {
    /// Retrieves the current VM call frame.
    #[track_caller]
    #[inline]
    pub(crate) fn frame(&self) -> &CallFrame {
        self.vm.frame()
    }

    /// Retrieves the current VM call frame mutably.
    #[track_caller]
    #[inline]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame {
        self.vm.frame_mut()
    }

    /// Returns a slice of all call frames.
    pub(crate) fn frames(&self) -> &[CallFrame] {
        &self.vm.frames
    }

    /// Returns a mutable slice of all call frames.
    pub(crate) fn frames_mut(&mut self) -> &mut [CallFrame] {
        &mut self.vm.frames
    }

    /// Push a call frame onto the VM.
    pub(crate) fn push_frame(&mut self, frame: CallFrame) {
        self.vm.push_frame(frame);
    }

    /// Push a call frame with this/function values onto the stack.
    pub(crate) fn push_frame_with_stack(
        &mut self,
        frame: CallFrame,
        this: JsValue,
        function: JsValue,
    ) {
        self.vm.push_frame_with_stack(frame, this, function);
    }

    /// Pop the current call frame.
    pub(crate) fn pop_frame(&mut self) -> Option<CallFrame> {
        self.vm.pop_frame()
    }
}

// === Return Value ===

impl Context {
    /// Get the return value (cloned).
    pub(crate) fn get_return_value(&self) -> JsValue {
        self.vm.get_return_value()
    }

    /// Set the return value.
    pub(crate) fn set_return_value(&mut self, value: JsValue) {
        self.vm.set_return_value(value);
    }

    /// Take the return value, replacing it with undefined.
    pub(crate) fn take_return_value(&mut self) -> JsValue {
        self.vm.take_return_value()
    }
}

// === Exception Handling ===

impl Context {
    /// Set the pending exception.
    pub(crate) fn set_pending_exception(&mut self, exception: JsError) {
        self.vm.pending_exception = Some(exception);
    }

    /// Take the pending exception, leaving None.
    pub(crate) fn take_pending_exception(&mut self) -> Option<JsError> {
        self.vm.pending_exception.take()
    }

    /// Check if there's a pending exception.
    pub(crate) fn has_pending_exception(&self) -> bool {
        self.vm.pending_exception.is_some()
    }

    /// Handle an exception at the given PC. Returns true if handled.
    #[inline]
    pub(crate) fn handle_exception_at(&mut self, pc: u32) -> bool {
        self.vm.handle_exception_at(pc)
    }
}

// === Promise Capability ===

impl Context {
    /// Set the promise capability for the current frame.
    #[track_caller]
    pub(crate) fn set_promise_capability(
        &mut self,
        promise_capability: PromiseCapability,
    ) -> JsResult<()> {
        self.vm.set_promise_capability(promise_capability)
    }

    /// Get the async generator object for the current frame.
    #[track_caller]
    pub(crate) fn async_generator_object(&self) -> Option<JsObject> {
        self.vm.async_generator_object()
    }
}

// === Native Function ===

impl Context {
    /// Set the native active function object.
    pub(crate) fn set_native_active_function(&mut self, function: Option<JsObject>) {
        self.vm.native_active_function = function;
    }
}

// === Host Call Depth ===

impl Context {
    /// Increment the host call depth.
    pub(crate) fn inc_host_call_depth(&mut self) {
        self.vm.host_call_depth += 1;
    }

    /// Decrement the host call depth (saturating).
    pub(crate) fn dec_host_call_depth(&mut self) {
        self.vm.host_call_depth = self.vm.host_call_depth.saturating_sub(1);
    }
}

// === Shadow Stack ===

impl Context {
    /// Get a reference to the shadow stack.
    pub(crate) fn shadow_stack(&self) -> &ShadowStack {
        &self.vm.shadow_stack
    }

    /// Get a mutable reference to the shadow stack.
    pub(crate) fn shadow_stack_mut(&mut self) -> &mut ShadowStack {
        &mut self.vm.shadow_stack
    }
}

// === Stack Operations ===

impl Context {
    /// Push a value onto the VM stack.
    pub(crate) fn stack_push<T: Into<JsValue>>(&mut self, value: T) {
        self.vm.stack.push(value);
    }

    /// Pop a value from the VM stack.
    #[track_caller]
    pub(crate) fn stack_pop(&mut self) -> JsValue {
        self.vm.stack.pop()
    }

    /// Clone the VM stack.
    #[cfg(feature = "experimental")]
    pub(crate) fn stack_clone(&self) -> Stack {
        self.vm.stack.clone()
    }

    /// Replace the VM stack.
    #[cfg(feature = "experimental")]
    pub(crate) fn stack_replace(&mut self, stack: Stack) {
        self.vm.stack = stack;
    }

    /// Swap the VM stack with another stack.
    pub(crate) fn stack_swap(&mut self, stack: &mut Stack) {
        std::mem::swap(&mut self.vm.stack, stack);
    }

    /// Truncate the stack to an arbitrary frame's frame pointer.
    pub(crate) fn stack_truncate_to(&mut self, frame: &CallFrame) {
        self.vm.stack.truncate_to_frame(frame);
    }

    /// Split the stack at an arbitrary frame.
    pub(crate) fn stack_split_off_at(&mut self, frame: &CallFrame) -> Stack {
        self.vm.stack.split_off_frame(frame)
    }

    /// Get the `this` value of the current frame.
    pub(crate) fn stack_get_this(&self) -> JsValue {
        self.vm.stack.get_this(self.vm.frame())
    }

    /// Set the `this` value of the current frame.
    pub(crate) fn stack_set_this(&mut self, this: JsValue) {
        let frame = self.vm.frames.last().expect("frame must exist");
        self.vm.stack.set_this(frame, this);
    }

    /// Get the function object of the current frame.
    pub(crate) fn stack_get_function(&self) -> Option<JsObject> {
        self.vm.stack.get_function(self.vm.frame())
    }

    /// Get the function arguments of the current frame.
    pub(crate) fn stack_get_arguments(&self) -> &[JsValue] {
        self.vm.stack.get_arguments(self.vm.frame())
    }

    /// Pop rest arguments from the current frame.
    pub(crate) fn stack_pop_rest_arguments(&mut self) -> Option<Vec<JsValue>> {
        let frame = self.vm.frames.last().expect("frame must exist");
        self.vm.stack.pop_rest_arguments(frame)
    }

    /// Pop function arguments according to the calling convention.
    pub(crate) fn stack_calling_convention_pop_arguments(
        &mut self,
        argument_count: usize,
    ) -> Vec<JsValue> {
        self.vm
            .stack
            .calling_convention_pop_arguments(argument_count)
    }

    /// Push function arguments according to the calling convention.
    pub(crate) fn stack_calling_convention_push_arguments(&mut self, values: &[JsValue]) {
        self.vm.stack.calling_convention_push_arguments(values);
    }

    /// Get the function object at the top of the stack according to the calling convention.
    #[track_caller]
    pub(crate) fn stack_calling_convention_get_function(&self, argument_count: usize) -> &JsValue {
        self.vm
            .stack
            .calling_convention_get_function(argument_count)
    }
}

// === Runtime Limits ===

impl Context {
    /// Get the loop iteration limit.
    pub(crate) fn loop_iteration_limit(&self) -> u64 {
        self.vm.runtime_limits.loop_iteration_limit()
    }
}

// === Kept Alive ===

impl Context {
    /// Add an object to the kept-alive list (prevents GC collection).
    pub(crate) fn add_kept_alive(&mut self, object: JsObject) {
        self.kept_alive.push(object);
    }
}
