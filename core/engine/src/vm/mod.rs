//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    Context, JsError, JsNativeError, JsObject, JsResult, JsString, JsValue, Module,
    builtins::promise::{PromiseCapability, ResolvingFunctions},
    environments::EnvironmentStack,
    error::RuntimeLimitError,
    object::JsFunction,
    realm::Realm,
    script::Script,
};
use boa_gc::{Finalize, Gc, Trace, custom_trace};
use shadow_stack::ShadowStack;
use std::{future::Future, ops::ControlFlow, pin::Pin, task};

#[cfg(feature = "trace")]
use crate::sys::time::Instant;

#[cfg(feature = "trace")]
use std::fmt::Write as _;

#[allow(unused_imports)]
pub(crate) use opcode::{Instruction, InstructionIterator, Opcode};

pub(crate) use {
    call_frame::CallFrameFlags,
    code_block::{
        CodeBlockFlags, Constant, Handler, create_function_object, create_function_object_fast,
    },
    completion_record::CompletionRecord,
    inline_cache::InlineCache,
};

pub use runtime_limits::RuntimeLimits;
pub use {
    call_frame::{CallFrame, GeneratorResumeKind},
    code_block::CodeBlock,
    source_info::{NativeSourceInfo, SourcePath},
};

mod call_frame;
mod code_block;
mod completion_record;
mod inline_cache;
mod runtime_limits;

pub(crate) mod opcode;
pub(crate) mod shadow_stack;
pub(crate) mod source_info;

#[cfg(feature = "flowgraph")]
pub mod flowgraph;

#[cfg(test)]
mod tests;

/// Virtual Machine.
#[derive(Debug)]
pub struct Vm {
    /// The current call frame.
    ///
    /// Whenever a new frame is pushed, it will be swapped into this field.
    /// Then the old frame will get pushed to the [`Self::frames`] stack.
    /// Whenever the current frame gets popped, the last frame on the [`Self::frames`] stack will be swapped into this field.
    ///
    /// By default this is a dummy frame that gets pushed to [`Self::frames`] when the first real frame is pushed.
    pub(crate) frame: CallFrame,

    /// The stack for call frames.
    pub(crate) frames: Vec<CallFrame>,

    pub(crate) stack: Stack,
    pub(crate) return_value: JsValue,

    /// When an error is thrown, the pending exception is set.
    ///
    /// If we throw an empty exception ([`None`]), this means that `return()` was called on a generator,
    /// propagating though the exception handlers and executing the finally code (if any).
    ///
    /// See [`ReThrow`](crate::vm::Opcode::ReThrow) and [`ReThrow`](crate::vm::Opcode::Exception) opcodes.
    ///
    /// This eliminates the conversion between [`crate::JsNativeError`] and [`crate::JsValue`] if not needed.
    pub(crate) pending_exception: Option<JsError>,
    pub(crate) runtime_limits: RuntimeLimits,

    /// This is used to assign a native (rust) function as the active function,
    /// because we don't push a frame for them.
    pub(crate) native_active_function: Option<JsObject>,

    pub(crate) shadow_stack: ShadowStack,

    #[cfg(feature = "trace")]
    pub(crate) trace: bool,
}

/// The stack holds the [`JsValue`]s that the VM is operating on.
///
/// The stack is persistent across frames.
/// It's addressing is relative to the frame pointer.
///
/// The stack stores the following elements:
/// - The function prologue
///   - The `this` value of the function
///   - The function object itself
/// - The arguments of the function
/// - The local function registers
/// - Some manually pushed values like the return value of a function.
///
/// This is the stack layout:
///
/// ```text
///                      Setup by the caller
///   ┌─────────────────────────────────────────────────────────┐ ┌───── register pointer
///   ▼                                                         ▼ ▼
/// | -(2 + N): this | -(1 + N): func | -N: arg1 | ... | -1: argN | 0: reg1 | ... | K: reglK |
///   ▲                              ▲   ▲                      ▲   ▲                        ▲
///   └──────────────────────────────┘   └──────────────────────┘   └────────────────────────┘
///         function prologue                    arguments              Setup by the callee
///   ▲
///   └─ Frame pointer
/// ```
///
/// ### Example
///
/// The following function calls, generate the following stack:
///
/// ```JavaScript
/// function x(a) {
/// }
/// function y(b, c) {
///     return x(b + c)
/// }
///
/// y(1, 2)
/// ```
///
/// ```text
///     caller prologue    caller arguments   callee prologue   callee arguments
///   ┌─────────────────┐   ┌─────────┐   ┌─────────────────┐  ┌──────┐
///   ▼                 ▼   ▼         ▼   │                 ▼  ▼      ▼
/// | 0: undefined | 1: y | 2: 1 | 3: 2 | 4: undefined | 5: x | 6:  3 |
/// ▲                                   ▲                             ▲
/// │       caller register pointer ────┤                             │
/// │                                   │                 callee register pointer
/// │                             callee frame pointer
/// │
/// └─────  caller frame pointer
/// ```
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct Stack {
    stack: Vec<JsValue>,
}

