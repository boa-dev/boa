//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::{iterable::IteratorRecord, Array, ForInIterator, Number},
    property::{DescriptorKind, PropertyDescriptor, PropertyKey},
    value::Numeric,
    vm::{
        call_frame::CatchAddresses,
        code_block::{create_function_object, create_generator_function_object, Readable},
    },
    Context, JsBigInt, JsResult, JsString, JsValue,
};
use boa_interner::ToInternedString;
use boa_profiler::Profiler;
use std::{convert::TryInto, mem::size_of, ops::Neg, time::Instant};

mod call_frame;
mod code_block;
mod opcode;

pub use {call_frame::CallFrame, code_block::CodeBlock, opcode::Opcode};

pub(crate) use {
    call_frame::{FinallyReturn, GeneratorResumeKind, TryStackEntry},
    opcode::BindingOpcode,
};

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
        self.frame.as_ref().expect("no frame found")
    }

    /// Retrieves the VM frame mutably
    ///
    /// # Panics
    ///
    /// If there is no frame, then this will panic.
    #[inline]
    pub(crate) fn frame_mut(&mut self) -> &mut CallFrame {
        self.frame.as_mut().expect("no frame found")
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

/// Indicates if the execution should continue, exit or yield.
#[derive(Debug, Clone, Copy)]
enum ShouldExit {
    True,
    False,
    Yield,
}

/// Indicates if the execution of a codeblock has ended normally or has been yielded.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ReturnType {
    Normal,
    Yield,
}

