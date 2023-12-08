//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    environments::EnvironmentStack, realm::Realm, script::Script, vm::code_block::Readable,
    Context, JsError, JsNativeError, JsObject, JsResult, JsValue, Module,
};

use boa_gc::{custom_trace, Finalize, Trace};
use boa_profiler::Profiler;
use std::{future::Future, mem::size_of, ops::ControlFlow, pin::Pin, task};

#[cfg(feature = "trace")]
use crate::sys::time::Instant;

mod call_frame;
mod code_block;
mod completion_record;
mod inline_cache;
mod opcode;
#[cfg(feature = "trace")]
pub mod trace;

mod runtime_limits;

#[cfg(feature = "flowgraph")]
pub mod flowgraph;

pub(crate) use inline_cache::InlineCache;

#[cfg(feature = "trace")]
use trace::{TraceAction, VmTrace};

// TODO: see if this can be exposed on all features.
#[allow(unused_imports)]
pub(crate) use opcode::{Instruction, InstructionIterator, Opcode, VaryingOperandKind};
pub use runtime_limits::RuntimeLimits;

pub use {
    call_frame::{CallFrame, GeneratorResumeKind},
    code_block::CodeBlock,
};

pub(crate) use {
    call_frame::CallFrameFlags,
    code_block::{
        create_function_object, create_function_object_fast, CodeBlockFlags, Constant, Handler,
    },
    completion_record::CompletionRecord,
    opcode::BindingOpcode,
};

#[cfg(test)]
mod tests;

/// Virtual Machine.
#[derive(Debug)]
pub struct Vm {
    pub(crate) frames: Vec<CallFrame>,
    pub(crate) stack: Vec<JsValue>,
    pub(crate) return_value: JsValue,

    /// When an error is thrown, the pending exception is set.
    ///
    /// If we throw an empty exception ([`None`]), this means that `return()` was called on a generator,
    /// propagating though the exception handlers and executing the finally code (if any).
    ///
    /// See [`ReThrow`](crate::vm::Opcode::ReThrow) and [`ReThrow`](crate::vm::Opcode::Exception) opcodes.
    ///
    /// This is also used to eliminates [`crate::JsNativeError`] to opaque conversion if not needed.
    pub(crate) pending_exception: Option<JsError>,
    pub(crate) environments: EnvironmentStack,
    pub(crate) runtime_limits: RuntimeLimits,

    /// This is used to assign a native (rust) function as the active function,
    /// because we don't push a frame for them.
    pub(crate) native_active_function: Option<JsObject>,

    /// realm holds both the global object and the environment
    pub(crate) realm: Realm,