impl Stack {
    /// Creates a new stack with the given capacity.
    fn new(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
        }
    }

    /// Truncate the stack to the given frame.
    pub(crate) fn truncate_to_frame(&mut self, frame: &CallFrame) {
        self.stack.truncate(frame.frame_pointer());
    }

    /// Split the stack at the given frame.
    pub(crate) fn split_off_frame(&mut self, frame: &CallFrame) -> Self {
        let frame_pointer = frame.frame_pointer();
        Self {
            stack: self.stack.split_off(frame_pointer),
        }
    }

    /// Get the `this` value of the given frame.
    pub(crate) fn get_this(&self, frame: &CallFrame) -> JsValue {
        self.stack[frame.this_index()].clone()
    }

    /// Set the `this` value of the given frame.
    pub(crate) fn set_this(&mut self, frame: &CallFrame, this: JsValue) {
        self.stack[frame.this_index()] = this;
    }

    /// Get the function object of the given frame.
    pub(crate) fn get_function(&self, frame: &CallFrame) -> Option<JsObject> {
        if let Some(object) = self.stack[frame.function_index()].as_object() {
            return Some(object.clone());
        }
        None
    }

    /// Get the function arguments of the given frame.
    pub(crate) fn get_arguments(&self, frame: &CallFrame) -> &[JsValue] {
        &self.stack[frame.arguments_range()]
    }

    /// Get a single function argument of the given frame by index.
    pub(crate) fn get_argument(&self, frame: &CallFrame, index: usize) -> Option<&JsValue> {
        self.get_arguments(frame).get(index)
    }

    /// Get the rest arguments of the given frame.
    pub(crate) fn pop_rest_arguments(&mut self, frame: &CallFrame) -> Option<Vec<JsValue>> {
        let argument_count = frame.argument_count as usize;
        let param_count = frame.code_block().parameter_length as usize;
        if argument_count < param_count {
            return None;
        }
        let rp = frame.rp as usize;
        let rest_count = argument_count - param_count + 1;

        Some(self.stack.drain((rp - rest_count)..rp).collect())
    }

    /// Set the promise capability for the given frame.
    #[track_caller]
    pub(crate) fn set_promise_capability(
        &mut self,
        frame: &CallFrame,
        promise_capability: Option<&PromiseCapability>,
    ) {
        debug_assert!(
            frame.code_block().is_async(),
            "Only async functions have a promise capability"
        );

        self.stack[frame.promise_capability_promise_register_index()] = promise_capability
            .map(PromiseCapability::promise)
            .cloned()
            .map_or_else(JsValue::undefined, Into::into);
        self.stack[frame.promise_capability_resolve_register_index()] = promise_capability
            .map(PromiseCapability::resolve)
            .cloned()
            .map_or_else(JsValue::undefined, Into::into);
        self.stack[frame.promise_capability_reject_register_index()] = promise_capability
            .map(PromiseCapability::reject)
            .cloned()
            .map_or_else(JsValue::undefined, Into::into);
    }

    /// Get the promise capability for the given frame.
    #[track_caller]
    pub(crate) fn get_promise_capability(&self, frame: &CallFrame) -> Option<PromiseCapability> {
        if !frame.code_block().is_async() {
            return None;
        }

        let promise = self
            .stack
            .get(frame.promise_capability_promise_register_index())
            .expect("stack must have a promise capability")
            .as_object()?;
        let resolve = self
            .stack
            .get(frame.promise_capability_resolve_register_index())
            .expect("stack must have a resolve function")
            .as_object()
            .and_then(JsFunction::from_object)?;
        let reject = self
            .stack
            .get(frame.promise_capability_reject_register_index())
            .expect("stack must have a reject function")
            .as_object()
            .and_then(JsFunction::from_object)?;

        Some(PromiseCapability {
            promise,
            functions: ResolvingFunctions { resolve, reject },
        })
    }

    /// Set the async generator object for the given frame.
    #[track_caller]
    pub(crate) fn set_async_generator_object(&mut self, frame: &CallFrame, object: JsObject) {
        self.stack[frame.async_generator_object_register_index()] = object.into();
    }

    /// Get the async generator object for the given frame.
    #[track_caller]
    pub(crate) fn async_generator_object(&self, frame: &CallFrame) -> Option<JsObject> {
        if !frame.code_block().is_async_generator() {
            return None;
        }

        self.stack
            .get(frame.async_generator_object_register_index())
            .expect("stack must have an async generator object")
            .as_object()
    }

    /// Push a value on the stack.
    pub(crate) fn push<T>(&mut self, value: T)
    where
        T: Into<JsValue>,
    {
        self.stack.push(value.into());
    }

    /// Pop a value off the stack.
    ///
    /// # Panics
    ///
    /// If there is nothing to pop, then this will panic.
    #[track_caller]
    pub(crate) fn pop(&mut self) -> JsValue {
        self.stack.pop().expect("stack was empty")
    }

    /// Pop the function arguments according to the calling convention.
    /// This will pop the last `argument_count` values from the stack.
    pub(crate) fn calling_convention_pop_arguments(
        &mut self,
        argument_count: usize,
    ) -> Vec<JsValue> {
        let index = self.stack.len() - argument_count;
        self.stack.split_off(index)
    }

    /// Push the function arguments according to the calling convention.
    /// This will push the given values onto the stack.
    pub(crate) fn calling_convention_push_arguments(&mut self, values: &[JsValue]) {
        self.stack.extend_from_slice(values);
    }

    /// Get the function object at the top of the stack according to the calling convention.
    #[track_caller]
    pub(crate) fn calling_convention_get_function(&self, argument_count: usize) -> &JsValue {
        let index = self.stack.len() - 1 - argument_count;
        self.stack
            .get(index)
            .expect("invalid calling convention function index")
    }

    /// Set the function object value at the top of the stack according to the calling convention.
    #[track_caller]
    pub(crate) fn calling_convention_set_function(
        &mut self,
        argument_count: usize,
        function: JsValue,
    ) {
        let index = self.stack.len() - 1 - argument_count;
        self.stack[index] = function;
    }

    /// Set the `this` value at the top of the stack according to the calling convention.
    #[track_caller]
    pub(crate) fn calling_convention_set_this(&mut self, argument_count: usize, function: JsValue) {
        let index = self.stack.len() - 2 - argument_count;
        self.stack[index] = function;
    }

    /// Insert the function arguments at the top of the stack according to the calling convention.
    /// This will insert the given values at the position of the function arguments.
    pub(crate) fn calling_convention_insert_arguments(
        &mut self,
        existing_argument_count: usize,
        arguments: &[JsValue],
    ) {
        let index = self.stack.len() - existing_argument_count;
        self.stack.splice(index..index, arguments.iter().cloned());
    }

    #[cfg(feature = "trace")]
    /// Display the stack trace of the current frame.
    fn display_trace(&self, frame: &CallFrame, frame_count: usize) -> String {
        let mut string = String::from("[ ");
        for (i, (j, value)) in self.stack.iter().enumerate().rev().enumerate() {
            match value {
                value if value.is_callable() => string.push_str("[function]"),
                value if value.is_object() => string.push_str("[object]"),
                value => string.push_str(&value.display().to_string()),
            }

            if frame.frame_pointer() == j {
                let _ = write!(string, " |{frame_count}|");
            } else if i + 1 != self.stack.len() {
                string.push(',');
            }

            string.push(' ');
        }

        string.push(']');
        string
    }
}

