//! Boa's ECMAScript Virtual Machine
//!
//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    vm::{call_frame::AbruptCompletionRecord, code_block::Readable},
    Context, JsResult, JsValue,
};
#[cfg(feature = "fuzz")]
use crate::{JsError, JsNativeError, JsNativeErrorKind};
use boa_interner::ToInternedString;
use boa_profiler::Profiler;
use std::{convert::TryInto, mem::size_of, time::Instant};

mod call_frame;
mod code_block;
mod opcode;

#[cfg(feature = "flowgraph")]
pub mod flowgraph;

pub use {call_frame::CallFrame, code_block::CodeBlock, opcode::Opcode};

pub(crate) use {
    call_frame::{FinallyReturn, GeneratorResumeKind},
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

/// Indicates if the execution should continue, exit or yield.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ShouldExit {
    True,
    False,
    Yield,
    Await,
}

/// Indicates if the execution of a codeblock has ended normally or has been yielded.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ReturnType {
    Normal,
    Yield,
}

impl Context<'_> {
    fn execute_instruction(&mut self) -> JsResult<ShouldExit> {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");
            let opcode = self.vm.frame().code_block.bytecode[self.vm.frame().pc]
                .try_into()
                .expect("could not convert code at PC to opcode");
            self.vm.frame_mut().pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        let result = opcode.execute(self)?;

        Ok(result)
    }

    pub(crate) fn run(&mut self) -> JsResult<(JsValue, ReturnType)> {
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

        let start_stack_size = self.vm.stack.len();

        // If the current executing function is an async function we have to resolve/reject it's promise at the end.
        // The relevant spec section is 3. in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
        let promise_capability = self
            .realm
            .environments
            .current_function_slots()
            .as_function_slots()
            .and_then(|slots| {
                let slots_borrow = slots.borrow();
                let function_object = slots_borrow.function_object();
                let function = function_object.borrow();
                function
                    .as_function()
                    .and_then(|f| f.get_promise_capability().cloned())
            });

        while self.vm.frame().pc < self.vm.frame().code_block.bytecode.len() {
            #[cfg(feature = "fuzz")]
            {
                if self.instructions_remaining == 0 {
                    return Err(JsError::from_native(JsNativeError::no_instructions_remain()));
                }
                self.instructions_remaining -= 1;
            }

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

            match result {
                Ok(ShouldExit::True) => {
                    let result = self.vm.pop();
                    self.vm.stack.truncate(start_stack_size);

                    // Step 3.e in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
                    if let Some(promise_capability) = promise_capability {
                        promise_capability
                            .resolve()
                            .call(&JsValue::undefined(), &[result.clone()], self)
                            .expect("cannot fail per spec");
                    }
                    // Step 4.e-j in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
                    else if let Some(generator_object) = self.vm.frame().async_generator.clone() {
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
                        AsyncGenerator::complete_step(&next, Ok(result), true, self);
                        AsyncGenerator::drain_queue(&generator_object, self);
                        return Ok((JsValue::undefined(), ReturnType::Normal));
                    }

                    return Ok((result, ReturnType::Normal));
                }
                Ok(ShouldExit::Await) => {
                    let result = self.vm.pop();
                    self.vm.stack.truncate(start_stack_size);
                    return Ok((result, ReturnType::Normal));
                }
                Ok(ShouldExit::False) => {}
                Ok(ShouldExit::Yield) => {
                    let result = self.vm.stack.pop().unwrap_or(JsValue::Undefined);
                    return Ok((result, ReturnType::Yield));
                }
                Err(e) => {
                    #[cfg(feature = "fuzz")]
                    if let Some(native_error) = e.as_native() {
                        // If we hit the execution step limit, bubble up the error to the
                        // (Rust) caller instead of trying to handle as an exception.
                        if matches!(native_error.kind, JsNativeErrorKind::NoInstructionsRemain) {
                            return Err(e);
                        }
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
                        let record = AbruptCompletionRecord::create_throw_completion()
                            .with_initial_target(catch_target);
                        self.vm.frame_mut().abrupt_completion = Some(record);
                        self.vm.frame_mut().finally_return = FinallyReturn::Err;
                        let err = e.to_opaque(self);
                        self.vm.push(err);
                    } else {
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

                            let record = AbruptCompletionRecord::create_throw_completion();
                            self.vm.frame_mut().abrupt_completion = Some(record);
                            self.vm.frame_mut().pc = address;
                            self.vm.frame_mut().finally_return = FinallyReturn::Err;
                            let err = e.to_opaque(self);
                            self.vm.push(err);
                        } else {
                            self.vm.stack.truncate(start_stack_size);

                            // Step 3.f in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
                            if let Some(promise_capability) = promise_capability {
                                let e = e.to_opaque(self);
                                promise_capability
                                    .reject()
                                    .call(&JsValue::undefined(), &[e.clone()], self)
                                    .expect("cannot fail per spec");

                                return Ok((e, ReturnType::Normal));
                            }
                            // Step 4.e-j in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
                            else if let Some(generator_object) =
                                self.vm.frame().async_generator.clone()
                            {
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
                                AsyncGenerator::complete_step(&next, Err(e), true, self);
                                AsyncGenerator::drain_queue(&generator_object, self);
                                return Ok((JsValue::undefined(), ReturnType::Normal));
                            }

                            return Err(e);
                        }
                    }
                }
            }
        }

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

        if self.vm.stack.len() <= start_stack_size {
            // Step 3.d in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
            if let Some(promise_capability) = promise_capability {
                promise_capability
                    .resolve()
                    .call(&JsValue::undefined(), &[], self)
                    .expect("cannot fail per spec");
            }
            // Step 4.e-j in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
            else if let Some(generator_object) = self.vm.frame().async_generator.clone() {
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
                AsyncGenerator::complete_step(&next, Ok(JsValue::undefined()), true, self);
                AsyncGenerator::drain_queue(&generator_object, self);
            }

            return Ok((JsValue::undefined(), ReturnType::Normal));
        }

        let result = self.vm.pop();
        self.vm.stack.truncate(start_stack_size);

        // Step 3.d in [AsyncBlockStart](https://tc39.es/ecma262/#sec-asyncblockstart).
        if let Some(promise_capability) = promise_capability {
            promise_capability
                .resolve()
                .call(&JsValue::undefined(), &[result.clone()], self)
                .expect("cannot fail per spec");
        }
        // Step 4.e-j in [AsyncGeneratorStart](https://tc39.es/ecma262/#sec-asyncgeneratorstart)
        else if let Some(generator_object) = self.vm.frame().async_generator.clone() {
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
            AsyncGenerator::complete_step(&next, Ok(result), true, self);
            AsyncGenerator::drain_queue(&generator_object, self);
            return Ok((JsValue::undefined(), ReturnType::Normal));
        }

        Ok((result, ReturnType::Normal))
    }
}
