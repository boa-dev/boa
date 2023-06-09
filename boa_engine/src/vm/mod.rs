//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

#[cfg(feature = "fuzz")]
use crate::JsNativeError;
use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    environments::{DeclarativeEnvironment, EnvironmentStack},
    script::Script,
    vm::code_block::Readable,
    Context, JsError, JsObject, JsResult, JsValue, Module,
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
pub use {call_frame::CallFrame, code_block::CodeBlock, opcode::Opcode};

pub(crate) use {
    call_frame::GeneratorResumeKind,
    code_block::{
        create_function_object, create_function_object_fast, create_generator_function_object,
        CodeBlockFlags,
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
    pub(crate) err: Option<JsError>,
    pub(crate) environments: EnvironmentStack,
    #[cfg(feature = "trace")]
    pub(crate) trace: bool,
    pub(crate) runtime_limits: RuntimeLimits,
    pub(crate) active_function: Option<JsObject>,
    pub(crate) active_runnable: Option<ActiveRunnable>,
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
            environments: EnvironmentStack::new(global),
            err: None,
            #[cfg(feature = "trace")]
            trace: false,
            runtime_limits: RuntimeLimits::default(),
            active_function: None,
            active_runnable: None,
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

    pub(crate) fn push_frame(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }

    pub(crate) fn pop_frame(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CompletionType {
    Normal,
    Return,
    Throw,
}

impl dyn Context<'_> + '_ {
    fn execute_instruction(&mut self) -> JsResult<CompletionType> {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");

            let frame = self.as_raw_context_mut().vm.frame_mut();

            let pc = frame.pc;
            let opcode = Opcode::from(frame.code_block.bytecode[pc as usize]);
            frame.pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        opcode.execute(self)
    }

    pub(crate) fn run(&mut self) -> CompletionRecord {
        #[cfg(feature = "trace")]
        const COLUMN_WIDTH: usize = 26;
        #[cfg(feature = "trace")]
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        #[cfg(feature = "trace")]
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        #[cfg(feature = "trace")]
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        #[cfg(feature = "trace")]
        const NUMBER_OF_COLUMNS: usize = 4;

        let _timer = Profiler::global().start_event("run", "vm");

        let raw_context = self.as_raw_context_mut();

        #[cfg(feature = "trace")]
        if raw_context.vm.trace {
            let msg = if raw_context.vm.frames.last().is_some() {
                " Call Frame "
            } else {
                " VM Start "
            };

            println!(
                "{}\n",
                raw_context
                    .vm
                    .frame()
                    .code_block
                    .to_interned_string(raw_context.interner())
            );
            println!(
                "{msg:-^width$}",
                width = COLUMN_WIDTH * NUMBER_OF_COLUMNS - 10
            );
            println!(
                "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {:<OPERAND_COLUMN_WIDTH$} Top Of Stack\n",
                "Time",
                "Opcode",
                "Operands",
            );
        }

        let current_stack_length = raw_context.vm.stack.len();
        raw_context
            .vm
            .frame_mut()
            .set_frame_pointer(current_stack_length as u32);

        // If the current executing function is an async function we have to resolve/reject it's promise at the end.
        // The relevant spec section is 3. in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
        let promise_capability = raw_context.vm.frame().promise_capability.clone();

        let execution_completion = loop {
            let raw_context = self.as_raw_context_mut();

            // 1. Exit the execution loop if program counter ever is equal to or exceeds the amount of instructions
            if raw_context.vm.frame().code_block.bytecode.len()
                <= raw_context.vm.frame().pc as usize
            {
                break CompletionType::Normal;
            }

            #[cfg(feature = "fuzz")]
            {
                if self.instructions_remaining == 0 {
                    let err = JsError::from_native(JsNativeError::no_instructions_remain());
                    raw_context.vm.err = Some(err);
                    break CompletionType::Throw;
                }
                self.instructions_remaining -= 1;
            }

            // 1. Run the next instruction.
            #[cfg(feature = "trace")]
            let result = if raw_context.vm.trace || raw_context.vm.frame().code_block.traceable() {
                let mut pc = raw_context.vm.frame().pc as usize;
                let opcode: Opcode = raw_context
                    .vm
                    .frame()
                    .code_block
                    .read::<u8>(pc)
                    .try_into()
                    .expect("invalid opcode");
                let operands = raw_context
                    .vm
                    .frame()
                    .code_block
                    .instruction_operands(&mut pc, raw_context.interner());

                let instant = Instant::now();
                let result = self.execute_instruction();

                let duration = instant.elapsed();
                println!(
                    "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {}",
                    format!("{}μs", duration.as_micros()),
                    opcode.as_str(),
                    match self.as_raw_context().vm.stack.last() {
                        Some(value) if value.is_callable() => "[function]".to_string(),
                        Some(value) if value.is_object() => "[object]".to_string(),
                        Some(value) => value.display().to_string(),
                        None => "<empty>".to_string(),
                    },
                );

                result
            } else {
                self.execute_instruction()
            };

            #[cfg(not(feature = "trace"))]
            let result = self.execute_instruction();

            let raw_context = self.as_raw_context_mut();

            // 2. Evaluate the result of executing the instruction.
            match result {
                Ok(CompletionType::Normal) => {}
                Ok(CompletionType::Return) => {
                    break CompletionType::Return;
                }
                Ok(CompletionType::Throw) => {
                    break CompletionType::Throw;
                }
                Err(err) => {
                    #[cfg(feature = "fuzz")]
                    {
                        if let Some(native_error) = err.as_native() {
                            // If we hit the execution step limit, bubble up the error to the
                            // (Rust) caller instead of trying to handle as an exception.
                            if native_error.is_no_instructions_remain() {
                                raw_context.err = Some(err);
                                break CompletionType::Throw;
                            }
                        }
                    }

                    if let Some(native_error) = err.as_native() {
                        // If we hit the execution step limit, bubble up the error to the
                        // (Rust) caller instead of trying to handle as an exception.
                        if native_error.is_runtime_limit() {
                            raw_context.vm.err = Some(err);
                            break CompletionType::Throw;
                        }
                    }

                    raw_context.vm.err = Some(err);

                    // If this frame has not evaluated the throw as an AbruptCompletion, then evaluate it
                    let evaluation = Opcode::Throw
                        .execute(self)
                        .expect("Opcode::Throw cannot return Err");

                    if evaluation == CompletionType::Normal {
                        continue;
                    }

                    break CompletionType::Throw;
                }
            }
        };

        let raw_context = self.as_raw_context_mut();

        // Early return immediately after loop.
        if raw_context.vm.frame().r#yield {
            raw_context.vm.frame_mut().r#yield = false;
            let result = raw_context.vm.pop();
            return CompletionRecord::Return(result);
        }

        #[cfg(feature = "trace")]
        if raw_context.vm.trace {
            println!("\nStack:");
            if raw_context.vm.stack.is_empty() {
                println!("    <empty>");
            } else {
                for (i, value) in raw_context.vm.stack.iter().enumerate() {
                    println!(
                        "{i:04}{:<width$} {}",
                        "",
                        if value.is_callable() {
                            "[function]".to_string()
                        } else if value.is_object() {
                            "[object]".to_string()
                        } else {
                            value.display().to_string()
                        },
                        width = COLUMN_WIDTH / 2 - 4,
                    );
                }
            }
            println!("\n");
        }

        let fp = raw_context.vm.frame().fp as usize;

        // Determine the execution result
        let execution_result = if execution_completion == CompletionType::Throw {
            raw_context.vm.frame_mut().abrupt_completion = None;
            raw_context.vm.stack.truncate(fp);
            JsValue::undefined()
        } else if execution_completion == CompletionType::Return {
            raw_context.vm.frame_mut().abrupt_completion = None;
            let result = raw_context.vm.pop();
            raw_context.vm.stack.truncate(fp);
            result
        } else if raw_context.vm.stack.len() <= fp {
            JsValue::undefined()
        } else {
            let result = raw_context.vm.pop();
            raw_context.vm.stack.truncate(fp);
            result
        };

        if let Some(promise) = promise_capability {
            match execution_completion {
                CompletionType::Normal => {
                    promise
                        .resolve()
                        .call(&JsValue::undefined(), &[], self)
                        .expect("cannot fail per spec");
                }
                CompletionType::Return => {
                    promise
                        .resolve()
                        .call(&JsValue::undefined(), &[execution_result.clone()], self)
                        .expect("cannot fail per spec");
                }
                CompletionType::Throw => {
                    let err = raw_context
                        .vm
                        .err
                        .take()
                        .expect("Take must exist on a Throw");
                    promise
                        .reject()
                        .call(&JsValue::undefined(), &[err.to_opaque(self)], self)
                        .expect("cannot fail per spec");
                    self.as_raw_context_mut().vm.err = Some(err);
                }
            }
        } else if let Some(generator_object) = raw_context.vm.frame().async_generator.clone() {
            // Step 3.e-g in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
            let mut generator_object_mut = generator_object.borrow_mut();
            let generator = generator_object_mut
                .as_async_generator_mut()
                .expect("must be async generator");

            generator.state = AsyncGeneratorState::Completed;
            generator.context = None;

            let next = generator
                .queue
                .pop_front()
                .expect("must have item in queue");
            drop(generator_object_mut);

            if execution_completion == CompletionType::Throw {
                AsyncGenerator::complete_step(
                    &next,
                    Err(raw_context
                        .vm
                        .err
                        .take()
                        .expect("err must exist on a Completion::Throw")),
                    true,
                    None,
                    self,
                );
            } else {
                AsyncGenerator::complete_step(&next, Ok(execution_result), true, None, self);
            }
            AsyncGenerator::drain_queue(&generator_object, self);

            return CompletionRecord::Normal(JsValue::undefined());
        }

        // Any valid return statement is re-evaluated as a normal completion vs. return (yield).
        if execution_completion == CompletionType::Throw {
            return CompletionRecord::Throw(
                self.as_raw_context_mut()
                    .vm
                    .err
                    .take()
                    .expect("Err must exist for a CompletionType::Throw"),
            );
        }
        CompletionRecord::Normal(execution_result)
    }
}
