//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

#[cfg(feature = "fuzz")]
use crate::JsNativeError;
use crate::{
    environments::{DeclarativeEnvironment, EnvironmentStack},
    script::Script,
    vm::code_block::Readable,
    Context, JsError, JsNativeErrorKind, JsObject, JsResult, JsValue, Module,
};

use boa_gc::{custom_trace, Finalize, Gc, Trace};
use boa_profiler::Profiler;
use std::mem::size_of;

#[cfg(feature = "trace")]
use boa_interner::ToInternedString;
#[cfg(feature = "trace")]
use std::time::Instant;

mod call_frame;
mod code_block;
mod completion_record;
mod opcode;

mod runtime_limits;

#[cfg(feature = "flowgraph")]
pub mod flowgraph;

pub use runtime_limits::RuntimeLimits;
pub use {
    call_frame::{CallFrame, GeneratorResumeKind},
    code_block::CodeBlock,
    opcode::{Instruction, InstructionIterator, Opcode, VaryingOperand, VaryingOperandKind},
};

pub(crate) use {
    code_block::{
        create_function_object, create_function_object_fast, create_generator_function_object,
        CodeBlockFlags, Handler,
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
    custom_trace!(this, {
        match this {
            Self::Script(script) => mark(script),
            Self::Module(module) => mark(module),
        }
    });
}

impl Vm {
    /// Creates a new virtual machine.
    pub(crate) fn new(global: Gc<DeclarativeEnvironment>) -> Self {
        Self {
            frames: Vec::with_capacity(16),
            stack: Vec::with_capacity(1024),
            return_value: JsValue::undefined(),
            environments: EnvironmentStack::new(global),
            pending_exception: None,
            runtime_limits: RuntimeLimits::default(),
            native_active_function: None,
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
        frame.set_frame_pointer(current_stack_length as u32);
        self.frames.push(frame);
    }

    pub(crate) fn pop_frame(&mut self) -> Option<CallFrame> {
        self.frames.pop()
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
        let sp = frame.fp + handler.stack_count;

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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CompletionType {
    Normal,
    Return,
    Throw,
    Yield,
}

#[cfg(feature = "trace")]
impl Context<'_> {
    const COLUMN_WIDTH: usize = 26;
    const TIME_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH / 2;
    const OPCODE_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const OPERAND_COLUMN_WIDTH: usize = Self::COLUMN_WIDTH;
    const NUMBER_OF_COLUMNS: usize = 4;

    fn trace_call_frame(&self) {
        let msg = if self.vm.frames.last().is_some() {
            format!(
                " Call Frame -- {} ",
                self.vm.frame().code_block().name().to_std_string_escaped()
            )
        } else {
            " VM Start ".to_string()
        };

        println!(
            "{}",
            self.vm
                .frame()
                .code_block
                .to_interned_string(self.interner())
        );
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

    fn trace_execute_instruction(&mut self) -> JsResult<CompletionType> {
        let bytecodes = &self.vm.frame().code_block.bytecode;
        let pc = self.vm.frame().pc as usize;
        let (_, varying_operand_kind, instruction) = InstructionIterator::with_pc(bytecodes, pc)
            .next()
            .expect("There should be an instruction left");
        let operands = self
            .vm
            .frame()
            .code_block
            .instruction_operands(&instruction, self.interner());

        let instant = Instant::now();
        let result = self.execute_instruction();

        let duration = instant.elapsed();

        let stack = {
            let mut stack = String::from("[ ");
            for (i, value) in self.vm.stack.iter().rev().enumerate() {
                match value {
                    value if value.is_callable() => stack.push_str("[function]"),
                    value if value.is_object() => stack.push_str("[object]"),
                    value => stack.push_str(&value.display().to_string()),
                }

                if i + 1 != self.vm.stack.len() {
                    stack.push(',');
                }

                stack.push(' ');
            }

            stack.push(']');
            stack
        };

        let varying_operand_kind = match varying_operand_kind {
            VaryingOperandKind::Short => "",
            VaryingOperandKind::Half => ".Half",
            VaryingOperandKind::Wide => ".Wide",
        };

        println!(
            "{:<TIME_COLUMN_WIDTH$} {}{:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {stack}",
            format!("{}μs", duration.as_micros()),
            instruction.opcode().as_str(),
            varying_operand_kind,
            TIME_COLUMN_WIDTH = Self::TIME_COLUMN_WIDTH,
            OPCODE_COLUMN_WIDTH = Self::OPCODE_COLUMN_WIDTH,
            OPERAND_COLUMN_WIDTH = Self::OPERAND_COLUMN_WIDTH,
        );

        result
    }
}

impl Context<'_> {
    fn execute_instruction(&mut self) -> JsResult<CompletionType> {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");

            let frame = self.vm.frame_mut();

            let pc = frame.pc;
            let opcode = frame.code_block.bytecode[pc as usize].into();
            frame.pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        opcode.execute(self)
    }

    pub(crate) fn run(&mut self) -> CompletionRecord {
        let _timer = Profiler::global().start_event("run", "vm");

        #[cfg(feature = "trace")]
        if self.vm.trace {
            self.trace_call_frame();
        }

        loop {
            #[cfg(feature = "fuzz")]
            {
                if self.instructions_remaining == 0 {
                    return CompletionRecord::Throw(JsError::from_native(
                        JsNativeError::no_instructions_remain(),
                    ));
                }
                self.instructions_remaining -= 1;
            }

            #[cfg(feature = "trace")]
            let result = if self.vm.trace || self.vm.frame().code_block.traceable() {
                self.trace_execute_instruction()
            } else {
                self.execute_instruction()
            };

            #[cfg(not(feature = "trace"))]
            let result = self.execute_instruction();

            match result {
                Ok(CompletionType::Normal) => {}
                Ok(CompletionType::Return) => {
                    self.vm.stack.truncate(self.vm.frame().fp as usize);
                    let execution_result = std::mem::take(&mut self.vm.return_value);
                    return CompletionRecord::Normal(execution_result);
                }
                Ok(CompletionType::Throw) => {
                    self.vm.stack.truncate(self.vm.frame().fp as usize);
                    return CompletionRecord::Throw(
                        self.vm
                            .pending_exception
                            .take()
                            .expect("Err must exist for a CompletionType::Throw"),
                    );
                }
                // Early return immediately.
                Ok(CompletionType::Yield) => {
                    let result = self.vm.pop();
                    return CompletionRecord::Return(result);
                }
                Err(err) => {
                    if let Some(native_error) = err.as_native() {
                        // If we hit the execution step limit, bubble up the error to the
                        // (Rust) caller instead of trying to handle as an exception.
                        match native_error.kind {
                            #[cfg(feature = "fuzz")]
                            JsNativeErrorKind::NoInstructionsRemain => {
                                return CompletionRecord::Throw(err);
                            }
                            JsNativeErrorKind::RuntimeLimit => {
                                self.vm.stack.truncate(self.vm.frame().fp as usize);
                                return CompletionRecord::Throw(err);
                            }
                            _ => {}
                        }
                    }

                    // Note: -1 because we increment after fetching the opcode.
                    let pc = self.vm.frame().pc.saturating_sub(1);
                    if self.vm.handle_exception_at(pc) {
                        self.vm.pending_exception = Some(err);
                        continue;
                    }

                    self.vm.stack.truncate(self.vm.frame().fp as usize);
                    return CompletionRecord::Throw(err);
                }
            }
        }
    }
}
