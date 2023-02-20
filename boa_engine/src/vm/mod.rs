//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    vm::{
        call_frame::AbruptCompletionRecord, code_block::Readable,
        completion_record::CompletionRecord,
    },
    Context, JsError, JsValue,
};
#[cfg(feature = "fuzz")]
use crate::{JsNativeError, JsNativeErrorKind};
use boa_interner::ToInternedString;
use boa_profiler::Profiler;
use std::{convert::TryInto, mem::size_of, time::Instant};

mod call_frame;
mod code_block;
mod completion_record;
mod opcode;

#[cfg(feature = "flowgraph")]
pub mod flowgraph;

pub use {call_frame::CallFrame, code_block::CodeBlock, opcode::Opcode};

pub(crate) use {
    call_frame::GeneratorResumeKind,
    code_block::{create_function_object, create_generator_function_object},
    opcode::BindingOpcode,
};

#[cfg(test)]
mod tests;
/// Virtual Machine.
#[derive(Debug)]
pub struct Vm {
    pub(crate) frames: Vec<CallFrame>,
    pub(crate) stack: Vec<JsValue>,
    pub(crate) trace: bool,
    pub(crate) stack_size_limit: usize,
}

impl Default for Vm {
    fn default() -> Self {
        Self {
            frames: Vec::with_capacity(16),
            stack: Vec::with_capacity(1024),
            trace: false,
            stack_size_limit: 1024,
        }
    }
}

impl Vm {
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
        let value = self.frame().code_block.read::<T>(self.frame().pc);
        self.frame_mut().pc += size_of::<T>();
        value
    }

    /// Retrieves the VM frame
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    pub(crate) fn frame(&self) -> &CallFrame {
        self.frames.last().expect("no frame found")
    }

    /// Retrieves the VM frame mutably
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
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

macro_rules! ok_or_throw_completion {
    ( $e:expr, $context:ident ) => {
        match $e {
            Ok(value) => value,
            Err(value) => {
                let error = value.to_opaque($context);
                $context.vm.push(error);
                return CompletionType::Throw;
            }
        }
    };
}

macro_rules! throw_completion {
    ($error:expr, $err_type:ty, $context:ident ) => {{
        let err: $err_type = $error;
        let value = err.to_opaque($context);
        $context.vm.push(value);
        return CompletionType::Throw;
    }};
}

pub(crate) use {ok_or_throw_completion, throw_completion};

