//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    environments::EnvironmentStack, realm::Realm, script::Script, vm::code_block::Readable,
    Context, JsError, JsNativeError, JsObject, JsResult, JsString, JsValue, Module,
};

use boa_gc::{custom_trace, Finalize, Gc, Trace};
use boa_profiler::Profiler;
use std::{fmt::Write as _, future::Future, mem::size_of, ops::ControlFlow, pin::Pin, task};

#[cfg(feature = "trace")]
use crate::sys::time::Instant;

mod call_frame;
mod code_block;
mod completion_record;
mod inline_cache;
mod opcode;
mod runtime_limits;

#[cfg(feature = "flowgraph")]
pub mod flowgraph;

pub(crate) use inline_cache::InlineCache;

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
    /// The current call frame.
    ///
    /// Whenever a new frame is pushed, it will be swaped into this field.
    /// Then the old frame will get pushed to the [`Self::frames`] stack.
    /// Whenever the current frame gets poped, the last frame on the [`Self::frames`] stack will be swaped into this field.
    ///
    /// By default this is a dummy frame that gets pushed to [`Self::frames`] when the first real frame is pushed.
    pub(crate) frame: CallFrame,

    /// The stack for call frames.
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
    pub(crate) trace: bool,
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
        let _timer = Profiler::global().start_event("VM::new", "VM");
        Self {
            frames: Vec::with_capacity(16),
            frame: CallFrame::new(
                Gc::new(CodeBlock::new(JsString::default(), 0, true)),
                None,
                EnvironmentStack::new(realm.environment().clone()),
                realm.clone(),
            ),
            stack: Vec::with_capacity(1024),
            return_value: JsValue::undefined(),
            environments: EnvironmentStack::new(realm.environment().clone()),
            pending_exception: None,
            runtime_limits: RuntimeLimits::default(),
            native_active_function: None,
            realm,
            #[cfg(feature = "trace")]
            trace: false,
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
        let frame = self.frame_mut();
        let value = frame.code_block.read::<T>(frame.pc as usize);
        frame.pc += size_of::<T>() as u32;
        value
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
        let current_stack_length = self.stack.len();
        frame.set_register_pointer(current_stack_length as u32);
        std::mem::swap(&mut self.environments, &mut frame.environments);
        std::mem::swap(&mut self.realm, &mut frame.realm);

        // NOTE: We need to check if we already pushed the registers,
        //       since generator-like functions push the same call
        //       frame with pre-built stack.
        if !frame.registers_already_pushed() {
            self.stack
                .resize_with(current_stack_length, JsValue::undefined);
        }

        // Keep carrying the last active runnable in case the current callframe
        // yields.
        if frame.active_runnable.is_none() {
            frame
                .active_runnable
                .clone_from(&self.frame.active_runnable);
        }

        std::mem::swap(&mut self.frame, &mut frame);
        self.frames.push(frame);
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
    }

    pub(crate) fn pop_frame(&mut self) -> Option<CallFrame> {
        if let Some(mut frame) = self.frames.pop() {
            std::mem::swap(&mut self.frame, &mut frame);
            std::mem::swap(&mut self.environments, &mut frame.environments);
            std::mem::swap(&mut self.realm, &mut frame.realm);
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

        self.environments.truncate(environment_sp as usize);

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
        registers: &mut Registers,
    ) -> JsResult<CompletionType>
    where
        F: FnOnce(Opcode, &mut Registers, &mut Context) -> JsResult<CompletionType>,
    {
        let frame = self.vm.frame();
        let bytecodes = &frame.code_block.bytecode;
        let pc = frame.pc as usize;
        let (_, varying_operand_kind, instruction) = InstructionIterator::with_pc(bytecodes, pc)
            .next()
            .expect("There should be an instruction left");
        let operands = frame.code_block.instruction_operands(&instruction);

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
        let result = self.execute_instruction(f, registers);
        let duration = instant.elapsed();

        let fp = if self.vm.frames.is_empty() {
            None
        } else {
            Some(self.vm.frame.fp() as usize)
        };

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
                    let _ = write!(stack, " |{frame_index}|");
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

        println!(
            "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {stack}",
            format!("{}μs", duration.as_micros()),
            format!("{}{varying_operand_kind}", opcode.as_str()),
            TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
            OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
            OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
        );

        result
    }
}

impl Context {
    fn execute_instruction<F>(
        &mut self,
        f: F,
        registers: &mut Registers,
    ) -> JsResult<CompletionType>
    where
        F: FnOnce(Opcode, &mut Registers, &mut Context) -> JsResult<CompletionType>,
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

