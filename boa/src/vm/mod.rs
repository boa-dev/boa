//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::{iterable::IteratorRecord, Array, ForInIterator, Number},
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        function_environment_record::{BindingStatus, FunctionEnvironmentRecord},
        lexical_environment::{Environment, VariableScope},
    },
    property::PropertyDescriptor,
    value::Numeric,
    vm::code_block::Readable,
    BoaProfiler, Context, JsBigInt, JsResult, JsString, JsValue,
};
use std::{convert::TryInto, mem::size_of, ops::Neg, time::Instant};

mod call_frame;
mod code_block;
mod opcode;

pub use call_frame::CallFrame;
pub use code_block::{CodeBlock, JsVmFunction};
pub use opcode::Opcode;

#[cfg(test)]
mod tests;
/// Virtual Machine.
#[derive(Debug)]
pub struct Vm {
    pub(crate) frame: Option<Box<CallFrame>>,
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
        self.stack.pop().unwrap()
    }

    #[track_caller]
    #[inline]
    pub(crate) fn read<T: Readable>(&mut self) -> T {
        let value = self.frame().code.read::<T>(self.frame().pc);
        self.frame_mut().pc += size_of::<T>();
        value
    }

    #[inline]
    pub(crate) fn frame(&self) -> &CallFrame {
        self.frame.as_ref().unwrap()
    }

    #[inline]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame {
        self.frame.as_mut().unwrap()
    }

    #[inline]
    pub(crate) fn push_frame(&mut self, mut frame: CallFrame) {
        let prev = self.frame.take();
        frame.prev = prev;
        self.frame = Some(Box::new(frame));
    }

    #[inline]
    pub(crate) fn pop_frame(&mut self) -> Option<Box<CallFrame>> {
        let mut current = self.frame.take()?;
        self.frame = current.prev.take();
        Some(current)
    }
}