/// Active runnable in the current vm context.
#[derive(Debug, Clone, Finalize)]
pub(crate) enum ActiveRunnable {
    Script(Script),
    Module(Module),
}

unsafe impl Trace for ActiveRunnable {
    custom_trace!(this, mark, {
        match this {
            Self::Script(script) => mark(script),
            Self::Module(module) => mark(module),
        }
    });
}

impl Vm {
    /// Creates a new virtual machine.
    pub(crate) fn new(realm: Realm) -> Self {
        Self {
            frames: Vec::with_capacity(16),
            frame: CallFrame::new(
                Gc::new(CodeBlock::new(JsString::default(), 0, true)),
                None,
                EnvironmentStack::new(realm.environment().clone()),
                realm,
            ),
            stack: Stack::new(1024),
            return_value: JsValue::undefined(),
            pending_exception: None,
            runtime_limits: RuntimeLimits::default(),
            native_active_function: None,
            shadow_stack: ShadowStack::default(),
            #[cfg(feature = "trace")]
            trace: false,
        }
    }

    #[track_caller]
    pub(crate) fn set_register(&mut self, index: usize, value: JsValue) {
        self.stack.stack[self.frame.rp as usize + index] = value;
    }

    #[track_caller]
    pub(crate) fn get_register(&self, index: usize) -> &JsValue {
        self.stack
            .stack
            .get(self.frame.rp as usize + index)
            .expect("registers must be initialized")
    }