impl Context {
    fn execute_instruction(&mut self) -> JsResult<ShouldExit> {
        macro_rules! bin_op {
            ($op:ident) => {{
                let rhs = self.vm.pop();
                let lhs = self.vm.pop();
                let value = lhs.$op(&rhs, self)?;
                self.vm.push(value)
            }};
        }

        let opcode: Opcode = {
            let _timer = Profiler::global().start_event("Opcode retrieval", "vm");
            let opcode = self.vm.frame().code.code[self.vm.frame().pc]
                .try_into()
                .expect("could not convert code at PC to opcode");
            self.vm.frame_mut().pc += 1;
            opcode
        };

        let _timer = Profiler::global().start_event(&format!("INST - {}", opcode.as_str()), "vm");

        match opcode {
            Opcode::Nop => {}
            Opcode::Pop => {
                let _val = self.vm.pop();
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
            Opcode::PushUndefined => self.vm.push(JsValue::undefined()),
            Opcode::PushNull => self.vm.push(JsValue::null()),
            Opcode::PushTrue => self.vm.push(true),
            Opcode::PushFalse => self.vm.push(false),
            Opcode::PushZero => self.vm.push(0),
            Opcode::PushOne => self.vm.push(1),
            Opcode::PushInt8 => {
                let value = self.vm.read::<i8>();
                self.vm.push(i32::from(value));
            }
            Opcode::PushInt16 => {
                let value = self.vm.read::<i16>();
                self.vm.push(i32::from(value));
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
                self.vm.push(value);
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
                let o = array.as_object().expect("should be an object");
                let len = o
                    .length_of_array_like(self)
                    .expect("should have 'length' property");
                o.create_data_property_or_throw(len, value, self)
                    .expect("should be able to create new data property");
                self.vm.push(array);
            }
            Opcode::PushElisionToArray => {
                let array = self.vm.pop();
                let o = array.as_object().expect("should always be an object");

                let len = o
                    .length_of_array_like(self)
                    .expect("arrays should always have a 'length' property");

                o.set("length", len + 1, true, self)?;
                self.vm.push(array);
            }
            Opcode::PushIteratorToArray => {
                let next_function = self.vm.pop();
                let iterator = self.vm.pop();
                let array = self.vm.pop();

                let iterator = IteratorRecord::new(iterator, next_function);
                while let Some(next) = iterator.step(self)? {
                    Array::push(&array, &[next.value(self)?], self)?;
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
                let _old = self.vm.pop();
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
                        self.vm.push(JsBigInt::add(&bigint, &JsBigInt::one()));
                    }
                }
            }
            Opcode::Dec => {
                let value = self.vm.pop();
                match value.to_numeric(self)? {
                    Numeric::Number(number) => self.vm.push(number - 1f64),
                    Numeric::BigInt(bigint) => {
                        self.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()));
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
            Opcode::DefVar => {
                let index = self.vm.read::<u32>();
                let binding_locator = self.vm.frame().code.bindings[index as usize];

                if binding_locator.is_global() {
                    let key = self
                        .interner()
                        .resolve_expect(binding_locator.name())
                        .into();
                    self.global_bindings_mut().entry(key).or_insert(
                        PropertyDescriptor::builder()
                            .value(JsValue::Undefined)
                            .writable(true)
                            .enumerable(true)
                            .configurable(true)
                            .build(),
                    );
                } else {
                    self.realm.environments.put_value_if_uninitialized(
                        binding_locator.environment_index(),
                        binding_locator.binding_index(),
                        JsValue::Undefined,
                    );
                }
            }
            Opcode::DefInitVar => {
                let index = self.vm.read::<u32>();
                let value = self.vm.pop();
                let binding_locator = self.vm.frame().code.bindings[index as usize];
                binding_locator.throw_mutate_immutable(self)?;

                if binding_locator.is_global() {
                    let key = self
                        .interner()
                        .resolve_expect(binding_locator.name())
                        .into();
                    crate::object::internal_methods::global::global_set_no_receiver(
                        &key, value, self,
                    )?;
                } else {
                    self.realm.environments.put_value(
                        binding_locator.environment_index(),
                        binding_locator.binding_index(),
                        value,
                    );
                }
            }
            Opcode::DefLet => {
                let index = self.vm.read::<u32>();
                let binding_locator = self.vm.frame().code.bindings[index as usize];
                self.realm.environments.put_value(
                    binding_locator.environment_index(),
                    binding_locator.binding_index(),
                    JsValue::Undefined,
                );
            }
            Opcode::DefInitLet | Opcode::DefInitConst | Opcode::DefInitArg => {
                let index = self.vm.read::<u32>();
                let value = self.vm.pop();
                let binding_locator = self.vm.frame().code.bindings[index as usize];
                self.realm.environments.put_value(
                    binding_locator.environment_index(),
                    binding_locator.binding_index(),
                    value,
                );
            }
            Opcode::GetName => {
                let index = self.vm.read::<u32>();
                let binding_locator = self.vm.frame().code.bindings[index as usize];
                binding_locator.throw_mutate_immutable(self)?;

                let value = if binding_locator.is_global() {
                    let key: JsString = self
                        .interner()
                        .resolve_expect(binding_locator.name())
                        .into();
                    match self.global_bindings_mut().get(&key) {
                        Some(desc) => match desc.kind() {
                            DescriptorKind::Data {
                                value: Some(value), ..
                            } => value.clone(),
                            DescriptorKind::Accessor { get: Some(get), .. }
                                if !get.is_undefined() =>
                            {
                                let get = get.clone();
                                self.call(&get, &self.global_object().clone().into(), &[])?
                            }
                            _ => {
                                return self.throw_reference_error(format!("{key} is not defined"))
                            }
                        },
                        _ => return self.throw_reference_error(format!("{key} is not defined")),
                    }
                } else if let Some(value) = self.realm.environments.get_value_optional(
                    binding_locator.environment_index(),
                    binding_locator.binding_index(),
                ) {
                    value
                } else {
                    let name =
                        JsString::from(self.interner().resolve_expect(binding_locator.name()));
                    return self.throw_reference_error(format!("{name} is not initialized"));
                };

                self.vm.push(value);
            }
            Opcode::GetNameOrUndefined => {
                let index = self.vm.read::<u32>();
                let binding_locator = self.vm.frame().code.bindings[index as usize];
                binding_locator.throw_mutate_immutable(self)?;
                let value = if binding_locator.is_global() {
                    let key: JsString = self
                        .interner()
                        .resolve_expect(binding_locator.name())
                        .into();
                    match self.global_bindings_mut().get(&key) {
                        Some(desc) => match desc.kind() {
                            DescriptorKind::Data {
                                value: Some(value), ..
                            } => value.clone(),
                            DescriptorKind::Accessor { get: Some(get), .. }
                                if !get.is_undefined() =>
                            {
                                let get = get.clone();
                                self.call(&get, &self.global_object().clone().into(), &[])?
                            }
                            _ => JsValue::undefined(),
                        },
                        _ => JsValue::undefined(),
                    }
                } else if let Some(value) = self.realm.environments.get_value_optional(
                    binding_locator.environment_index(),
                    binding_locator.binding_index(),
                ) {
                    value
                } else {
                    JsValue::undefined()
                };

                self.vm.push(value);
            }
            Opcode::SetName => {
                let index = self.vm.read::<u32>();
                let binding_locator = self.vm.frame().code.bindings[index as usize];
                let value = self.vm.pop();
                binding_locator.throw_mutate_immutable(self)?;

                if binding_locator.is_global() {
                    let key: JsString = self
                        .interner()
                        .resolve_expect(binding_locator.name())
                        .into();
                    let exists = self.global_bindings_mut().contains_key(&key);

                    if !exists && (self.strict() || self.vm.frame().code.strict) {
                        return self.throw_reference_error(format!(
                            "assignment to undeclared variable {key}"
                        ));
                    }

                    let success = crate::object::internal_methods::global::global_set_no_receiver(
                        &key.clone().into(),
                        value,
                        self,
                    )?;

                    if !success && (self.strict() || self.vm.frame().code.strict) {
                        return self
                            .throw_type_error(format!("cannot set non-writable property: {key}",));
                    }
                } else if !self.realm.environments.put_value_if_initialized(
                    binding_locator.environment_index(),
                    binding_locator.binding_index(),
                    value,
                ) {
                    self.throw_reference_error(format!(
                        "cannot access '{}' before initialization",
                        self.interner().resolve_expect(binding_locator.name())
                    ))?;
                }
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
                    self.vm.push(value);
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

                let name = self.vm.frame().code.variables[index as usize];
                let name: PropertyKey = self.interner().resolve_expect(name).into();
                let result = object.get(name, self)?;

                self.vm.push(result);
            }
            Opcode::GetPropertyByValue => {
                let object = self.vm.pop();
                let key = self.vm.pop();
                let object = if let Some(object) = object.as_object() {
                    object.clone()
                } else {
                    object.to_object(self)?
                };

                let key = key.to_property_key(self)?;
                let value = object.get(key, self)?;

                self.vm.push(value);
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

                let name = self.vm.frame().code.variables[index as usize];
                let name: PropertyKey = self.interner().resolve_expect(name).into();

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

                let name = self.vm.frame().code.variables[index as usize];
                let name = self.interner().resolve_expect(name);

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
                let value = self.vm.pop();
                let key = self.vm.pop();
                let object = self.vm.pop();
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

                let name = self.vm.frame().code.variables[index as usize];
                let name = self.interner().resolve_expect(name).into();
                let set = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(PropertyDescriptor::set)
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
                let value = self.vm.pop();
                let key = self.vm.pop();
                let object = self.vm.pop();
                let object = object.to_object(self)?;
                let name = key.to_property_key(self)?;
                let set = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(PropertyDescriptor::set)
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
                let name = self.vm.frame().code.variables[index as usize];
                let name = self.interner().resolve_expect(name).into();
                let get = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(PropertyDescriptor::get)
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
                let value = self.vm.pop();
                let key = self.vm.pop();
                let object = self.vm.pop();
                let object = object.to_object(self)?;
                let name = key.to_property_key(self)?;
                let get = object
                    .__get_own_property__(&name, self)?
                    .as_ref()
                    .and_then(PropertyDescriptor::get)
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
                let key = self.vm.frame().code.variables[index as usize];
                let key = self.interner().resolve_expect(key).into();
                let object = self.vm.pop();
                let result = object.to_object(self)?.__delete__(&key, self)?;
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
                    excluded_keys.push(self.vm.pop().as_string().expect("not a string").clone());
                }
                let value = self.vm.pop();
                let object = value.as_object().expect("not an object");
                let source = self.vm.pop();
                object.copy_data_properties(&source, excluded_keys, self)?;
                self.vm.push(value);
            }
            Opcode::Throw => {
                let value = self.vm.pop();
                return Err(value);
            }
            Opcode::TryStart => {
                let next = self.vm.read::<u32>();
                let finally = self.vm.read::<u32>();
                let finally = if finally == 0 { None } else { Some(finally) };
                self.vm
                    .frame_mut()
                    .catch
                    .push(CatchAddresses { next, finally });
                self.vm.frame_mut().finally_jump.push(None);
                self.vm.frame_mut().finally_return = FinallyReturn::None;
                self.vm.frame_mut().try_env_stack.push(TryStackEntry {
                    num_env: 0,
                    num_loop_stack_entries: 0,
                });
            }
            Opcode::TryEnd | Opcode::CatchEnd => {
                self.vm.frame_mut().catch.pop();
                let try_stack_entry = self.vm.frame_mut().try_env_stack.pop().expect("must exist");
                for _ in 0..try_stack_entry.num_env {
                    self.realm.environments.pop();
                }
                let mut num_env = try_stack_entry.num_env;
                for _ in 0..try_stack_entry.num_loop_stack_entries {
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
                self.vm.frame_mut().finally_return = FinallyReturn::None;
            }
            Opcode::CatchStart => {
                let finally = self.vm.read::<u32>();
                self.vm.frame_mut().catch.push(CatchAddresses {
                    next: finally,
                    finally: Some(finally),
                });
                self.vm.frame_mut().try_env_stack.push(TryStackEntry {
                    num_env: 0,
                    num_loop_stack_entries: 0,
                });
            }
            Opcode::CatchEnd2 => {
                let frame = self.vm.frame_mut();
                if frame.finally_return == FinallyReturn::Err {
                    frame.finally_return = FinallyReturn::None;
                }
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
                match self.vm.frame_mut().finally_return {
                    FinallyReturn::None => {
                        if let Some(address) = address {
                            self.vm.frame_mut().pc = address as usize;
                        }
                    }
                    FinallyReturn::Ok => {
                        return Ok(ShouldExit::True);
                    }
                    FinallyReturn::Err => {
                        return Err(self.vm.pop());
                    }
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
                let this = self.vm.frame().this.clone();
                self.vm.push(this);
            }
            Opcode::Case => {
                let address = self.vm.read::<u32>();
                let cond = self.vm.pop();
                let value = self.vm.pop();

                if value.strict_equals(&cond) {
                    self.vm.frame_mut().pc = address as usize;
                } else {
                    self.vm.push(value);
                }
            }
            Opcode::Default => {
                let exit = self.vm.read::<u32>();
                let _val = self.vm.pop();
                self.vm.frame_mut().pc = exit as usize;
            }
            Opcode::GetFunction => {
                let index = self.vm.read::<u32>();
                let code = self.vm.frame().code.functions[index as usize].clone();
                let function = create_function_object(code, self);
                self.vm.push(function);
            }
            Opcode::GetGenerator => {
                let index = self.vm.read::<u32>();
                let code = self.vm.frame().code.functions[index as usize].clone();
                let function = create_generator_function_object(code, self);
                self.vm.push(function);
            }
            Opcode::Call => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argument_count = self.vm.read::<u32>();
                let mut arguments = Vec::with_capacity(argument_count as usize);
                for _ in 0..argument_count {
                    arguments.push(self.vm.pop());
                }
                arguments.reverse();

                let func = self.vm.pop();
                let mut this = self.vm.pop();

                let object = match func {
                    JsValue::Object(ref object) if object.is_callable() => object.clone(),
                    _ => return self.throw_type_error("not a callable function"),
                };

                if this.is_null_or_undefined() {
                    this = self.global_object().clone().into();
                }

                let result = object.__call__(&this, &arguments, self)?;

                self.vm.push(result);
            }
            Opcode::CallWithRest => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argument_count = self.vm.read::<u32>();
                let rest_argument = self.vm.pop();
                let mut arguments = Vec::with_capacity(argument_count as usize);
                for _ in 0..(argument_count - 1) {
                    arguments.push(self.vm.pop());
                }
                arguments.reverse();
                let func = self.vm.pop();
                let mut this = self.vm.pop();

                let iterator_record = rest_argument.get_iterator(self, None, None)?;
                let mut rest_arguments = Vec::new();
                while let Some(next) = iterator_record.step(self)? {
                    rest_arguments.push(next.value(self)?);
                }
                arguments.append(&mut rest_arguments);

                let object = match func {
                    JsValue::Object(ref object) if object.is_callable() => object.clone(),
                    _ => return self.throw_type_error("not a callable function"),
                };

                if this.is_null_or_undefined() {
                    this = self.global_object().clone().into();
                }

                let result = object.__call__(&this, &arguments, self)?;

                self.vm.push(result);
            }
            Opcode::New => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argument_count = self.vm.read::<u32>();
                let mut arguments = Vec::with_capacity(argument_count as usize);
                for _ in 0..argument_count {
                    arguments.push(self.vm.pop());
                }
                arguments.reverse();
                let func = self.vm.pop();

                let result = func
                    .as_constructor()
                    .ok_or_else(|| self.construct_type_error("not a constructor"))
                    .and_then(|cons| cons.__construct__(&arguments, &cons.clone().into(), self))?;

                self.vm.push(result);
            }
            Opcode::NewWithRest => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return self.throw_range_error("Maximum call stack size exceeded");
                }
                let argument_count = self.vm.read::<u32>();
                let rest_argument = self.vm.pop();
                let mut arguments = Vec::with_capacity(argument_count as usize);
                for _ in 0..(argument_count - 1) {
                    arguments.push(self.vm.pop());
                }
                arguments.reverse();
                let func = self.vm.pop();

