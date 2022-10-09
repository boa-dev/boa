//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        function::{ConstructorKind, Function},
        iterable::{IteratorHint, IteratorRecord, IteratorResult},
        Array, ForInIterator, JsArgs, Number, Promise,
    },
    environments::EnvironmentSlots,
    error::JsNativeError,
    object::{FunctionBuilder, JsFunction, JsObject, ObjectData, PrivateElement},
    property::{DescriptorKind, PropertyDescriptor, PropertyDescriptorBuilder, PropertyKey},
    value::Numeric,
    vm::{
        call_frame::CatchAddresses,
        code_block::{initialize_instance_elements, Readable},
    },
    Context, JsBigInt, JsError, JsResult, JsString, JsValue,
};
use boa_interner::ToInternedString;
use boa_profiler::Profiler;
use std::{convert::TryInto, mem::size_of, time::Instant};

mod call_frame;
mod code_block;
mod opcode;

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
enum ShouldExit {
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
            Opcode::Nop(op) => Nop::execute(self)?,
            Opcode::Pop(op) => Pop::execute(self)?,
            Opcode::PopIfThrown(op) => PopIfThrown::execute(self)?,
            Opcode::Dup(op) => Dup::execute(self)?,
            Opcode::Swap(op) => Swap::execute(self)?,
            Opcode::PushUndefined(op) => PushUndefined::execute(self)?,
            Opcode::PushNull(op) => PushNull::execute(self)?,
            Opcode::PushTrue(op) => PushTrue::execute(self)?,
            Opcode::PushFalse(op) => PushFalse::execute(self)?,
            Opcode::PushZero(op) => PushZero::execute(self)?,
            Opcode::PushOne(op) => PushOne::execute(self)?,
            Opcode::PushInt8(op) => PushInt8::execute(self)?,
            Opcode::PushInt16(op) => PushInt16::execute(self)?,
            Opcode::PushInt32(op) => PushInt32::execute(self)?,
            Opcode::PushRational(op) => PushRational::execute(self)?,
            Opcode::PushNaN(op) => PushNaN::execute(self)?,
            Opcode::PushPositiveInfinity(op) => PushPositiveInfinity::execute(self)?,
            Opcode::PushNegativeInfinity(op) => PushNegativeInfinity::execute(self)?,
            Opcode::PushLiteral(op) => PushLiteral::execute(self)?,
            Opcode::PushEmptyObject(op) => PushEmptyObject::execute(self)?,
            Opcode::PushClassPrototype(op) => PushClassPrototype::execute(self)?,
            Opcode::SetClassPrototype(op) => SetClassPrototype::execute(self)?,
            Opcode::SetHomeObject(op) => SetHomeObject::execute(self)?,
            Opcode::PushNewArray(op) => PushNewArray::execute(self)?,
            Opcode::PushValueToArray(op) => PushValueToArray::execute(self)?,
            Opcode::PushElisionToArray(op) => PushElisionToArray::execute(self)?,
            Opcode::PushIteratorToArray(op) => PushIteratorToArray::execute(self)?,
            Opcode::Add(op) => Add::execute(self)?,
            Opcode::Sub(op) => Sub::execute(self)?,
            Opcode::Mul(op) => Mul::execute(self)?,
            Opcode::Div(op) => Div::execute(self)?,
            Opcode::Pow(op) => Pow::execute(self)?,
            Opcode::Mod(op) => Mod::execute(self)?,
            Opcode::BitAnd(op) => BitAnd::execute(self)?,
            Opcode::BitOr(op) => BitOr::execute(self)?,
            Opcode::BitXor(op) => BitXor::execute(self)?,
            Opcode::ShiftLeft(op) => ShiftLeft::execute(self)?,
            Opcode::ShiftRight(op) => ShiftRight::execute(self)?,
            Opcode::UnsignedShiftRight(op) => UnsignedShiftRight::execute(self)?,
            Opcode::Eq(op) => Eq::execute(self)?,
            Opcode::NotEq(op) => NotEq::execute(self)?,
            Opcode::StrictEq(op) => StrictEq::execute(self)?,
            Opcode::StrictNotEq(op) => StrictNotEq::execute(self)?,
            Opcode::GreaterThan(op) => GreaterThan::execute(self)?,
            Opcode::GreaterThanOrEq(op) => GreaterThanOrEq::execute(self)?,
            Opcode::LessThan(op) => LessThan::execute(self)?,
            Opcode::LessThanOrEq(op) => LessThanOrEq::execute(self)?,
            Opcode::In(op) => In::execute(self)?,
            Opcode::InstanceOf(op) => InstanceOf::execute(self)?,
            Opcode::Void(op) => Void::execute(self)?,
            Opcode::TypeOf(op) => TypeOf::execute(self)?,
            Opcode::Pos(op) => Pos::execute(self)?,
            Opcode::Neg(op) => Neg::execute(self)?,
            Opcode::Inc(op) => Inc::execute(self)?,
            Opcode::IncPost(op) => IncPost::execute(self)?,
            Opcode::Dec(op) => Dec::execute(self)?,
            Opcode::DecPost(op) => DecPost::execute(self)?,
            Opcode::LogicalNot(op) => LogicalNot::execute(self)?,
            Opcode::BitNot(op) => BitNot::execute(self)?,
            Opcode::DefVar(op) => DefVar::execute(self)?,
            Opcode::DefInitVar(op) => DefInitVar::execute(self)?,
            Opcode::DefLet(op) => DefLet::execute(self)?,
            Opcode::DefInitLet(op) => DefInitLet::execute(self)?,
            Opcode::DefInitConst(op) => DefInitConst::execute(self)?,
            Opcode::DefInitArg(op) => DefInitArg::execute(self)?,
            Opcode::GetName(op) => GetName::execute(self)?,
            Opcode::GetNameOrUndefined(op) => GetNameOrUndefined::execute(self)?,
            Opcode::SetName(op) => SetName::execute(self)?,
            Opcode::Jump(op) => Jump::execute(self)?,
            Opcode::JumpIfFalse(op) => JumpIfFalse::execute(self)?,
            Opcode::JumpIfNotUndefined(op) => JumpIfNotUndefined::execute(self)?,
            Opcode::LogicalAnd(op) => LogicalAnd::execute(self)?,
            Opcode::LogicalOr(op) => LogicalOr::execute(self)?,
            Opcode::Coalesce(op) => Coalesce::execute(self)?,
            Opcode::ToBoolean(op) => ToBoolean::execute(self)?,
            Opcode::GetPropertyByName(op) => GetPropertyByName::execute(self)?,
            Opcode::GetPropertyByValue(op) => GetPropertyByValue::execute(self)?,
            Opcode::GetPropertyByValuePush(op) => GetPropertyByValuePush::execute(self)?,
            Opcode::SetPropertyByName(op) => SetPropertyByName::execute(self)?,
            Opcode::DefineOwnPropertyByName(op) => DefineOwnPropertyByName::execute(self)?,
            Opcode::DefineClassMethodByName(op) => DefineClassMethodByName::execute(self)?,
            Opcode::SetPropertyByValue(op) => SetPropertyByValue::execute(self)?,
            Opcode::DefineOwnPropertyByValue(op) => DefineOwnPropertyByValue::execute(self)?,
            Opcode::DefineClassMethodByValue(op) => DefineClassMethodByValue::execute(self)?,
            Opcode::SetPropertyGetterByName(op) => SetPropertyGetterByName::execute(self)?,
            Opcode::DefineClassGetterByName(op) => DefineClassGetterByName::execute(self)?,
            Opcode::SetPropertyGetterByValue(op) => SetPropertyGetterByValue::execute(self)?,
            Opcode::DefineClassGetterByValue(op) => DefineClassGetterByValue::execute(self)?,
            Opcode::SetPropertySetterByName(op) => SetPropertySetterByName::execute(self)?,
            Opcode::DefineClassSetterByName(op) => DefineClassSetterByName::execute(self)?,
            Opcode::SetPropertySetterByValue(op) => SetPropertySetterByValue::execute(self)?,
            Opcode::DefineClassSetterByValue(op) => DefineClassSetterByValue::execute(self)?,
            Opcode::AssignPrivateField(op) => AssignPrivateField::execute(self)?,
            Opcode::SetPrivateField(op) => SetPrivateField::execute(self)?,
            Opcode::SetPrivateMethod(op) => SetPrivateMethod::execute(self)?,
            Opcode::SetPrivateSetter(op) => SetPrivateSetter::execute(self)?,
            Opcode::SetPrivateGetter(op) => SetPrivateGetter::execute(self)?,
            Opcode::GetPrivateField(op) => GetPrivateField::execute(self)?,
            Opcode::PushClassField(op) => PushClassField::execute(self)?,
            Opcode::PushClassFieldPrivate(op) => PushClassFieldPrivate::execute(self)?,
            Opcode::PushClassPrivateGetter(op) => PushClassPrivateGetter::execute(self)?,
            Opcode::PushClassPrivateSetter(op) => PushClassPrivateSetter::execute(self)?,
            Opcode::PushClassPrivateMethod(op) => PushClassPrivateMethod::execute(self)?,
            Opcode::DeletePropertyByName(op) => DeletePropertyByName::execute(self)?,
            Opcode::DeletePropertyByValue(op) => DeletePropertyByValue::execute(self)?,
            Opcode::CopyDataProperties(op) => CopyDataProperties::execute(self)?,
            Opcode::ToPropertyKey(op) => ToPropertyKey::execute(self)?,
            Opcode::Throw(op) => Throw::execute(self)?,
            Opcode::TryStart(op) => TryStart::execute(self)?,
            Opcode::TryEnd(op) => TryEnd::execute(self)?,
            Opcode::CatchEnd(op) => CatchEnd::execute(self)?,
            Opcode::CatchStart(op) => CatchStart::execute(self)?,
            Opcode::CatchEnd2(op) => CatchEnd2::execute(self)?,
            Opcode::FinallyStart(op) => FinallyStart::execute(self)?,
            Opcode::FinallyEnd(op) => FinallyEnd::execute(self)?,
            Opcode::FinallySetJump(op) => FinallySetJump::execute(self)?,
            Opcode::This(op) => This::execute(self)?,
            Opcode::Super(op) => Super::execute(self)?,
            Opcode::SuperCall(op) => SuperCall::execute(self)?,
            Opcode::SuperCallSpread(op) => SuperCallSpread::execute(self)?,
            Opcode::SuperCallDerived(op) => SuperCallDerived::execute(self)?,
            Opcode::Case(op) => Case::execute(self)?,
            Opcode::Default(op) => Default::execute(self)?,
            Opcode::GetFunction(op) => GetFunction::execute(self)?,
            Opcode::GetFunctionAsync(op) => GetFunctionAsync::execute(self)?,
            Opcode::GetGenerator(op) => GetGenerator::execute(self)?,
            Opcode::GetGeneratorAsync(op) => GetGeneratorAsync::execute(self)?,
            Opcode::CallEval(op) => CallEval::execute(self)?,
            Opcode::CallEvalSpread(op) => CallEvalSpread::execute(self)?,
            Opcode::Call(op) => Call::execute(self)?,
            Opcode::CallSpread(op) => CallSpread::execute(self)?,
            Opcode::New(op) => New::execute(self)?,
            Opcode::NewSpread(op) => NewSpread::execute(self)?,
            Opcode::Return(op) => Return::execute(self)?,
            Opcode::PushDeclarativeEnvironment(op) => PushDeclarativeEnvironment::execute(self)?,
            Opcode::PushFunctionEnvironment(op) => PushFunctionEnvironment::execute(self)?,
            Opcode::PopEnvironment(op) => PopEnvironment::execute(self)?,
            Opcode::LoopStart(op) => LoopStart::execute(self)?,
            Opcode::LoopContinue(op) => LoopContinue::execute(self)?,
            Opcode::LoopEnd(op) => LoopEnd::execute(self)?,
            Opcode::ForInLoopInitIterator(op) => ForInLoopInitIterator::execute(self)?,
            Opcode::InitIterator(op) => InitIterator::execute(self)?,
            Opcode::InitIteratorAsync(op) => InitIteratorAsync::execute(self)?,
            Opcode::IteratorNext(op) => IteratorNext::execute(self)?,
            Opcode::IteratorClose(op) => IteratorClose::execute(self)?,
            Opcode::IteratorToArray(op) => IteratorToArray::execute(self)?,
            Opcode::ForInLoopNext(op) => ForInLoopNext::execute(self)?,
            Opcode::ForAwaitOfLoopIterate(op) => ForAwaitOfLoopIterate::execute(self)?,
            Opcode::ForAwaitOfLoopNext(op) => ForAwaitOfLoopNext::execute(self)?,
            Opcode::ConcatToString(op) => ConcatToString::execute(self)?,
            Opcode::RequireObjectCoercible(op) => RequireObjectCoercible::execute(self)?,
            Opcode::ValueNotNullOrUndefined(op) => ValueNotNullOrUndefined::execute(self)?,
            Opcode::RestParameterInit(op) => RestParameterInit::execute(self)?,
            Opcode::RestParameterPop(op) => RestParameterPop::execute(self)?,
            Opcode::PopOnReturnAdd(op) => PopOnReturnAdd::execute(self)?,
            Opcode::PopOnReturnSub(op) => PopOnReturnSub::execute(self)?,
            Opcode::Yield(op) => Yield::execute(self)?,
            Opcode::GeneratorNext(op) => GeneratorNext::execute(self)?,
            Opcode::AsyncGeneratorNext(op) => AsyncGeneratorNext::execute(self)?,
            Opcode::GeneratorNextDelegate(op) => GeneratorNextDelegate::execute(self)?,
            Opcode::Await(op) => Await::execute(self)?,
            Opcode::PushNewTarget(op) => PushNewTarget::execute(self)?,
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