    /// Retrieves the VM frame.
    #[track_caller]
    pub(crate) fn frame(&self) -> &CallFrame {
        &self.frame
    }

    /// Retrieves the VM frame mutably.
    #[track_caller]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame {
        &mut self.frame
    }

    pub(crate) fn push_frame(&mut self, mut frame: CallFrame) {
        let current_stack_length = self.stack.stack.len();
        frame.set_register_pointer(current_stack_length as u32);

        // NOTE: We need to check if we already pushed the registers,
        //       since generator-like functions push the same call
        //       frame with pre-built stack.
        if !frame.registers_already_pushed() {
            self.stack.stack.resize_with(
                current_stack_length + frame.code_block.register_count as usize,
                JsValue::undefined,
            );
        }

        // Keep carrying the last active runnable in case the current callframe
        // yields.
        if frame.active_runnable.is_none() {
            frame
                .active_runnable
                .clone_from(&self.frame.active_runnable);
        }

        self.shadow_stack
            .push_bytecode(self.frame.pc, frame.code_block().source_info.clone());

        std::mem::swap(&mut self.frame, &mut frame);
        self.frames.push(frame);
    }

    pub(crate) fn push_frame_with_stack(
        &mut self,
        frame: CallFrame,
        this: JsValue,
        function: JsValue,
    ) {
        self.stack.push(this);
        self.stack.push(function);

        self.push_frame(frame);
    }

    pub(crate) fn pop_frame(&mut self) -> Option<CallFrame> {
        if let Some(mut frame) = self.frames.pop() {
            self.shadow_stack.pop();

            std::mem::swap(&mut self.frame, &mut frame);
            Some(frame)
        } else {
            None
        }
    }

    /// Handles an exception thrown at position `pc`.
    ///
    /// Returns `true` if the exception was handled, `false` otherwise.
    #[inline]
    pub(crate) fn handle_exception_at(&mut self, pc: u32) -> bool {
        let frame = self.frame_mut();
        let Some((_, handler)) = frame.code_block().find_handler(pc) else {
            return false;
        };

        let catch_address = handler.handler();
        let environment_sp = frame.env_fp + handler.environment_count;

        // Go to handler location.
        frame.pc = catch_address;

        self.frame.environments.truncate(environment_sp as usize);

        true
    }

    pub(crate) fn get_return_value(&self) -> JsValue {
        self.return_value.clone()
    }

    pub(crate) fn set_return_value(&mut self, value: JsValue) {
        self.return_value = value;
    }

    pub(crate) fn take_return_value(&mut self) -> JsValue {
        std::mem::take(&mut self.return_value)
    }
}

#[allow(clippy::print_stdout)]
#[cfg(feature = "trace")]
impl Context {
    const COLUMN_WIDTH: usize = 26;
    const TIME_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH / 2;
    const OPCODE_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const OPERAND_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const NUMBER_OF_COLUMNS: usize = 4;

    pub(crate) fn trace_call_frame(&self) {
        let frame = self.vm.frame();
        let msg = if self.vm.frames.is_empty() {
            " VM Start ".to_string()
        } else {
            format!(
                " Call Frame -- {} ",
                frame.code_block().name().to_std_string_escaped()
            )
        };

        println!("{}", frame.code_block);
        println!(
            "{msg:-^width$}",
            width = Self::COLUMN_WIDTH * Self::NUMBER_OF_COLUMNS - 10
        );
        println!(
            "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {:<OPERAND_COLUMN_WIDTH$} Stack\n",
            "Time",
            "Opcode",
            "Operands",
            TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
            OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
            OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
        );
    }

