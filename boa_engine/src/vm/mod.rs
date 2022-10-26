//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::async_generator::{AsyncGenerator, AsyncGeneratorState},
    vm::{call_frame::CatchAddresses, code_block::Readable},
    Context, JsResult, JsValue,
};
use boa_interner::ToInternedString;
use boa_profiler::Profiler;
use std::{convert::TryInto, mem::size_of, time::Instant};

mod call_frame;
mod code_block;
mod opcode;

#[allow(clippy::wildcard_imports)]
use opcode::*;

pub use {call_frame::CallFrame, code_block::CodeBlock, opcode::Opcode};

pub(crate) use {
    call_frame::{FinallyReturn, GeneratorResumeKind, TryStackEntry},
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

impl Vm {
    /// Push a value on the stack.
    #[inline]
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
    #[inline]
    #[track_caller]
    pub(crate) fn pop(&mut self) -> JsValue {
        self.stack.pop().expect("stack was empty")
    }

    #[track_caller]
    #[inline]
    pub(crate) fn read<T: Readable>(&mut self) -> T {
        let value = self.frame().code.read::<T>(self.frame().pc);
        self.frame_mut().pc += size_of::<T>();
        value
    }

    /// Retrieves the VM frame
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    #[inline]
    pub(crate) fn frame(&self) -> &CallFrame {
        self.frames.last().expect("no frame found")
    }

    /// Retrieves the VM frame mutably
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    #[inline]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("no frame found")
    }

    #[inline]
    pub(crate) fn push_frame(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }

    #[inline]
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

impl Context {
    fn execute_instruction(&mut self) -> JsResult<ShouldExit> {
        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");
            let opcode = self.vm.frame().code.code[self.vm.frame().pc]
                .try_into()
                .expect("could not convert code at PC to opcode");
            self.vm.frame_mut().pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(opcode.as_instruction_str(), "vm");

        let result = match opcode {
            Opcode::Nop => Nop::execute(self)?,
            Opcode::Pop => Pop::execute(self)?,
            Opcode::PopIfThrown => PopIfThrown::execute(self)?,
            Opcode::Dup => Dup::execute(self)?,
            Opcode::Swap => Swap::execute(self)?,
            Opcode::PushUndefined => PushUndefined::execute(self)?,
            Opcode::PushNull => PushNull::execute(self)?,
            Opcode::PushTrue => PushTrue::execute(self)?,
            Opcode::PushFalse => PushFalse::execute(self)?,
            Opcode::PushZero => PushZero::execute(self)?,
            Opcode::PushOne => PushOne::execute(self)?,
            Opcode::PushInt8 => PushInt8::execute(self)?,
            Opcode::PushInt16 => PushInt16::execute(self)?,
            Opcode::PushInt32 => PushInt32::execute(self)?,
            Opcode::PushRational => PushRational::execute(self)?,
            Opcode::PushNaN => PushNaN::execute(self)?,
            Opcode::PushPositiveInfinity => PushPositiveInfinity::execute(self)?,
            Opcode::PushNegativeInfinity => PushNegativeInfinity::execute(self)?,
            Opcode::PushLiteral => PushLiteral::execute(self)?,
            Opcode::PushEmptyObject => PushEmptyObject::execute(self)?,
            Opcode::PushClassPrototype => PushClassPrototype::execute(self)?,
            Opcode::SetClassPrototype => SetClassPrototype::execute(self)?,
            Opcode::SetHomeObject => SetHomeObject::execute(self)?,
            Opcode::PushNewArray => PushNewArray::execute(self)?,
            Opcode::PushValueToArray => PushValueToArray::execute(self)?,
            Opcode::PushElisionToArray => PushElisionToArray::execute(self)?,
            Opcode::PushIteratorToArray => PushIteratorToArray::execute(self)?,
            Opcode::Add => Add::execute(self)?,
            Opcode::Sub => Sub::execute(self)?,
            Opcode::Mul => Mul::execute(self)?,
            Opcode::Div => Div::execute(self)?,
            Opcode::Pow => Pow::execute(self)?,
            Opcode::Mod => Mod::execute(self)?,
            Opcode::BitAnd => BitAnd::execute(self)?,
            Opcode::BitOr => BitOr::execute(self)?,
            Opcode::BitXor => BitXor::execute(self)?,
            Opcode::ShiftLeft => ShiftLeft::execute(self)?,
            Opcode::ShiftRight => ShiftRight::execute(self)?,
            Opcode::UnsignedShiftRight => UnsignedShiftRight::execute(self)?,
            Opcode::Eq => Eq::execute(self)?,
            Opcode::NotEq => NotEq::execute(self)?,
            Opcode::StrictEq => StrictEq::execute(self)?,
            Opcode::StrictNotEq => StrictNotEq::execute(self)?,
            Opcode::GreaterThan => GreaterThan::execute(self)?,
            Opcode::GreaterThanOrEq => GreaterThanOrEq::execute(self)?,
            Opcode::LessThan => LessThan::execute(self)?,
            Opcode::LessThanOrEq => LessThanOrEq::execute(self)?,
            Opcode::In => In::execute(self)?,
            Opcode::InstanceOf => InstanceOf::execute(self)?,
            Opcode::Void => Void::execute(self)?,
            Opcode::TypeOf => TypeOf::execute(self)?,
            Opcode::Pos => Pos::execute(self)?,
            Opcode::Neg => Neg::execute(self)?,
            Opcode::Inc => Inc::execute(self)?,
            Opcode::IncPost => IncPost::execute(self)?,
            Opcode::Dec => Dec::execute(self)?,
            Opcode::DecPost => DecPost::execute(self)?,
            Opcode::LogicalNot => LogicalNot::execute(self)?,
            Opcode::BitNot => BitNot::execute(self)?,
            Opcode::DefVar => DefVar::execute(self)?,
            Opcode::DefInitVar => DefInitVar::execute(self)?,
            Opcode::DefLet => DefLet::execute(self)?,
            Opcode::DefInitLet => DefInitLet::execute(self)?,
            Opcode::DefInitConst => DefInitConst::execute(self)?,
            Opcode::DefInitArg => DefInitArg::execute(self)?,
            Opcode::GetName => GetName::execute(self)?,
            Opcode::GetNameOrUndefined => GetNameOrUndefined::execute(self)?,
            Opcode::SetName => SetName::execute(self)?,
            Opcode::Jump => Jump::execute(self)?,
            Opcode::JumpIfFalse => JumpIfFalse::execute(self)?,
            Opcode::JumpIfNotUndefined => JumpIfNotUndefined::execute(self)?,
            Opcode::LogicalAnd => LogicalAnd::execute(self)?,
            Opcode::LogicalOr => LogicalOr::execute(self)?,
            Opcode::Coalesce => Coalesce::execute(self)?,
            Opcode::ToBoolean => ToBoolean::execute(self)?,
            Opcode::GetPropertyByName => GetPropertyByName::execute(self)?,
            Opcode::GetPropertyByValue => GetPropertyByValue::execute(self)?,
            Opcode::GetPropertyByValuePush => GetPropertyByValuePush::execute(self)?,
            Opcode::SetPropertyByName => SetPropertyByName::execute(self)?,
            Opcode::DefineOwnPropertyByName => DefineOwnPropertyByName::execute(self)?,
            Opcode::DefineClassMethodByName => DefineClassMethodByName::execute(self)?,
            Opcode::SetPropertyByValue => SetPropertyByValue::execute(self)?,
            Opcode::DefineOwnPropertyByValue => DefineOwnPropertyByValue::execute(self)?,
            Opcode::DefineClassMethodByValue => DefineClassMethodByValue::execute(self)?,
            Opcode::SetPropertyGetterByName => SetPropertyGetterByName::execute(self)?,
            Opcode::DefineClassGetterByName => DefineClassGetterByName::execute(self)?,
            Opcode::SetPropertyGetterByValue => SetPropertyGetterByValue::execute(self)?,
            Opcode::DefineClassGetterByValue => DefineClassGetterByValue::execute(self)?,
            Opcode::SetPropertySetterByName => SetPropertySetterByName::execute(self)?,
            Opcode::DefineClassSetterByName => DefineClassSetterByName::execute(self)?,
            Opcode::SetPropertySetterByValue => SetPropertySetterByValue::execute(self)?,
            Opcode::DefineClassSetterByValue => DefineClassSetterByValue::execute(self)?,
            Opcode::AssignPrivateField => AssignPrivateField::execute(self)?,
            Opcode::SetPrivateField => SetPrivateField::execute(self)?,
            Opcode::SetPrivateMethod => SetPrivateMethod::execute(self)?,
            Opcode::SetPrivateSetter => SetPrivateSetter::execute(self)?,
            Opcode::SetPrivateGetter => SetPrivateGetter::execute(self)?,
            Opcode::GetPrivateField => GetPrivateField::execute(self)?,
            Opcode::PushClassField => PushClassField::execute(self)?,
            Opcode::PushClassFieldPrivate => PushClassFieldPrivate::execute(self)?,
            Opcode::PushClassPrivateGetter => PushClassPrivateGetter::execute(self)?,
            Opcode::PushClassPrivateSetter => PushClassPrivateSetter::execute(self)?,
            Opcode::PushClassPrivateMethod => PushClassPrivateMethod::execute(self)?,
            Opcode::DeletePropertyByName => DeletePropertyByName::execute(self)?,
            Opcode::DeletePropertyByValue => DeletePropertyByValue::execute(self)?,
            Opcode::CopyDataProperties => CopyDataProperties::execute(self)?,
            Opcode::ToPropertyKey => ToPropertyKey::execute(self)?,
            Opcode::Throw => Throw::execute(self)?,
            Opcode::TryStart => TryStart::execute(self)?,
            Opcode::TryEnd => TryEnd::execute(self)?,
            Opcode::CatchEnd => CatchEnd::execute(self)?,
            Opcode::CatchStart => CatchStart::execute(self)?,
            Opcode::CatchEnd2 => CatchEnd2::execute(self)?,
            Opcode::FinallyStart => FinallyStart::execute(self)?,
            Opcode::FinallyEnd => FinallyEnd::execute(self)?,
            Opcode::FinallySetJump => FinallySetJump::execute(self)?,
            Opcode::This => This::execute(self)?,
            Opcode::Super => Super::execute(self)?,
            Opcode::SuperCall => SuperCall::execute(self)?,
            Opcode::SuperCallSpread => SuperCallSpread::execute(self)?,
            Opcode::SuperCallDerived => SuperCallDerived::execute(self)?,
            Opcode::Case => Case::execute(self)?,
            Opcode::Default => Default::execute(self)?,
            Opcode::GetArrowFunction => GetArrowFunction::execute(self)?,
            Opcode::GetFunction => GetFunction::execute(self)?,
            Opcode::GetFunctionAsync => GetFunctionAsync::execute(self)?,
            Opcode::GetGenerator => GetGenerator::execute(self)?,
            Opcode::GetGeneratorAsync => GetGeneratorAsync::execute(self)?,
            Opcode::CallEval => CallEval::execute(self)?,
            Opcode::CallEvalSpread => CallEvalSpread::execute(self)?,
            Opcode::Call => Call::execute(self)?,
            Opcode::CallSpread => CallSpread::execute(self)?,
            Opcode::New => New::execute(self)?,
            Opcode::NewSpread => NewSpread::execute(self)?,
            Opcode::Return => Return::execute(self)?,
            Opcode::PushDeclarativeEnvironment => PushDeclarativeEnvironment::execute(self)?,
            Opcode::PushFunctionEnvironment => PushFunctionEnvironment::execute(self)?,
            Opcode::PopEnvironment => PopEnvironment::execute(self)?,
            Opcode::LoopStart => LoopStart::execute(self)?,
            Opcode::LoopContinue => LoopContinue::execute(self)?,
            Opcode::LoopEnd => LoopEnd::execute(self)?,
            Opcode::ForInLoopInitIterator => ForInLoopInitIterator::execute(self)?,
            Opcode::InitIterator => InitIterator::execute(self)?,
            Opcode::InitIteratorAsync => InitIteratorAsync::execute(self)?,
            Opcode::IteratorNext => IteratorNext::execute(self)?,
            Opcode::IteratorClose => IteratorClose::execute(self)?,
            Opcode::IteratorToArray => IteratorToArray::execute(self)?,
            Opcode::ForInLoopNext => ForInLoopNext::execute(self)?,
            Opcode::ForAwaitOfLoopIterate => ForAwaitOfLoopIterate::execute(self)?,
            Opcode::ForAwaitOfLoopNext => ForAwaitOfLoopNext::execute(self)?,
            Opcode::ConcatToString => ConcatToString::execute(self)?,
            Opcode::RequireObjectCoercible => RequireObjectCoercible::execute(self)?,
            Opcode::ValueNotNullOrUndefined => ValueNotNullOrUndefined::execute(self)?,
            Opcode::RestParameterInit => RestParameterInit::execute(self)?,
            Opcode::RestParameterPop => RestParameterPop::execute(self)?,
            Opcode::PopOnReturnAdd => PopOnReturnAdd::execute(self)?,
            Opcode::PopOnReturnSub => PopOnReturnSub::execute(self)?,
            Opcode::Yield => Yield::execute(self)?,
            Opcode::GeneratorNext => GeneratorNext::execute(self)?,
            Opcode::AsyncGeneratorNext => AsyncGeneratorNext::execute(self)?,
            Opcode::GeneratorNextDelegate => GeneratorNextDelegate::execute(self)?,
            Opcode::Await => Await::execute(self)?,
            Opcode::PushNewTarget => PushNewTarget::execute(self)?,
        };

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
                self.vm.frame().code.to_interned_string(self.interner())
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
            .get_this_environment()
            .as_function_slots()
            .and_then(|slots| {
                let slots_borrow = slots.borrow();
                let function_object = slots_borrow.function_object();
                let function = function_object.borrow();
                function
                    .as_function()
                    .and_then(|f| f.get_promise_capability().cloned())
            });

        while self.vm.frame().pc < self.vm.frame().code.code.len() {
            let result = if self.vm.trace {
                let mut pc = self.vm.frame().pc;
                let opcode: Opcode = self
                    .vm
                    .frame()
                    .code
                    .read::<u8>(pc)
                    .try_into()
                    .expect("invalid opcode");
                let operands = self
                    .vm
                    .frame()
                    .code
                    .instruction_operands(&mut pc, self.interner());

                let instant = Instant::now();
                let result = self.execute_instruction();
                let duration = instant.elapsed();

                println!(
                    "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {}",
                    format!("{}μs", duration.as_micros()),
                    opcode.as_str(),
                    match self.vm.stack.last() {
                        None => "<empty>".to_string(),
                        Some(value) => {
                            if value.is_callable() {
                                "[function]".to_string()
                            } else if value.is_object() {
                                "[object]".to_string()
                            } else {
                                value.display().to_string()
                            }
                        }
                    },
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
                    if let Some(address) = self.vm.frame().catch.last() {
                        let address = address.next;
                        let try_stack_entry = self
                            .vm
                            .frame_mut()
                            .try_env_stack
                            .last_mut()
                            .expect("must exist");
                        let try_stack_entry_copy = *try_stack_entry;
                        try_stack_entry.num_env = 0;
                        try_stack_entry.num_loop_stack_entries = 0;
                        for _ in 0..try_stack_entry_copy.num_env {
                            self.realm.environments.pop();
                        }
                        let mut num_env = try_stack_entry_copy.num_env;
                        for _ in 0..try_stack_entry_copy.num_loop_stack_entries {
                            num_env -= self
                                .vm
                                .frame_mut()
                                .loop_env_stack
                                .pop()
                                .expect("must exist");
                        }
                        *self
                            .vm
                            .frame_mut()
                            .loop_env_stack
                            .last_mut()
                            .expect("must exist") -= num_env;
                        self.vm.frame_mut().try_env_stack.pop().expect("must exist");
                        for _ in 0..self.vm.frame().pop_on_return {
                            self.vm.pop();
                        }
                        self.vm.frame_mut().pop_on_return = 0;
                        self.vm.frame_mut().pc = address as usize;
                        self.vm.frame_mut().catch.pop();
                        self.vm.frame_mut().finally_return = FinallyReturn::Err;
                        self.vm.frame_mut().thrown = true;
                        let e = e.to_opaque(self);
                        self.vm.push(e);
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
