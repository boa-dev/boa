//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::Array, environment::lexical_environment::VariableScope, BoaProfiler, Context,
    JsResult, JsValue,
};

mod call_frame;
mod code_block;
mod opcode;

pub use call_frame::CallFrame;
pub use code_block::CodeBlock;
pub use code_block::JsVmFunction;
pub use opcode::Opcode;

use std::{convert::TryInto, mem::size_of, time::Instant};

use self::code_block::Readable;

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
                let count = self.vm.read::<u32>();
                let mut elements = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    elements.push(self.vm.pop());
                }
                let array = Array::create_array_from_list(elements, self);
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
                    return Err(self.construct_type_error(format!(
                        "right-hand side of 'in' should be an object, got {}",
                        rhs.type_of()
                    )));
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
                let value = self.vm.pop().neg(self)?;
                self.vm.push(value);
            }
            Opcode::LogicalNot => {
                let value = self.vm.pop();
                self.vm.push(!value.to_boolean());
            }
            Opcode::BitNot => {
                let target = self.vm.pop();
                let num = target.to_number(self)?;
                let value = if num.is_nan() {
                    -1
                } else {
                    // TODO: this is not spec compliant.
                    !(num as i32)
                };
                self.vm.push(value);
            }
            Opcode::DefVar => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                self.create_mutable_binding(name.as_ref(), false, VariableScope::Function)?;
            }
            Opcode::DefLet => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                self.create_mutable_binding(name.as_ref(), false, VariableScope::Block)?;
            }
            Opcode::DefConst => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                self.create_immutable_binding(name.as_ref(), false, VariableScope::Block)?;
            }
            Opcode::InitLexical => {
                let index = self.vm.read::<u32>();
                let value = self.vm.pop();
                let name = self.vm.frame().code.variables[index as usize].clone();

                self.initialize_binding(&name, value)?;
            }
            Opcode::GetName => {
                let index = self.vm.read::<u32>();
                let name = self.vm.frame().code.variables[index as usize].clone();

                let value = self.get_binding_value(&name)?;
                self.vm.push(value);
            }
            Opcode::SetName => {
                let index = self.vm.read::<u32>();
                let value = self.vm.pop();
                let name = self.vm.frame().code.variables[index as usize].clone();

                if self.has_binding(name.as_ref())? {
                    // Binding already exists
                    self.set_mutable_binding(name.as_ref(), value, self.strict())?;
                } else {
                    self.create_mutable_binding(name.as_ref(), true, VariableScope::Function)?;
                    self.initialize_binding(name.as_ref(), value)?;
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
            Opcode::JumpIfTrue => {
                let address = self.vm.read::<u32>();
                if self.vm.pop().to_boolean() {
                    self.vm.frame_mut().pc = address as usize;
                }
            }
            Opcode::LogicalAnd => {
                let exit = self.vm.read::<u32>();
                let lhs = self.vm.pop();
                if !lhs.to_boolean() {
                    self.vm.frame_mut().pc = exit as usize;
                    self.vm.push(false);
                }
            }
            Opcode::LogicalOr => {
                let exit = self.vm.read::<u32>();
                let lhs = self.vm.pop();
                if lhs.to_boolean() {
                    self.vm.frame_mut().pc = exit as usize;
                    self.vm.push(true);
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

                object.set(name, value, true, self)?;
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
                object.set(key, value, true, self)?;
            }
            Opcode::DeletePropertyByName => {
                let index = self.vm.read::<u32>();
                let key = self.vm.frame().code.variables[index as usize].clone();
                let object = self.vm.pop();
                let result = object.to_object(self)?.__delete__(&key.into(), self)?;
                self.vm.push(result);
            }
            Opcode::DeletePropertyByValue => {
                let object = self.vm.pop();
                let key = self.vm.pop();
                let result = object
                    .to_object(self)?
                    .__delete__(&key.to_property_key(self)?, self)?;
                self.vm.push(result);
            }
            Opcode::Throw => {
                let value = self.vm.pop();
                return Err(value);
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
                let environment = self.vm.frame().environment.clone();
                let function = JsVmFunction::new(code, environment, self);
                self.vm.push(function);
            }
            Opcode::Call => {
                if self.vm.stack_size_limit <= self.vm.stack.len() {
                    return Err(self.construct_range_error("Maximum call stack size exceeded"));
                }
                let argc = self.vm.read::<u32>();
                let func = self.vm.pop();
                let this = self.vm.pop();
                let mut args = Vec::with_capacity(argc as usize);
                for _ in 0..argc {
                    args.push(self.vm.pop());
                }

                let object = match func {
                    JsValue::Object(ref object) if object.is_callable() => object.clone(),
                    _ => return Err(self.construct_type_error("not a callable function")),
                };

                let result = object.call_internal(&this, &args, self, false)?;

                self.vm.push(result);
            }
            Opcode::Return => {
                let exit = self.vm.frame().exit_on_return;

                let _ = self.vm.pop_frame();

                if exit {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Unwind the stack.
    fn unwind(&mut self) -> bool {
        let mut fp = 0;
        while let Some(mut frame) = self.vm.frame.take() {
            fp = frame.fp;
            if frame.exit_on_return {
                break;
            }

            self.vm.frame = frame.prev.take();
        }
        while self.vm.stack.len() > fp {
            let _ = self.vm.pop();
        }
        true
    }

    pub(crate) fn run(&mut self) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("run", "vm");

        const COLUMN_WIDTH: usize = 24;
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const NUMBER_OF_COLUMNS: usize = 4;

        let msg = if self.vm.frame().exit_on_return {
            " VM Start"
        } else {
            " Call Frame "
        };

        if self.vm.trace {
            println!("{}\n", self.vm.frame().code);
            println!(
                "{:-^width$}",
                msg,
                width = COLUMN_WIDTH * NUMBER_OF_COLUMNS - 10
            );
            println!(
                "{:<time_width$} {:<opcode_width$} {:<operand_width$} Top Of Stack",
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
                    let should_exit = self.unwind();
                    if should_exit {
                        return Err(e);
                    } else {
                        self.vm.push(e);
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