impl Context<'_> {
    fn execute_instruction(&mut self) -> CompletionType {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");
            let opcode = self.vm.frame().code_block.bytecode[self.vm.frame().pc]
                .try_into()
                .expect("could not convert code at PC to opcode");
            self.vm.frame_mut().pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        opcode.execute(self)
    }

    pub(crate) fn run(&mut self) -> CompletionRecord {
        const COLUMN_WIDTH: usize = 26;
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const NUMBER_OF_COLUMNS: usize = 4;

        let _timer = Profiler::global().start_event("run", "vm");

        if self.vm.trace {
            let msg = if self.vm.frames.last().is_some() {
                " Call Frame "
            } else {
                " VM Start "
            };

            println!(
                "{}\n",
                self.vm
                    .frame()
                    .code_block
                    .to_interned_string(self.interner())
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

        let stack_len = self.vm.stack.len();
        self.vm.frame_mut().set_frame_pointer(stack_len);

        let execution_completion = loop {
            #[cfg(feature = "fuzz")]
            {
                if self.instructions_remaining == 0 {
                    return Err(JsError::from_native(JsNativeError::no_instructions_remain()));
                }
                self.instructions_remaining -= 1;
            }

            // 1. Run the next instruction.
            let result = if self.vm.trace {
                let mut pc = self.vm.frame().pc;
                let opcode: Opcode = self
                    .vm
                    .frame()
                    .code_block
                    .read::<u8>(pc)
                    .try_into()
                    .expect("invalid opcode");
                let operands = self
                    .vm
                    .frame()
                    .code_block
                    .instruction_operands(&mut pc, self.interner());

                let instant = Instant::now();
                let result = self.execute_instruction();
                let duration = instant.elapsed();

                println!(
                    "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {}",
                    format!("{}μs", duration.as_micros()),
                    opcode.as_str(),
                    match self.vm.stack.last() {
                        Some(value) if value.is_callable() => "[function]".to_string(),
                        Some(value) if value.is_object() => "[object]".to_string(),
                        Some(value) => value.display().to_string(),
                        None => "<empty>".to_string(),
                    }
                );

                result
            } else {
                self.execute_instruction()
            };

            // 2. Evaluate the result of executing the instruction.
            match result {
                CompletionType::Normal => {},
                CompletionType::Return => {
                    break CompletionType::Return;
                }
                CompletionType::Throw => {
                    // TODO: Adapt the below fuzz for the new execution loop
                    let error = self.vm.pop();
                    #[cfg(feature = "fuzz")]
                    {
                        let throw = if let Some(native_error) =
                            JsError::from_opaque(error).as_native()
                        {
                            // If we hit the execution step limit, bubble up the error to the
                            // (Rust) caller instead of trying to handle as an exception.
                            if matches!(native_error.kind, JsNativeErrorKind::NoInstructionsRemain)
                            {
                                self.vm.push(error);
                                return CompletionType::Throw;
                            }
                        };
                    }

                    // 1. Find the viable catch and finally blocks
                    let current_address = self.vm.frame().pc;
                    let viable_catch_candidates =
                        self.vm.frame().env_stack.iter().filter(|env| {
                            env.is_try_env() && env.start_address() < env.exit_address()
                        });

                    if let Some(candidate) = viable_catch_candidates.last() {
                        let catch_target = candidate.start_address();

                        let mut env_to_pop = 0;
                        let mut target_address = u32::MAX;
                        while self.vm.frame().env_stack.len() > 1 {
                            let env_entry = self
                                .vm
                                .frame_mut()
                                .env_stack
                                .last()
                                .expect("EnvStackEntries must exist");

                            if env_entry.is_try_env()
                                && env_entry.start_address() < env_entry.exit_address()
                            {
                                target_address = env_entry.start_address();
                                env_to_pop += env_entry.env_num();
                                self.vm.frame_mut().env_stack.pop();
                                break;
                            } else if env_entry.is_finally_env() {
                                if current_address > (env_entry.start_address() as usize) {
                                    target_address = env_entry.exit_address();
                                } else {
                                    target_address = env_entry.start_address();
                                }
                                break;
                            }
                            env_to_pop += env_entry.env_num();
                            self.vm.frame_mut().env_stack.pop();
                        }

                        let env_truncation_len =
                            self.realm.environments.len().saturating_sub(env_to_pop);
                        self.realm.environments.truncate(env_truncation_len);

                        if target_address == catch_target {
                            self.vm.frame_mut().pc = catch_target as usize;
                        } else {
                            self.vm.frame_mut().pc = target_address as usize;
                        };

                        for _ in 0..self.vm.frame().pop_on_return {
                            self.vm.pop();
                        }

                        self.vm.frame_mut().pop_on_return = 0;
                        let record =
                            AbruptCompletionRecord::new_throw().with_initial_target(catch_target);
                        self.vm.frame_mut().abrupt_completion = Some(record);
                        self.vm.push(error);
                        continue;
                    }

                    let mut env_to_pop = 0;
                    let mut target_address = None;
                    let mut env_stack_to_pop = 0;
                    for env_entry in self.vm.frame_mut().env_stack.iter_mut().rev() {
                        if env_entry.is_finally_env() {
                            if (env_entry.start_address() as usize) < current_address {
                                target_address = Some(env_entry.exit_address() as usize);
                            } else {
                                target_address = Some(env_entry.start_address() as usize);
                            }
                            break;
                        };

                        env_to_pop += env_entry.env_num();
                        if env_entry.is_global_env() {
                            env_entry.clear_env_num();
                            break;
                        };

                        env_stack_to_pop += 1;
                    }

                    if let Some(address) = target_address {
                        for _ in 0..env_stack_to_pop {
                            self.vm.frame_mut().env_stack.pop();
                        }

                        let env_truncation_len =
                            self.realm.environments.len().saturating_sub(env_to_pop);
                        self.realm.environments.truncate(env_truncation_len);

                        let previous_stack_size = self
                            .vm
                            .stack
                            .len()
                            .saturating_sub(self.vm.frame().pop_on_return);
                        self.vm.stack.truncate(previous_stack_size);
                        self.vm.frame_mut().pop_on_return = 0;

                        let record = AbruptCompletionRecord::new_throw();
                        self.vm.frame_mut().abrupt_completion = Some(record);
                        self.vm.frame_mut().pc = address;
                        self.vm.push(error);
                        continue;
                    }

                    self.vm.push(error);
                    break CompletionType::Throw;
                }
            }

            // 3. Exit the execution loop if program counter ever is equal to or exceeds the amount of instructions
            if self.vm.frame().code_block.bytecode.len() <= self.vm.frame().pc {
                break CompletionType::Normal;
            }
        };

        if self.vm.trace {
            println!("\nStack:");
            if self.vm.stack.is_empty() {
                println!("    <empty>");
            } else {
                for (i, value) in self.vm.stack.iter().enumerate() {
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

        let execution_result = if execution_completion == CompletionType::Return {
            let result = self.vm.pop();
            self.vm.stack.truncate(self.vm.frame().fp);
            result
        } else if self.vm.stack.len() <= self.vm.frame().fp {
            JsValue::undefined()
        } else {
            let result = self.vm.pop();
            self.vm.stack.truncate(self.vm.frame().fp);
            result
        };

        if let Some(generator_object) = self.vm.frame().async_generator.clone() {
            let mut generator_object_mut = generator_object.borrow_mut();
            let generator = generator_object_mut
                .as_async_generator_mut()
                .expect("must be async generator");

            generator.state = AsyncGeneratorState::Completed;

            let next = generator
                .queue
                .pop_front()
                .expect("must have item in queue");
            drop(generator_object_mut);

            if execution_completion == CompletionType::Normal
                && self.vm.frame().abrupt_completion.is_none()
            {
                AsyncGenerator::complete_step(&next, Ok(JsValue::undefined()), true, self);
            } else if execution_completion == CompletionType::Normal {
                AsyncGenerator::complete_step(&next, Ok(execution_result), true, self);
            } else if execution_completion == CompletionType::Throw {
                AsyncGenerator::complete_step(
                    &next,
                    Err(JsError::from_opaque(execution_result)),
                    true,
                    self,
                );
            }
            AsyncGenerator::drain_queue(&generator_object, self);

            return CompletionRecord::new(CompletionType::Normal, JsValue::undefined());
        }

        // Any valid return statement is re-evaluated as a normal completion vs. return (yield).
        if execution_completion == CompletionType::Return
            && self.vm.frame().abrupt_completion.is_some()
        {
            return CompletionRecord::new(CompletionType::Normal, execution_result);
        }
        CompletionRecord::new(execution_completion, execution_result)
    }
}