        f(opcode, registers, self)
    }

    fn execute_one<F>(&mut self, f: F, registers: &mut Registers) -> ControlFlow<CompletionRecord>
    where
        F: FnOnce(Opcode, &mut Registers, &mut Context) -> JsResult<CompletionType>,
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
        let result = if self.vm.trace || self.vm.frame().code_block.traceable() {
            self.trace_execute_instruction(f, registers)
        } else {
            self.execute_instruction(f, registers)
        };

        #[cfg(not(feature = "trace"))]
        let result = self.execute_instruction(f, registers);

        let result = match result {
            Ok(result) => result,
            Err(err) => {
                // If we hit the execution step limit, bubble up the error to the
                // (Rust) caller instead of trying to handle as an exception.
                if !err.is_catchable() {
                    let mut fp = self.vm.stack.len();
                    let mut env_fp = self.vm.environments.len();
                    loop {
                        if self.vm.frame.exit_early() {
                            break;
                        }

                        fp = self.vm.frame.fp() as usize;
                        env_fp = self.vm.frame.env_fp as usize;

                        if self.vm.pop_frame().is_some() {
                            registers
                                .pop_function(self.vm.frame().code_block().register_count as usize);
                        } else {
                            break;
                        }
                    }
                    self.vm.environments.truncate(env_fp);
                    self.vm.stack.truncate(fp);
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
                let frame = self.vm.frame();
                let fp = frame.fp() as usize;
                let exit_early = frame.exit_early();
                self.vm.stack.truncate(fp);

                let result = self.vm.take_return_value();
                if exit_early {
                    return ControlFlow::Break(CompletionRecord::Normal(result));
                }

                self.vm.push(result);
                self.vm.pop_frame().expect("frame must exist");
                registers.pop_function(self.vm.frame().code_block().register_count as usize);
            }
            CompletionType::Throw => {
                let frame = self.vm.frame();
                let mut fp = frame.fp();
                let mut env_fp = frame.env_fp;
                if frame.exit_early() {
                    self.vm.environments.truncate(env_fp as usize);
                    self.vm.stack.truncate(fp as usize);
                    return ControlFlow::Break(CompletionRecord::Throw(
                        self.vm
                            .pending_exception
                            .take()
                            .expect("Err must exist for a CompletionType::Throw"),
                    ));
                }

                self.vm.pop_frame().expect("frame must exist");
                registers.pop_function(self.vm.frame().code_block().register_count as usize);

                loop {
                    fp = self.vm.frame.fp();
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

                    if self.vm.pop_frame().is_some() {
                        registers
                            .pop_function(self.vm.frame().code_block().register_count as usize);
                    } else {
                        break;
                    }
                }
                self.vm.environments.truncate(env_fp as usize);
                self.vm.stack.truncate(fp as usize);
            }
            // Early return immediately.
            CompletionType::Yield => {
                let result = self.vm.take_return_value();
                if self.vm.frame().exit_early() {
                    return ControlFlow::Break(CompletionRecord::Return(result));
                }

                self.vm.push(result);
                self.vm.pop_frame().expect("frame must exist");
                registers.pop_function(self.vm.frame().code_block().register_count as usize);
            }
        }

        ControlFlow::Continue(())
    }

    /// Runs the current frame to completion, yielding to the caller each time `budget`
    /// "clock cycles" have passed.
    #[allow(clippy::future_not_send)]
    pub(crate) async fn run_async_with_budget(
        &mut self,
        budget: u32,
        registers: &mut Registers,
    ) -> CompletionRecord {
        let _timer = Profiler::global().start_event("run_async_with_budget", "vm");

        #[cfg(feature = "trace")]
        if self.vm.trace {
            self.trace_call_frame();
        }

        let mut runtime_budget: u32 = budget;

        loop {
            match self.execute_one(
                |opcode, registers, context| {
                    opcode.spend_budget_and_execute(registers, context, &mut runtime_budget)
                },
                registers,
            ) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(record) => return record,
            }

            if runtime_budget == 0 {
                runtime_budget = budget;
                yield_now().await;
            }
        }
    }

    pub(crate) fn run(&mut self, registers: &mut Registers) -> CompletionRecord {
        let _timer = Profiler::global().start_event("run", "vm");

        #[cfg(feature = "trace")]
        if self.vm.trace {
            self.trace_call_frame();
        }

        loop {
            match self.execute_one(Opcode::execute, registers) {
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

#[derive(Debug, Default, Trace, Finalize)]
pub(crate) struct Registers {
    rp: usize,
    registers: Vec<JsValue>,
}

impl Registers {
    pub(crate) fn new(register_count: usize) -> Self {
        let mut registers = Vec::with_capacity(register_count);
        registers.resize(register_count, JsValue::undefined());

        Self { rp: 0, registers }
    }

    pub(crate) fn push_function(&mut self, register_count: usize) {
        self.rp = self.registers.len();
        self.registers
            .resize(self.rp + register_count, JsValue::undefined());
    }

    #[track_caller]
    pub(crate) fn pop_function(&mut self, register_count: usize) {
        self.registers.truncate(self.rp);
        self.rp -= register_count;
    }

    #[track_caller]
    pub(crate) fn set(&mut self, index: u32, value: JsValue) {
        self.registers[self.rp + index as usize] = value;
    }

    #[track_caller]
    pub(crate) fn get(&self, index: u32) -> &JsValue {
        self.registers
            .get(self.rp + index as usize)
            .expect("registers must be initialized")
    }

    pub(crate) fn clone_current_frame(&self) -> Self {
        Self {
            rp: 0,
            registers: self.registers[self.rp..].to_vec(),
        }
    }
}