                let iterator_record = rest_argument.get_iterator(self, None, None)?;
                let mut rest_arguments = Vec::new();
                while let Some(next) = iterator_record.step(self)? {
                    rest_arguments.push(next.value(self)?);
                }
                arguments.append(&mut rest_arguments);

                let result = func
                    .as_constructor()
                    .ok_or_else(|| self.construct_type_error("not a constructor"))
                    .and_then(|cons| cons.__construct__(&arguments, &cons.clone().into(), self))?;

                self.vm.push(result);
            }
            Opcode::Return => {
                if let Some(finally_address) = self.vm.frame().catch.last().and_then(|c| c.finally)
                {
                    let frame = self.vm.frame_mut();
                    frame.pc = finally_address as usize;
                    frame.finally_return = FinallyReturn::Ok;
                    frame.catch.pop();
                    let try_stack_entry =
                        self.vm.frame_mut().try_env_stack.pop().expect("must exist");
                    for _ in 0..try_stack_entry.num_env {
                        self.realm.environments.pop();
                    }
                    let mut num_env = try_stack_entry.num_env;
                    for _ in 0..try_stack_entry.num_loop_stack_entries {
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
                } else {
                    return Ok(ShouldExit::True);
                }
            }
            Opcode::PushDeclarativeEnvironment => {
                let num_bindings = self.vm.read::<u32>();
                self.realm
                    .environments
                    .push_declarative(num_bindings as usize);
                self.vm.frame_mut().loop_env_stack_inc();
                self.vm.frame_mut().try_env_stack_inc();
            }
            Opcode::PushFunctionEnvironment => {
                let num_bindings = self.vm.read::<u32>();
                let is_constructor = self.vm.frame().code.constructor;
                let is_lexical = self.vm.frame().code.this_mode.is_lexical();
                let this = if is_constructor || !is_lexical {
                    self.vm.frame().this.clone()
                } else {
                    JsValue::undefined()
                };

                self.realm
                    .environments
                    .push_function(num_bindings as usize, this);
            }
            Opcode::PopEnvironment => {
                self.realm.environments.pop();
                self.vm.frame_mut().loop_env_stack_dec();
                self.vm.frame_mut().try_env_stack_dec();
            }
            Opcode::LoopStart => {
                self.vm.frame_mut().loop_env_stack.push(0);
                self.vm.frame_mut().try_env_stack_loop_inc();
            }
            Opcode::LoopContinue => {
                let env_num = self
                    .vm
                    .frame_mut()
                    .loop_env_stack
                    .last_mut()
                    .expect("loop env stack entry must exist");
                let env_num_copy = *env_num;
                *env_num = 0;
                for _ in 0..env_num_copy {
                    self.realm.environments.pop();
                }
            }
            Opcode::LoopEnd => {
                let env_num = self
                    .vm
                    .frame_mut()
                    .loop_env_stack
                    .pop()
                    .expect("loop env stack entry must exist");
                for _ in 0..env_num {
                    self.realm.environments.pop();
                    self.vm.frame_mut().try_env_stack_dec();
                }
                self.vm.frame_mut().try_env_stack_loop_dec();
            }
            Opcode::ForInLoopInitIterator => {
                let address = self.vm.read::<u32>();

                let object = self.vm.pop();
                if object.is_null_or_undefined() {
                    self.vm.frame_mut().pc = address as usize;
                }

                let object = object.to_object(self)?;
                let iterator = ForInIterator::create_for_in_iterator(JsValue::new(object), self);
                let next_function = iterator
                    .get_property("next")
                    .as_ref()
                    .map(PropertyDescriptor::expect_value)
                    .cloned()
                    .ok_or_else(|| self.construct_type_error("Could not find property `next`"))?;

                self.vm.push(iterator);
                self.vm.push(next_function);
            }
            Opcode::InitIterator => {
                let object = self.vm.pop();
                let iterator = object.get_iterator(self, None, None)?;
                self.vm.push(iterator.iterator_object());
                self.vm.push(iterator.next_function());
            }
            Opcode::IteratorNext => {
                let next_function = self.vm.pop();
                let iterator = self.vm.pop();

                let iterator_record = IteratorRecord::new(iterator.clone(), next_function.clone());
                let next = iterator_record.step(self)?;

                self.vm.push(iterator);
                self.vm.push(next_function);
                if let Some(next) = next {
                    let value = next.value(self)?;
                    self.vm.push(value);
                } else {
                    self.vm.push(JsValue::undefined());
                }
            }
            Opcode::IteratorNextFull => {
                let next_function = self.vm.pop();
                let iterator = self.vm.pop();

                let iterator_record = IteratorRecord::new(iterator.clone(), next_function.clone());
                let next = iterator_record.step(self)?;

                self.vm.push(iterator);
                self.vm.push(next_function);
                if let Some(next) = next {
                    let value = next.value(self)?;
                    self.vm.push(false);
                    self.vm.push(value);
                } else {
                    self.vm.push(true);
                    self.vm.push(JsValue::undefined());
                }
            }
            Opcode::IteratorClose => {
                let done = self.vm.pop();
                let next_function = self.vm.pop();
                let iterator = self.vm.pop();
                if !done.as_boolean().expect("not a boolean") {
                    let iterator_record = IteratorRecord::new(iterator, next_function);
                    iterator_record.close(Ok(JsValue::Null), self)?;
                }
            }
            Opcode::IteratorToArray => {
                let next_function = self.vm.pop();
                let iterator = self.vm.pop();

                let iterator_record = IteratorRecord::new(iterator.clone(), next_function.clone());
                let mut values = Vec::new();

                while let Some(result) = iterator_record.step(self)? {
                    values.push(result.value(self)?);
                }

                let array = Array::create_array_from_list(values, self);

                self.vm.push(iterator);
                self.vm.push(next_function);
                self.vm.push(array);
            }
            Opcode::ForInLoopNext => {
                let address = self.vm.read::<u32>();

                let next_function = self.vm.pop();
                let iterator = self.vm.pop();

                let iterator_record = IteratorRecord::new(iterator.clone(), next_function.clone());
                if let Some(next) = iterator_record.step(self)? {
                    self.vm.push(iterator);
                    self.vm.push(next_function);
                    let value = next.value(self)?;
                    self.vm.push(value);
                } else {
                    self.vm.frame_mut().pc = address as usize;
                    self.vm.frame_mut().loop_env_stack_dec();
                    self.vm.frame_mut().try_env_stack_dec();
                    self.realm.environments.pop();
                    self.vm.push(iterator);
                    self.vm.push(next_function);
                }
            }
            Opcode::ConcatToString => {
                let value_count = self.vm.read::<u32>();
                let mut strings = Vec::with_capacity(value_count as usize);
                for _ in 0..value_count {
                    strings.push(self.vm.pop().to_string(self)?);
                }
                strings.reverse();
                let s = JsString::concat_array(
                    &strings.iter().map(JsString::as_str).collect::<Vec<&str>>(),
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
                    let array: _ = Array::create_array_from_list(args, self);

                    self.vm.push(array);
                } else {
                    self.vm.pop();

                    let array = Array::array_create(0, None, self)
                        .expect("could not create an empty array");
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
            Opcode::PopOnReturnAdd => {
                self.vm.frame_mut().pop_on_return += 1;
            }
            Opcode::PopOnReturnSub => {
                self.vm.frame_mut().pop_on_return -= 1;
            }
            Opcode::Yield => return Ok(ShouldExit::Yield),
            Opcode::GeneratorNext => match self.vm.frame().generator_resume_kind {
                GeneratorResumeKind::Normal => return Ok(ShouldExit::False),
                GeneratorResumeKind::Throw => {
                    let received = self.vm.pop();
                    return Err(received);
                }
                GeneratorResumeKind::Return => {
                    let mut finally_left = false;

                    while let Some(catch_addresses) = self.vm.frame().catch.last() {
                        if let Some(finally_address) = catch_addresses.finally {
                            let frame = self.vm.frame_mut();
                            frame.pc = finally_address as usize;
                            frame.finally_return = FinallyReturn::Ok;
                            frame.catch.pop();
                            finally_left = true;
                            break;
                        }
                        self.vm.frame_mut().catch.pop();
                    }

                    if finally_left {
                        return Ok(ShouldExit::False);
                    }
                    return Ok(ShouldExit::True);
                }
            },
            Opcode::GeneratorNextDelegate => {
                let done_address = self.vm.read::<u32>();
                let received = self.vm.pop();
                let next_function = self.vm.pop();
                let iterator = self.vm.pop();

                match self.vm.frame().generator_resume_kind {
                    GeneratorResumeKind::Normal => {
                        let result = self.call(&next_function, &iterator, &[received])?;
                        let result_object = result.as_object().ok_or_else(|| {
                            self.construct_type_error("generator next method returned non-object")
                        })?;
                        let done = result_object.get("done", self)?.to_boolean();
                        if done {
                            self.vm.frame_mut().pc = done_address as usize;
                            let value = result_object.get("value", self)?;
                            self.vm.push(value);
                            return Ok(ShouldExit::False);
                        }
                        let value = result_object.get("value", self)?;
                        self.vm.push(iterator);
                        self.vm.push(next_function);
                        self.vm.push(value);
                        return Ok(ShouldExit::Yield);
                    }
                    GeneratorResumeKind::Throw => {
                        let throw = iterator.get_method("throw", self)?;
                        if let Some(throw) = throw {
                            let result = throw.call(&iterator, &[received], self)?;
                            let result_object = result.as_object().ok_or_else(|| {
                                self.construct_type_error(
                                    "generator throw method returned non-object",
                                )
                            })?;
                            let done = result_object.get("done", self)?.to_boolean();
                            if done {
                                self.vm.frame_mut().pc = done_address as usize;
                                let value = result_object.get("value", self)?;
                                self.vm.push(value);
                                return Ok(ShouldExit::False);
                            }
                            let value = result_object.get("value", self)?;
                            self.vm.push(iterator);
                            self.vm.push(next_function);
                            self.vm.push(value);
                            return Ok(ShouldExit::Yield);
                        }
                        self.vm.frame_mut().pc = done_address as usize;
                        let iterator_record =
                            IteratorRecord::new(iterator.clone(), next_function.clone());
                        iterator_record.close(Ok(JsValue::Undefined), self)?;
                        let error =
                            self.construct_type_error("iterator does not have a throw method");
                        return Err(error);
                    }
                    GeneratorResumeKind::Return => {
                        let r#return = iterator.get_method("return", self)?;
                        if let Some(r#return) = r#return {
                            let result = r#return.call(&iterator, &[received], self)?;
                            let result_object = result.as_object().ok_or_else(|| {
                                self.construct_type_error(
                                    "generator return method returned non-object",
                                )
                            })?;
                            let done = result_object.get("done", self)?.to_boolean();
                            if done {
                                self.vm.frame_mut().pc = done_address as usize;
                                let value = result_object.get("value", self)?;
                                self.vm.push(value);
                                return Ok(ShouldExit::True);
                            }
                            let value = result_object.get("value", self)?;
                            self.vm.push(iterator);
                            self.vm.push(next_function);
                            self.vm.push(value);
                            return Ok(ShouldExit::Yield);
                        }
                        self.vm.frame_mut().pc = done_address as usize;
                        self.vm.push(received);
                        return Ok(ShouldExit::True);
                    }
                }
            }
        }

        Ok(ShouldExit::False)
    }

    pub(crate) fn run(&mut self) -> JsResult<(JsValue, ReturnType)> {
        const COLUMN_WIDTH: usize = 26;
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const NUMBER_OF_COLUMNS: usize = 4;

        let _timer = Profiler::global().start_event("run", "vm");

        if self.vm.trace {
            let msg = if self.vm.frame().prev.is_some() {
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
                        self.vm.push(e);
                    } else {
                        self.vm.stack.truncate(start_stack_size);
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

        if self.vm.stack.is_empty() {
            return Ok((JsValue::undefined(), ReturnType::Normal));
        }

        let result = self.vm.pop();
        self.vm.stack.truncate(start_stack_size);
        Ok((result, ReturnType::Normal))
    }
}