    fn trace_execute_instruction<F>(
        &mut self,
        f: F,
        opcode: Opcode,
    ) -> ControlFlow<CompletionRecord>
    where
        F: FnOnce(&mut Context, Opcode) -> ControlFlow<CompletionRecord>,
    {
        let frame = self.vm.frame();
        let (instruction, _) = frame
            .code_block
            .bytecode
            .next_instruction(frame.pc as usize);
        let operands = self
            .vm
            .frame()
            .code_block()
            .instruction_operands(&instruction);

        match opcode {
            Opcode::Call
            | Opcode::CallSpread
            | Opcode::CallEval
            | Opcode::CallEvalSpread
            | Opcode::New
            | Opcode::NewSpread
            | Opcode::Return
            | Opcode::SuperCall
            | Opcode::SuperCallSpread
            | Opcode::SuperCallDerived => {
                println!();
            }
            _ => {}
        }

        let instant = Instant::now();
        let result = self.execute_instruction(f, opcode);
        let duration = instant.elapsed();

        let stack = self
            .vm
            .stack
            .display_trace(self.vm.frame(), self.vm.frames.len() - 1);

        println!(
            "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {stack}",
            format!("{}μs", duration.as_micros()),
            format!("{}", opcode.as_str()),
            TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
            OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
            OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
        );

        result
    }
}

impl Context {
    fn execute_instruction<F>(&mut self, f: F, opcode: Opcode) -> ControlFlow<CompletionRecord>
    where
        F: FnOnce(&mut Context, Opcode) -> ControlFlow<CompletionRecord>,
    {
        f(self, opcode)
    }

    fn execute_one<F>(&mut self, f: F, opcode: Opcode) -> ControlFlow<CompletionRecord>
    where
        F: FnOnce(&mut Context, Opcode) -> ControlFlow<CompletionRecord>,
    {
        #[cfg(feature = "fuzz")]
        {
            use crate::error::EngineError;
            if self.instructions_remaining == 0 {
                return ControlFlow::Break(CompletionRecord::Throw(
                    EngineError::NoInstructionsRemain.into(),
                ));
            }
            self.instructions_remaining -= 1;
        }

        #[cfg(feature = "trace")]
        if self.vm.trace || self.vm.frame().code_block.traceable() {
            self.trace_execute_instruction(f, opcode)
        } else {
            self.execute_instruction(f, opcode)
        }

        #[cfg(not(feature = "trace"))]
        self.execute_instruction(f, opcode)
    }

    fn handle_error(&mut self, mut err: JsError) -> ControlFlow<CompletionRecord> {
        // If we hit the execution step limit, bubble up the error to the
        // (Rust) caller instead of trying to handle as an exception.
        if !err.is_catchable() {
            if err.backtrace.is_none() {
                err.backtrace = Some(
                    self.vm
                        .shadow_stack
                        .take(self.vm.runtime_limits.backtrace_limit(), self.vm.frame.pc),
                );
            }

            let mut frame = None;
            let mut env_fp = self.vm.frame.environments.len();
            loop {
                if self.vm.frame.exit_early() {
                    break;
                }

                env_fp = self.vm.frame.env_fp as usize;

                let Some(f) = self.vm.pop_frame() else {
                    break;
                };
                frame = Some(f);
            }
            self.vm.frame.environments.truncate(env_fp);
            if let Some(frame) = frame {
                self.vm.stack.truncate_to_frame(&frame);
            }
            return ControlFlow::Break(CompletionRecord::Throw(err));
        }

        // Note: -1 because we increment after fetching the opcode.
        let pc = self.vm.frame().pc.saturating_sub(1);
        if self.vm.handle_exception_at(pc) {
            self.vm.pending_exception = Some(err);
            return ControlFlow::Continue(());
        }

        // Inject realm before crossing the function boundary
        let err = err.inject_realm(self.realm().clone());

        self.vm.pending_exception = Some(err);
        self.handle_throw()
    }

    fn handle_return(&mut self) -> ControlFlow<CompletionRecord> {
        let exit_early = self.vm.frame().exit_early();
        self.vm.stack.truncate_to_frame(&self.vm.frame);

        let result = self.vm.take_return_value();
        if exit_early {
            return ControlFlow::Break(CompletionRecord::Normal(result));
        }

        self.vm.stack.push(result);
        self.vm.pop_frame().expect("frame must exist");
        ControlFlow::Continue(())
    }