    #[cfg(feature = "trace")]
    pub(crate) trace: VmTrace,
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
            stack: Vec::with_capacity(1024),
            return_value: JsValue::undefined(),
            environments: EnvironmentStack::new(realm.environment().clone()),
            pending_exception: None,
            runtime_limits: RuntimeLimits::default(),
            native_active_function: None,
            realm,
            #[cfg(feature = "trace")]
            trace: VmTrace::default(),
        }
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

    #[track_caller]
    pub(crate) fn read<T: Readable>(&mut self) -> T {
        let value = self.frame().code_block.read::<T>(self.frame().pc as usize);
        self.frame_mut().pc += size_of::<T>() as u32;
        value
    }

    /// Retrieves the VM frame
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    #[track_caller]
    pub(crate) fn frame(&self) -> &CallFrame {
        self.frames.last().expect("no frame found")
    }

    /// Retrieves the VM frame mutably
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    #[track_caller]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("no frame found")
    }

    pub(crate) fn push_frame(&mut self, mut frame: CallFrame) {
        let current_stack_length = self.stack.len();
        frame.set_register_pointer(current_stack_length as u32);
        std::mem::swap(&mut self.environments, &mut frame.environments);
        std::mem::swap(&mut self.realm, &mut frame.realm);

        // NOTE: We need to check if we already pushed the registers,
        //       since generator-like functions push the same call
        //       frame with pre-built stack.
        if !frame.registers_already_pushed() {
            let register_count = frame.code_block().register_count;
            self.stack.resize_with(
                current_stack_length + register_count as usize,
                JsValue::undefined,
            );
        }

        // Keep carrying the last active runnable in case the current callframe
        // yields.
        if frame.active_runnable.is_none() {
            frame.active_runnable = self.frames.last().and_then(|fr| fr.active_runnable.clone());
        }

        self.frames.push(frame);

        #[cfg(feature = "trace")]
        self.trace.trace_call_frame(self);
    }

    pub(crate) fn push_frame_with_stack(
        &mut self,
        frame: CallFrame,
        this: JsValue,
        function: JsValue,
    ) {
        self.push(this);
        self.push(function);

        self.push_frame(frame);

        #[cfg(feature = "trace")]
        self.trace.trace_call_frame(self);
    }

    pub(crate) fn pop_frame(&mut self) -> Option<CallFrame> {
        let mut frame = self.frames.pop();
        if let Some(frame) = &mut frame {
            std::mem::swap(&mut self.environments, &mut frame.environments);
            std::mem::swap(&mut self.realm, &mut frame.realm);
        }

        frame
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
        let sp = frame.rp + handler.stack_count;

        // Go to handler location.
        frame.pc = catch_address;

        self.environments.truncate(environment_sp as usize);
        self.stack.truncate(sp as usize);

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

    pub(crate) fn pop_n_values(&mut self, n: usize) -> Vec<JsValue> {
        let at = self.stack.len() - n;
        self.stack.split_off(at)
    }

    pub(crate) fn push_values(&mut self, values: &[JsValue]) {
        self.stack.extend_from_slice(values);
    }

    pub(crate) fn insert_values_at(&mut self, values: &[JsValue], at: usize) {
        self.stack.splice(at..at, values.iter().cloned());
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CompletionType {
    Normal,
    Return,
    Throw,
    Yield,
}

#[cfg(feature = "trace")]
impl Context {
    fn trace_execute_instruction<F>(&mut self, f: F) -> JsResult<CompletionType>
    where
        F: FnOnce(Opcode, &mut Context) -> JsResult<CompletionType>,
    {
        let bytecodes = &self.vm.frame().code_block.bytecode;
        let pc = self.vm.frame().pc as usize;
        let (_, varying_operand_kind, instruction) = InstructionIterator::with_pc(bytecodes, pc)
            .next()
            .expect("There should be an instruction left");
        let operands = self
            .vm
            .frame()
            .code_block
            .instruction_operands(&instruction);

        let opcode = instruction.opcode();
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
        let result = self.execute_instruction(f);
        let duration = instant.elapsed();

        let fp = self
            .vm
            .frames
            .last()
            .map(CallFrame::fp)
            .map(|fp| fp as usize);

        let stack = {
            let mut stack = String::from("[ ");
            for (i, (j, value)) in self.vm.stack.iter().enumerate().rev().enumerate() {
                match value {
                    value if value.is_callable() => stack.push_str("[function]"),
                    value if value.is_object() => stack.push_str("[object]"),
                    value => stack.push_str(&value.display().to_string()),
                }

                if fp == Some(j) {
                    let frame_index = self.vm.frames.len() - 1;
                    stack.push_str(&format!(" |{frame_index}|"));
                } else if i + 1 != self.vm.stack.len() {
                    stack.push(',');
                }

                stack.push(' ');
            }

            stack.push(']');
            stack
        };

        let varying_operand_kind = match varying_operand_kind {
            VaryingOperandKind::U8 => "",
            VaryingOperandKind::U16 => ".U16",
            VaryingOperandKind::U32 => ".U32",
        };

        self.vm.trace.trace_instruction(
            duration.as_micros(),
            varying_operand_kind,
            opcode.as_str(),
            &operands,
            &stack,
        );

        result
    }
}

impl Context {
    fn execute_instruction<F>(&mut self, f: F) -> JsResult<CompletionType>
    where
        F: FnOnce(Opcode, &mut Context) -> JsResult<CompletionType>,
    {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");

            let frame = self.vm.frame_mut();

            let pc = frame.pc;
            let opcode = frame.code_block.bytecode[pc as usize].into();
            frame.pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        f(opcode, self)
    }

    fn execute_one<F>(&mut self, f: F) -> ControlFlow<CompletionRecord>
    where
        F: FnOnce(Opcode, &mut Context) -> JsResult<CompletionType>,
    {
        #[cfg(feature = "fuzz")]
        {
            if self.instructions_remaining == 0 {
                return ControlFlow::Break(CompletionRecord::Throw(JsError::from_native(
                    JsNativeError::no_instructions_remain(),
                )));
            }
            self.instructions_remaining -= 1;
        }

        #[cfg(feature = "trace")]
        let result = if self.vm.trace.should_trace(&self.vm) == TraceAction::None {
            self.execute_instruction(f)
        } else {
            self.trace_execute_instruction(f)
        };

        #[cfg(not(feature = "trace"))]
        let result = self.execute_instruction(f);

        let result = match result {
            Ok(result) => result,
            Err(err) => {
                // If we hit the execution step limit, bubble up the error to the
                // (Rust) caller instead of trying to handle as an exception.
                if !err.is_catchable() {
                    let mut fp = self.vm.stack.len();
                    let mut env_fp = self.vm.environments.len();
                    while let Some(frame) = self.vm.frames.last() {
                        if frame.exit_early() {
                            break;
                        }

                        fp = frame.fp() as usize;
                        env_fp = frame.env_fp as usize;
                        self.vm.pop_frame();
                    }
                    self.vm.environments.truncate(env_fp);
                    self.vm.stack.truncate(fp);
                    #[cfg(feature = "trace")]
                    self.vm.trace.trace_frame_end(&self.vm, "Throw");
                    return ControlFlow::Break(CompletionRecord::Throw(err));
                }

                // Note: -1 because we increment after fetching the opcode.
                let pc = self.vm.frame().pc.saturating_sub(1);
                if self.vm.handle_exception_at(pc) {
                    self.vm.pending_exception = Some(err);
                    return ControlFlow::Continue(());
                }

                // Inject realm before crossing the function boundry
                let err = err.inject_realm(self.realm().clone());

                self.vm.pending_exception = Some(err);
                CompletionType::Throw
            }
        };

        match result {
            CompletionType::Normal => {}
            CompletionType::Return => {
                let fp = self.vm.frame().fp() as usize;
                self.vm.stack.truncate(fp);

                #[cfg(feature = "trace")]
                self.vm.trace.trace_frame_end(&self.vm, "Return");

                let result = self.vm.take_return_value();
                if self.vm.frame().exit_early() {
                    return ControlFlow::Break(CompletionRecord::Normal(result));
                }

                self.vm.push(result);
                self.vm.pop_frame();
            }
            CompletionType::Throw => {
                let mut fp = self.vm.frame().fp();
                let mut env_fp = self.vm.frame().env_fp;
                #[cfg(feature = "trace")]
                self.vm.trace.trace_frame_end(&self.vm, "Throw");
                if self.vm.frame().exit_early() {
                    self.vm.environments.truncate(env_fp as usize);
                    self.vm.stack.truncate(fp as usize);
                    return ControlFlow::Break(CompletionRecord::Throw(
                        self.vm
                            .pending_exception
                            .take()
                            .expect("Err must exist for a CompletionType::Throw"),
                    ));
                }

                self.vm.pop_frame();

                while let Some(frame) = self.vm.frames.last_mut() {
                    fp = frame.fp();
                    env_fp = frame.env_fp;
                    let pc = frame.pc;
                    let exit_early = frame.exit_early();

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

                    self.vm.pop_frame();
                }
                self.vm.environments.truncate(env_fp as usize);
                self.vm.stack.truncate(fp as usize);
            }
            // Early return immediately.
            CompletionType::Yield => {
                #[cfg(feature = "trace")]
                self.vm.trace.trace_frame_end(&self.vm, "Yield");

                let result = self.vm.take_return_value();
                if self.vm.frame().exit_early() {
                    return ControlFlow::Break(CompletionRecord::Return(result));
                }

                self.vm.push(result);
                self.vm.pop_frame();
            }
        }

        ControlFlow::Continue(())
    }

    /// Runs the current frame to completion, yielding to the caller each time `budget`
    /// "clock cycles" have passed.
    #[allow(clippy::future_not_send)]
    pub(crate) async fn run_async_with_budget(&mut self, budget: u32) -> CompletionRecord {
        let _timer = Profiler::global().start_event("run_async_with_budget", "vm");

        let mut runtime_budget: u32 = budget;

        loop {
            match self.execute_one(|opcode, context| {
                opcode.spend_budget_and_execute(context, &mut runtime_budget)
            }) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(record) => return record,
            }

            if runtime_budget == 0 {
                runtime_budget = budget;
                yield_now().await;
            }
        }
    }

    pub(crate) fn run(&mut self) -> CompletionRecord {
        let _timer = Profiler::global().start_event("run", "vm");

        loop {
            match self.execute_one(Opcode::execute) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(value) => return value,
            }
        }
    }

    /// Checks if we haven't exceeded the defined runtime limits.
    pub(crate) fn check_runtime_limits(&self) -> JsResult<()> {
        // Must throw if the number of recursive calls exceeds the defined limit.
        if self.vm.runtime_limits.recursion_limit() <= self.vm.frames.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("exceeded maximum number of recursive calls")
                .into());
        }
        // Must throw if the stack size exceeds the defined maximum length.
        if self.vm.runtime_limits.stack_size_limit() <= self.vm.stack.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("exceeded maximum call stack length")
                .into());
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