impl Context {
    fn execute_instruction(&mut self) -> JsResult<bool> {
        let _timer = BoaProfiler::global().start_event("execute_instruction", "vm");

        macro_rules! bin_op {
            ($op:ident) => {{
                let rhs = self.vm.pop();
                let lhs = self.vm.pop();
                let value = lhs.$op(&rhs, self)?;
                self.vm.push(value)
            }};
        }

        let opcode = self.vm.frame().code.code[self.vm.frame().pc]
            .try_into()
            .unwrap();
        self.vm.frame_mut().pc += 1;

        match opcode {
            Opcode::Nop => {}
            Opcode::Pop => {
                let _ = self.vm.pop();
            }
            Opcode::Dup => {
                let value = self.vm.pop();
                self.vm.push(value.clone());
                self.vm.push(value);
            }
            Opcode::Swap => {
                let first = self.vm.pop();
                let second = self.vm.pop();

                self.vm.push(first);
                self.vm.push(second);
            }
            Opcode::Swap2 => {
                let first = self.vm.pop();
                let second = self.vm.pop();
                let third = self.vm.pop();

                self.vm.push(first);
                self.vm.push(second);
                self.vm.push(third);
            }
            Opcode::PushUndefined => self.vm.push(JsValue::undefined()),
            Opcode::PushNull => self.vm.push(JsValue::null()),
            Opcode::PushTrue => self.vm.push(true),
            Opcode::PushFalse => self.vm.push(false),
            Opcode::PushZero => self.vm.push(0),
            Opcode::PushOne => self.vm.push(1),
            Opcode::PushInt8 => {
                let value = self.vm.read::<i8>();
                self.vm.push(value as i32);
            }
            Opcode::PushInt16 => {
                let value = self.vm.read::<i16>();
                self.vm.push(value as i32);
            }
            Opcode::PushInt32 => {
                let value = self.vm.read::<i32>();
                self.vm.push(value);
            }
            Opcode::PushRational => {
                let value = self.vm.read::<f64>();
                self.vm.push(value);
            }
            Opcode::PushNaN => self.vm.push(JsValue::nan()),
            Opcode::PushPositiveInfinity => self.vm.push(JsValue::positive_infinity()),
            Opcode::PushNegativeInfinity => self.vm.push(JsValue::negative_infinity()),
            Opcode::PushLiteral => {
                let index = self.vm.read::<u32>() as usize;
                let value = self.vm.frame().code.literals[index].clone();
                self.vm.push(value)
            }
            Opcode::PushEmptyObject => self.vm.push(self.construct_object()),
            Opcode::PushNewArray => {
                let array = Array::array_create(0, None, self)
                    .expect("Array creation with 0 length should never fail");
                self.vm.push(array);
            }
            Opcode::PushValueToArray => {
                let value = self.vm.pop();
                let array = self.vm.pop();
                let array = Array::add_to_array_object(&array, &[value], self)?;
                self.vm.push(array);
            }
            Opcode::PushIteratorToArray => {
                let next_function = self.vm.pop();
                let for_in_iterator = self.vm.pop();
                let array = self.vm.pop();

                let iterator = IteratorRecord::new(for_in_iterator, next_function);
                loop {
                    let next = iterator.next(self)?;

                    if next.done {
                        break;
                    } else {
                        Array::add_to_array_object(&array, &[next.value], self)?;
                    }
                }

                self.vm.push(array);
            }
            Opcode::Add => bin_op!(add),
            Opcode::Sub => bin_op!(sub),
            Opcode::Mul => bin_op!(mul),
            Opcode::Div => bin_op!(div),
            Opcode::Pow => bin_op!(pow),
            Opcode::Mod => bin_op!(rem),
            Opcode::BitAnd => bin_op!(bitand),
            Opcode::BitOr => bin_op!(bitor),
            Opcode::BitXor => bin_op!(bitxor),
            Opcode::ShiftLeft => bin_op!(shl),
            Opcode::ShiftRight => bin_op!(shr),
            Opcode::UnsignedShiftRight => bin_op!(ushr),
            Opcode::Eq => {
                let rhs = self.vm.pop();
                let lhs = self.vm.pop();
                let value = lhs.equals(&rhs, self)?;
                self.vm.push(value);
            }
            Opcode::NotEq => {
                let rhs = self.vm.pop();
                let lhs = self.vm.pop();
                let value = !lhs.equals(&rhs, self)?;
                self.vm.push(value);
            }
            Opcode::StrictEq => {
                let rhs = self.vm.pop();
                let lhs = self.vm.pop();
                self.vm.push(lhs.strict_equals(&rhs));
            }
            Opcode::StrictNotEq => {
                let rhs = self.vm.pop();
                let lhs = self.vm.pop();
                self.vm.push(!lhs.strict_equals(&rhs));
            }
            Opcode::GreaterThan => bin_op!(gt),
            Opcode::GreaterThanOrEq => bin_op!(ge),
            Opcode::LessThan => bin_op!(lt),
            Opcode::LessThanOrEq => bin_op!(le),
            Opcode::In => {
                let rhs = self.vm.pop();
                let lhs = self.vm.pop();

                if !rhs.is_object() {
                    return self.throw_type_error(format!(
                        "right-hand side of 'in' should be an object, got {}",
                        rhs.type_of()
                    ));
                }
                let key = lhs.to_property_key(self)?;
                let value = self.has_property(&rhs, &key)?;
                self.vm.push(value);
            }
            Opcode::InstanceOf => {
                let target = self.vm.pop();
                let v = self.vm.pop();
                let value = v.instance_of(&target, self)?;

                self.vm.push(value);
            }
            Opcode::Void => {
                let _ = self.vm.pop();
                self.vm.push(JsValue::undefined());
            }
            Opcode::TypeOf => {
                let value = self.vm.pop();
                self.vm.push(value.type_of());
            }
            Opcode::Pos => {
                let value = self.vm.pop();
                let value = value.to_number(self)?;
                self.vm.push(value);
            }
            Opcode::Neg => {
                let value = self.vm.pop();
                match value.to_numeric(self)? {
                    Numeric::Number(number) => self.vm.push(number.neg()),
                    Numeric::BigInt(bigint) => self.vm.push(JsBigInt::neg(&bigint)),
                }
            }
            Opcode::Inc => {
                let value = self.vm.pop();
                match value.to_numeric(self)? {
                    Numeric::Number(number) => self.vm.push(number + 1f64),
                    Numeric::BigInt(bigint) => {
                        self.vm.push(JsBigInt::add(&bigint, &JsBigInt::one()))
                    }
                }
            }
            Opcode::Dec => {
                let value = self.vm.pop();
                match value.to_numeric(self)? {
                    Numeric::Number(number) => self.vm.push(number - 1f64),
                    Numeric::BigInt(bigint) => {
                        self.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()))
                    }
                }
            }
            Opcode::LogicalNot => {
                let value = self.vm.pop();
                self.vm.push(!value.to_boolean());
            }
            Opcode::BitNot => {
                let value = self.vm.pop();
                match value.to_numeric(self)? {
                    Numeric::Number(number) => self.vm.push(Number::not(number)),
                    Numeric::BigInt(bigint) => self.vm.push(JsBigInt::not(&bigint)),
                }
            }
            Opcode::DefInitArg => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();
                let value = self.vm.pop();
                let local_env = self.get_current_environment();
                local_env
                    .create_mutable_binding(name.as_ref(), false, true, self)
                    .expect("Failed to create argument binding");
                self.initialize_binding(name.as_ref(), value)?;
            }
            Opcode::DefVar => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                if !self.has_binding(name.as_ref())? {
                    self.create_mutable_binding(name.as_ref(), false, VariableScope::Function)?;
                    self.initialize_binding(name.as_ref(), JsValue::Undefined)?;
                }
            }
            Opcode::DefInitVar => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();
                let value = self.vm.pop();

                if self.has_binding(name.as_ref())? {
                    self.set_mutable_binding(name.as_ref(), value, self.strict())?;
                } else {
                    self.create_mutable_binding(name.as_ref(), false, VariableScope::Function)?;
                    self.initialize_binding(name.as_ref(), value)?;
                }
            }
            Opcode::DefLet => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                self.create_mutable_binding(name.as_ref(), false, VariableScope::Block)?;
                self.initialize_binding(name.as_ref(), JsValue::Undefined)?;
            }
            Opcode::DefInitLet => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();
                let value = self.vm.pop();

                self.create_mutable_binding(name.as_ref(), false, VariableScope::Block)?;
                self.initialize_binding(name.as_ref(), value)?;
            }
            Opcode::DefInitConst => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();
                let value = self.vm.pop();

                self.create_immutable_binding(name.as_ref(), true, VariableScope::Block)?;
                self.initialize_binding(name.as_ref(), value)?;
            }
            Opcode::GetName => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                let value = self.get_binding_value(&name)?;
                self.vm.push(value);
            }
            Opcode::GetNameOrUndefined => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                let value = if self.has_binding(&name)? {
                    self.get_binding_value(&name)?
                } else {
                    JsValue::Undefined
                };
                self.vm.push(value)
            }
            Opcode::SetName => {
                let index = self.vm.read::<u32>();
                let value = self.vm.pop();
                let name = self.vm.frame().code.variables[index as usize].clone();

                self.set_mutable_binding(
                    name.as_ref(),
                    value,
                    self.strict() || self.vm.frame().code.strict,
                )?;
            }
            Opcode::Jump => {
                let address = self.vm.read::<u32>();
                self.vm.frame_mut().pc = address as usize;
            }
            Opcode::JumpIfFalse => {
                let address = self.vm.read::<u32>();
                if !self.vm.pop().to_boolean() {
                    self.vm.frame_mut().pc = address as usize;
                }
            }
            Opcode::JumpIfNotUndefined => {
                let address = self.vm.read::<u32>();
                let value = self.vm.pop();
                if !value.is_undefined() {
                    self.vm.frame_mut().pc = address as usize;
                    self.vm.push(value)
                }
            }
            Opcode::LogicalAnd => {
                let exit = self.vm.read::<u32>();
                let lhs = self.vm.pop();
                if !lhs.to_boolean() {
                    self.vm.frame_mut().pc = exit as usize;
                    self.vm.push(lhs);
                }
            }
            Opcode::LogicalOr => {
                let exit = self.vm.read::<u32>();
                let lhs = self.vm.pop();
                if lhs.to_boolean() {
                    self.vm.frame_mut().pc = exit as usize;
                    self.vm.push(lhs);
                }
            }
            Opcode::Coalesce => {
                let exit = self.vm.read::<u32>();
                let lhs = self.vm.pop();
                if !lhs.is_null_or_undefined() {
                    self.vm.frame_mut().pc = exit as usize;
                    self.vm.push(lhs);
                }
            }
            Opcode::ToBoolean => {
                let value = self.vm.pop();
                self.vm.push(value.to_boolean());
            }
            Opcode::GetPropertyByName => {
                let index = self.vm.read::<u32>();

                let value = self.vm.pop();
                let object = if let Some(object) = value.as_object() {
                    object.clone()
                } else {
                    value.to_object(self)?
                };

                let name = self.vm.frame().code.variables[index as usize].clone();
                let result = object.get(name, self)?;

                self.vm.push(result)
            }
            Opcode::GetPropertyByValue => {
                let value = self.vm.pop();
                let key = self.vm.pop();
                let object = if let Some(object) = value.as_object() {
                    object.clone()
                } else {
                    value.to_object(self)?
                };

                let key = key.to_property_key(self)?;
                let result = object.get(key, self)?;

                self.vm.push(result)
            }
            Opcode::SetPropertyByName => {
                let index = self.vm.read::<u32>();

                let object = self.vm.pop();
                let value = self.vm.pop();
                let object = if let Some(object) = object.as_object() {
                    object.clone()
                } else {
                    object.to_object(self)?
                };

                let name = self.vm.frame().code.variables[index as usize].clone();

                object.set(
                    name,
                    value,
                    self.strict() || self.vm.frame().code.strict,
                    self,
                )?;
            }
            Opcode::DefineOwnPropertyByName => {
                let index = self.vm.read::<u32>();

                let object = self.vm.pop();
                let value = self.vm.pop();
                let object = if let Some(object) = object.as_object() {
                    object.clone()
                } else {
                    object.to_object(self)?
                };

                let name = self.vm.frame().code.variables[index as usize].clone();

                object.__define_own_property__(
                    name.into(),
                    PropertyDescriptor::builder()
                        .value(value)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true)
                        .build(),
                    self,
                )?;
            }
            Opcode::SetPropertyByValue => {
                let object = self.vm.pop();
                let key = self.vm.pop();
                let value = self.vm.pop();
                let object = if let Some(object) = object.as_object() {
                    object.clone()
                } else {
                    object.to_object(self)?
                };

                let key = key.to_property_key(self)?;
                object.set(
                    key,
                    value,
                    self.strict() || self.vm.frame().code.strict,
                    self,
                )?;
            }
            Opcode::DefineOwnPropertyByValue => {
                let object = self.vm.pop();
                let key = self.vm.pop();
                let value = self.vm.pop();
                let object = if let Some(object) = object.as_object() {
                    object.clone()
                } else {
                    object.to_object(self)?
                };

                let key = key.to_property_key(self)?;

                object.__define_own_property__(
                    key,
                    PropertyDescriptor::builder()
                        .value(value)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true)
                        .build(),
                    self,
                )?;
            }
            Opcode::SetPropertyGetterByName => {
                let index = self.vm.read::<u32>();
                let object = self.vm.pop();
                let value = self.vm.pop();
                let object = object.to_object(self)?;

                let name = self.vm.frame().code.variables[index as usize]
                    .clone()
                    .into();
                let set = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(|a| a.set())
                    .cloned();
                object.__define_own_property__(
                    name,
                    PropertyDescriptor::builder()
                        .maybe_get(Some(value))
                        .maybe_set(set)
                        .enumerable(true)
                        .configurable(true)
                        .build(),
                    self,
                )?;
            }
            Opcode::SetPropertyGetterByValue => {
                let object = self.vm.pop();
                let key = self.vm.pop();
                let value = self.vm.pop();
                let object = object.to_object(self)?;
                let name = key.to_property_key(self)?;
                let set = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(|a| a.set())
                    .cloned();
                object.__define_own_property__(
                    name,
                    PropertyDescriptor::builder()
                        .maybe_get(Some(value))
                        .maybe_set(set)
                        .enumerable(true)
                        .configurable(true)
                        .build(),
                    self,
                )?;
            }
            Opcode::SetPropertySetterByName => {
                let index = self.vm.read::<u32>();
                let object = self.vm.pop();
                let value = self.vm.pop();
                let object = object.to_object(self)?;
                let name = self.vm.frame().code.variables[index as usize]
                    .clone()
                    .into();
                let get = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(|a| a.get())
                    .cloned();
                object.__define_own_property__(
                    name,
                    PropertyDescriptor::builder()
                        .maybe_set(Some(value))
                        .maybe_get(get)
                        .enumerable(true)
                        .configurable(true)
                        .build(),
                    self,
                )?;
            }
            Opcode::SetPropertySetterByValue => {
                let object = self.vm.pop();
                let key = self.vm.pop();
                let value = self.vm.pop();
                let object = object.to_object(self)?;
                let name = key.to_property_key(self)?;
                let get = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(|a| a.get())
                    .cloned();
                object.__define_own_property__(
                    name,
                    PropertyDescriptor::builder()
                        .maybe_set(Some(value))
                        .maybe_get(get)
                        .enumerable(true)
                        .configurable(true)
                        .build(),
                    self,
                )?;
            }
            Opcode::DeletePropertyByName => {
                let index = self.vm.read::<u32>();
                let key = self.vm.frame().code.variables[index as usize].clone();
                let object = self.vm.pop();
                let result = object.to_object(self)?.__delete__(&key.into(), self)?;
                if !result && self.strict() || self.vm.frame().code.strict {
                    return Err(self.construct_type_error("Cannot delete property"));
                }
                self.vm.push(result);
            }
            Opcode::DeletePropertyByValue => {
                let object = self.vm.pop();
                let key = self.vm.pop();
                let result = object
                    .to_object(self)?
                    .__delete__(&key.to_property_key(self)?, self)?;
                if !result && self.strict() || self.vm.frame().code.strict {
                    return Err(self.construct_type_error("Cannot delete property"));
                }
                self.vm.push(result);
            }
            Opcode::CopyDataProperties => {
                let excluded_key_count = self.vm.read::<u32>();
                let mut excluded_keys = Vec::with_capacity(excluded_key_count as usize);
                for _ in 0..excluded_key_count {
                    excluded_keys.push(self.vm.pop().as_string().unwrap().clone());
                }
                let value = self.vm.pop();
                let object = value.as_object().unwrap();
                let rest_obj = self.vm.pop();
                object.copy_data_properties(&rest_obj, excluded_keys, self)?;
                self.vm.push(value);
            }
            Opcode::Throw => {
                let value = self.vm.pop();
                return Err(value);
            }
            Opcode::TryStart => {
                let index = self.vm.read::<u32>();
                self.vm.frame_mut().catch.push(index);
                self.vm.frame_mut().finally_jump.push(None);
                self.vm.frame_mut().has_thrown = false;
            }
            Opcode::TryEnd => {
                self.vm.frame_mut().catch.pop();
                self.vm.frame_mut().has_thrown = false;
            }
            Opcode::CatchStart => {
                let index = self.vm.read::<u32>();
                self.vm.frame_mut().catch.push(index);
            }
            Opcode::CatchEnd => {
                self.vm.frame_mut().catch.pop();
                self.vm.frame_mut().has_thrown = false;
            }
            Opcode::CatchEnd2 => {
                self.vm.frame_mut().has_thrown = false;
            }
            Opcode::FinallyStart => {
                *self
                    .vm
                    .frame_mut()
                    .finally_jump
                    .last_mut()
                    .expect("finally jump must exist here") = None;
            }
            Opcode::FinallyEnd => {
                let address = self
                    .vm
                    .frame_mut()
                    .finally_jump
                    .pop()
                    .expect("finally jump must exist here");
                let has_thrown = self.vm.frame().has_thrown;
                if has_thrown {
                    self.vm.frame_mut().has_thrown = false;
                    return Err(self.vm.pop());
                }
                if let Some(address) = address {
                    self.vm.frame_mut().pc = address as usize;
                }
            }
            Opcode::FinallySetJump => {
                let address = self.vm.read::<u32>();
                *self
                    .vm
                    .frame_mut()
                    .finally_jump
                    .last_mut()
                    .expect("finally jump must exist here") = Some(address);
            }
            Opcode::This => {
                let this = self.get_this_binding()?;
                self.vm.push(this);
            }
            Opcode::Case => {
                let address = self.vm.read::<u32>();
                let cond = self.vm.pop();
                let value = self.vm.pop();

                if !value.strict_equals(&cond) {
                    self.vm.push(value);
                } else {
                    self.vm.frame_mut().pc = address as usize;
                }
            }
            Opcode::Default => {
                let exit = self.vm.read::<u32>();
                let _ = self.vm.pop();
                self.vm.frame_mut().pc = exit as usize;
            }
            Opcode::GetFunction => {
                let index = self.vm.read::<u32>();
                let code = self.vm.frame().code.functions[index as usize].clone();
                let environment = self.get_current_environment();
                let function = JsVmFunction::new(code, environment, self);
                self.vm.push(function);
            }
            Opcode::Call => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argc = self.vm.read::<u32>();
                let mut args = Vec::with_capacity(argc as usize);
                for _ in 0..argc {
                    args.push(self.vm.pop());
                }
                args.reverse();

                let func = self.vm.pop();
                let mut this = self.vm.pop();

                let object = match func {
                    JsValue::Object(ref object) if object.is_callable() => object.clone(),
                    _ => return self.throw_type_error("not a callable function"),
                };

                if this.is_null_or_undefined() {
                    this = self.global_object().into();
                }

                let result = object.__call__(&this, &args, self)?;

                self.vm.push(result);
            }
            Opcode::CallWithRest => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argc = self.vm.read::<u32>();
                let rest_arg_value = self.vm.pop();
                let mut args = Vec::with_capacity(argc as usize);
                for _ in 0..(argc - 1) {
                    args.push(self.vm.pop());
                }
                args.reverse();
                let func = self.vm.pop();
                let mut this = self.vm.pop();

                let iterator_record = rest_arg_value.get_iterator(self, None, None)?;
                let mut rest_args = Vec::new();
                loop {
                    let next = iterator_record.next(self)?;
                    if next.done {
                        break;
                    }
                    rest_args.push(next.value);
                }
                args.append(&mut rest_args);

                let object = match func {
                    JsValue::Object(ref object) if object.is_callable() => object.clone(),
                    _ => return self.throw_type_error("not a callable function"),
                };

                if this.is_null_or_undefined() {
                    this = self.global_object().into();
                }

                let result = object.__call__(&this, &args, self)?;

                self.vm.push(result);
            }
            Opcode::New => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argc = self.vm.read::<u32>();
                let mut args = Vec::with_capacity(argc as usize);
                for _ in 0..argc {
                    args.push(self.vm.pop());
                }
                args.reverse();
                let func = self.vm.pop();

                let result = func
                    .as_constructor()
                    .ok_or_else(|| self.construct_type_error("not a constructor"))
                    .and_then(|cons| cons.__construct__(&args, &cons.clone().into(), self))?;

                self.vm.push(result);
            }
            Opcode::NewWithRest => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argc = self.vm.read::<u32>();
                let rest_arg_value = self.vm.pop();
                let mut args = Vec::with_capacity(argc as usize);
                for _ in 0..(argc - 1) {
                    args.push(self.vm.pop());
                }
                args.reverse();
                let func = self.vm.pop();

                let iterator_record = rest_arg_value.get_iterator(self, None, None)?;
                let mut rest_args = Vec::new();
                loop {
                    let next = iterator_record.next(self)?;
                    if next.done {
                        break;
                    }
                    rest_args.push(next.value);
                }
                args.append(&mut rest_args);

                let result = func
                    .as_constructor()
                    .ok_or_else(|| self.construct_type_error("not a constructor"))
                    .and_then(|cons| cons.__construct__(&args, &cons.clone().into(), self))?;

                self.vm.push(result);
            }
            Opcode::Return => {
                for _ in 0..self.vm.frame().pop_env_on_return {
                    self.pop_environment();
                }
                self.vm.frame_mut().pop_env_on_return = 0;
                let _ = self.vm.pop_frame();
                return Ok(true);
            }
            Opcode::PushDeclarativeEnvironment => {
                let env = self.get_current_environment();
                self.push_environment(DeclarativeEnvironmentRecord::new(Some(env)));
                self.vm.frame_mut().pop_env_on_return += 1;
            }
            Opcode::PushFunctionEnvironment => {
                let is_constructor = self.vm.frame().code.constructor;
                let is_lexical = self.vm.frame().code.this_mode.is_lexical();
                let current_env = self.get_current_environment();
                let this = &self.vm.frame().this;

                let new_env = FunctionEnvironmentRecord::new(
                    this.clone().as_object().unwrap().clone(), //TODO: is this ok? this_function object on stack mb?
                    if is_constructor || !is_lexical {
                        Some(this.clone())
                    } else {
                        None
                    },
                    Some(current_env),
                    if is_lexical {
                        BindingStatus::Lexical
                    } else {
                        BindingStatus::Uninitialized
                    },
                    JsValue::undefined(),
                    self,
                )?;

                let new_env: Environment = new_env.into();
                self.push_environment(new_env);
            }
            Opcode::PopEnvironment => {
                let _ = self.pop_environment();
                self.vm.frame_mut().pop_env_on_return -= 1;
            }
            Opcode::ForInLoopInitIterator => {
                let address = self.vm.read::<u32>();

                let object = self.vm.pop();
                if object.is_null_or_undefined() {
                    self.vm.frame_mut().pc = address as usize;
                }

                let object = object.to_object(self)?;
                let for_in_iterator =
                    ForInIterator::create_for_in_iterator(JsValue::new(object), self);
                let next_function = for_in_iterator
                    .get_property("next")
                    .as_ref()
                    .map(|p| p.expect_value())
                    .cloned()
                    .ok_or_else(|| self.construct_type_error("Could not find property `next`"))?;

                self.vm.push(for_in_iterator);
                self.vm.push(next_function);
            }
            Opcode::InitIterator => {
                let iterable = self.vm.pop();
                let iterator = iterable.get_iterator(self, None, None)?;

                self.vm.push(iterator.iterator_object());
                self.vm.push(iterator.next_function());
            }
            Opcode::IteratorNext => {
                let next_function = self.vm.pop();
                let for_in_iterator = self.vm.pop();

                let iterator = IteratorRecord::new(for_in_iterator.clone(), next_function.clone());
                let iterator_result = iterator.next(self)?;

                self.vm.push(for_in_iterator);
                self.vm.push(next_function);
                self.vm.push(iterator_result.value);
            }
            Opcode::IteratorNextFull => {
                let next_function = self.vm.pop();
                let for_in_iterator = self.vm.pop();

                let iterator = IteratorRecord::new(for_in_iterator.clone(), next_function.clone());
                let iterator_result = iterator.next(self)?;

                self.vm.push(for_in_iterator);
                self.vm.push(next_function);
                self.vm.push(iterator_result.done);
                self.vm.push(iterator_result.value);
            }
            Opcode::IteratorClose => {
                let done = self.vm.pop();
                let next_function = self.vm.pop();
                let for_in_iterator = self.vm.pop();
                if !done.as_boolean().unwrap() {
                    let iterator = IteratorRecord::new(for_in_iterator, next_function);
                    iterator.close(Ok(JsValue::Null), self)?;
                }
            }
            Opcode::IteratorToArray => {
                let next_function = self.vm.pop();
                let for_in_iterator = self.vm.pop();

                let iterator = IteratorRecord::new(for_in_iterator.clone(), next_function.clone());
                let mut values = Vec::new();

                loop {
                    let next = iterator.next(self)?;

                    if next.done {
                        break;
                    }

                    values.push(next.value);
                }

                let array = Array::array_create(0, None, self)
                    .expect("Array creation with 0 length should never fail");

                Array::add_to_array_object(&array.clone().into(), &values, self)?;

                self.vm.push(for_in_iterator);
                self.vm.push(next_function);
                self.vm.push(array);
            }
            Opcode::ForInLoopNext => {
                let address = self.vm.read::<u32>();

                let next_function = self.vm.pop();
                let for_in_iterator = self.vm.pop();

                let iterator = IteratorRecord::new(for_in_iterator.clone(), next_function.clone());
                let iterator_result = iterator.next(self)?;
                if iterator_result.done {
                    self.vm.frame_mut().pc = address as usize;
                    self.vm.frame_mut().pop_env_on_return -= 1;
                    self.pop_environment();
                    self.vm.push(for_in_iterator);
                    self.vm.push(next_function);
                } else {
                    self.vm.push(for_in_iterator);
                    self.vm.push(next_function);
                    self.vm.push(iterator_result.value);
                }
            }
            Opcode::ConcatToString => {
                let n = self.vm.read::<u32>();
                let mut strings = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    strings.push(self.vm.pop().to_string(self)?);
                }
                strings.reverse();
                let s = JsString::concat_array(
                    &strings.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                );
                self.vm.push(s);
            }
            Opcode::RequireObjectCoercible => {
                let value = self.vm.pop();
                let value = value.require_object_coercible(self)?;
                self.vm.push(value);
            }
            Opcode::ValueNotNullOrUndefined => {
                let value = self.vm.pop();
                if value.is_null() {
                    return self.throw_type_error("Cannot destructure 'null' value");
                }
                if value.is_undefined() {
                    return self.throw_type_error("Cannot destructure 'undefined' value");
                }
                self.vm.push(value);
            }
            Opcode::RestParameterInit => {
                let arg_count = self.vm.frame().arg_count;
                let param_count = self.vm.frame().param_count;
                if arg_count >= param_count {
                    let rest_count = arg_count - param_count + 1;
                    let mut args = Vec::with_capacity(rest_count);
                    for _ in 0..rest_count {
                        args.push(self.vm.pop());
                    }
                    let array = Array::new_array(self);
                    Array::add_to_array_object(&array, &args, self).unwrap();
                    self.vm.push(array);
                } else {
                    self.vm.pop();
                    let array = Array::new_array(self);
                    self.vm.push(array);
                }
            }
            Opcode::RestParameterPop => {
                let arg_count = self.vm.frame().arg_count;
                let param_count = self.vm.frame().param_count;
                if arg_count > param_count {
                    for _ in 0..(arg_count - param_count) {
                        self.vm.pop();
                    }
                }
            }
        }

        Ok(false)
    }

    pub(crate) fn run(&mut self) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("run", "vm");

        const COLUMN_WIDTH: usize = 26;
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const NUMBER_OF_COLUMNS: usize = 4;

        if self.vm.trace {
            let msg = if self.vm.frame().prev.is_some() {
                " Call Frame "
            } else {
                " VM Start "
            };

            println!("{}\n", self.vm.frame().code);
            println!(
                "{:-^width$}",
                msg,
                width = COLUMN_WIDTH * NUMBER_OF_COLUMNS - 10
            );
            println!(
                "{:<time_width$} {:<opcode_width$} {:<operand_width$} Top Of Stack\n",
                "Time",
                "Opcode",
                "Operands",
                time_width = TIME_COLUMN_WIDTH,
                opcode_width = OPCODE_COLUMN_WIDTH,
                operand_width = OPERAND_COLUMN_WIDTH,
            );
        }

        self.vm.frame_mut().pc = 0;
        while self.vm.frame().pc < self.vm.frame().code.code.len() {
            let result = if self.vm.trace {
                let mut pc = self.vm.frame().pc;
                let opcode: Opcode = self.vm.frame().code.read::<u8>(pc).try_into().unwrap();
                let operands = self.vm.frame().code.instruction_operands(&mut pc);

                let instant = Instant::now();
                let result = self.execute_instruction();
                let duration = instant.elapsed();

                println!(
                    "{:<time_width$} {:<opcode_width$} {:<operand_width$} {}",
                    format!("{}μs", duration.as_micros()),
                    opcode.as_str(),
                    operands,
                    match self.vm.stack.last() {
                        None => "<empty>".to_string(),
                        Some(value) => {
                            if value.is_callable() {
                                "[function]".to_string()
                            } else if value.is_object() {
                                "[object]".to_string()
                            } else {
                                format!("{}", value.display())
                            }
                        }
                    },
                    time_width = TIME_COLUMN_WIDTH,
                    opcode_width = OPCODE_COLUMN_WIDTH,
                    operand_width = OPERAND_COLUMN_WIDTH,
                );

                result
            } else {
                self.execute_instruction()
            };

            match result {
                Ok(should_exit) => {
                    if should_exit {
                        let result = self.vm.pop();
                        return Ok(result);
                    }
                }
                Err(e) => {
                    if let Some(address) = self.vm.frame().catch.last() {
                        let address = *address;
                        if self.vm.frame().pop_env_on_return > 0 {
                            self.pop_environment();
                            self.vm.frame_mut().pop_env_on_return -= 1;
                        }
                        self.vm.frame_mut().pc = address as usize;
                        self.vm.frame_mut().catch.pop();
                        self.vm.frame_mut().has_thrown = true;
                        self.vm.push(e);
                    } else {
                        for _ in 0..self.vm.frame().pop_env_on_return {
                            self.pop_environment();
                        }
                        self.vm.pop_frame();

                        return Err(e);
                    }
                }
            }
        }

        if self.vm.trace {
            println!("\nStack:");
            if !self.vm.stack.is_empty() {
                for (i, value) in self.vm.stack.iter().enumerate() {
                    println!(
                        "{:04}{:<width$} {}",
                        i,
                        "",
                        if value.is_callable() {
                            "[function]".to_string()
                        } else if value.is_object() {
                            "[object]".to_string()
                        } else {
                            format!("{}", value.display())
                        },
                        width = COLUMN_WIDTH / 2 - 4,
                    );
                }
            } else {
                println!("    <empty>");
            }
            println!("\n");
        }

        if self.vm.stack.is_empty() {
            return Ok(JsValue::undefined());
        }

        Ok(self.vm.pop())
    }
}