    fn handle_yield(&mut self) -> ControlFlow<CompletionRecord> {
        let result = self.vm.take_return_value();
        if self.vm.frame().exit_early() {
            return ControlFlow::Break(CompletionRecord::Return(result));
        }

        self.vm.stack.push(result);
        self.vm.pop_frame().expect("frame must exist");
        ControlFlow::Continue(())
    }

    fn handle_throw(&mut self) -> ControlFlow<CompletionRecord> {
        if let Some(err) = &mut self.vm.pending_exception
            && err.backtrace.is_none()
        {
            err.backtrace = Some(
                self.vm
                    .shadow_stack
                    .take(self.vm.runtime_limits.backtrace_limit(), self.vm.frame.pc),
            );
        }

        let mut env_fp = self.vm.frame().env_fp;
        if self.vm.frame().exit_early() {
            self.vm.frame.environments.truncate(env_fp as usize);
            self.vm.stack.truncate_to_frame(&self.vm.frame);
            return ControlFlow::Break(CompletionRecord::Throw(
                self.vm
                    .pending_exception
                    .take()
                    .expect("Err must exist for a CompletionType::Throw"),
            ));
        }

        let mut frame = self.vm.pop_frame().expect("frame must exist");

        loop {
            env_fp = self.vm.frame.env_fp;
            let pc = self.vm.frame.pc;
            let exit_early = self.vm.frame.exit_early();

            if self.vm.handle_exception_at(pc) {
                return ControlFlow::Continue(());
            }

            if exit_early {
                return ControlFlow::Break(CompletionRecord::Throw(
                    self.vm
                        .pending_exception
                        .take()
                        .expect("Err must exist for a CompletionType::Throw"),
                ));
            }

            let Some(f) = self.vm.pop_frame() else {
                break;
            };
            frame = f;
        }
        self.vm.frame.environments.truncate(env_fp as usize);
        self.vm.stack.truncate_to_frame(&frame);
        ControlFlow::Continue(())
    }

    /// Runs the current frame to completion, yielding to the caller each time `budget`
    /// "clock cycles" have passed.
    #[allow(clippy::future_not_send)]
    pub(crate) async fn run_async_with_budget(&mut self, budget: u32) -> CompletionRecord {
        #[cfg(feature = "trace")]
        if self.vm.trace {
            self.trace_call_frame();
        }

        let mut runtime_budget: u32 = budget;

        while let Some(byte) = self
            .vm
            .frame
            .code_block
            .bytecode
            .bytecode
            .get(self.vm.frame.pc as usize)
        {
            let opcode = Opcode::decode(*byte);

            match self.execute_one(
                |context, opcode| {
                    context.execute_bytecode_instruction_with_budget(&mut runtime_budget, opcode)
                },
                opcode,
            ) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(value) => return value,
            }

            if runtime_budget == 0 {
                runtime_budget = budget;
                yield_now().await;
            }
        }

        CompletionRecord::Throw(JsError::from_native(JsNativeError::error()))
    }

    pub(crate) fn run(&mut self) -> CompletionRecord {
        #[cfg(feature = "trace")]
        if self.vm.trace {
            self.trace_call_frame();
        }

        while let Some(byte) = self
            .vm
            .frame
            .code_block
            .bytecode
            .bytecode
            .get(self.vm.frame.pc as usize)
        {
            let opcode = Opcode::decode(*byte);

            match self.execute_one(Self::execute_bytecode_instruction, opcode) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(value) => return value,
            }
        }

        CompletionRecord::Throw(JsError::from_native(JsNativeError::error()))
    }

    /// Checks if we haven't exceeded the defined runtime limits.
    pub(crate) fn check_runtime_limits(&self) -> JsResult<()> {
        // Must throw if the number of recursive calls exceeds the defined limit.
        if self.vm.runtime_limits.recursion_limit() <= self.vm.frames.len() {
            return Err(RuntimeLimitError::Recursion.into());
        }
        // Must throw if the stack size exceeds the defined maximum length.
        if self.vm.runtime_limits.stack_size_limit() <= self.vm.stack.stack.len() {
            return Err(RuntimeLimitError::StackSize.into());
        }

        Ok(())
    }
}

/// Yields once to the executor.
fn yield_now() -> impl Future<Output = ()> {
    struct YieldNow(bool);

    impl Future for YieldNow {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
            if self.0 {
                task::Poll::Ready(())
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                task::Poll::Pending
            }
        }
    }

    YieldNow(false)
}
