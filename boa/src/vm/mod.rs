//! The Virtual Machine (VM) handles generating instructions, then executing them.
//! This module will provide an instruction set for the AST to use, various traits,
//! plus an interpreter to execute those instructions

use crate::{
    builtins::Array, environment::lexical_environment::VariableScope, symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult, JsValue,
};

mod code_block;
mod opcode;

pub use code_block::CodeBlock;
pub use opcode::Opcode;

use std::{convert::TryInto, mem::size_of, time::Instant};

use self::code_block::Readable;

/// Virtual Machine.
#[derive(Debug)]
pub struct Vm<'a> {
    context: &'a mut Context,
    pc: usize,
    code: CodeBlock,
    stack: Vec<JsValue>,
    stack_pointer: usize,
    is_trace: bool,
}

#[cfg(test)]
mod tests;

impl<'a> Vm<'a> {
    pub fn new(code: CodeBlock, context: &'a mut Context) -> Self {
        let trace = context.trace;
        Self {
            context,
            pc: 0,
            code,
            stack: Vec::with_capacity(128),
            stack_pointer: 0,
            is_trace: trace,
        }
    }

    /// Push a value on the stack.
    #[inline]
    pub fn push<T>(&mut self, value: T)
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
    pub fn pop(&mut self) -> JsValue {
        self.stack.pop().unwrap()
    }

    fn read<T: Readable>(&mut self) -> T {
        let value = self.code.read::<T>(self.pc);
        self.pc += size_of::<T>();
        value
    }

    fn execute_instruction(&mut self) -> JsResult<()> {
        let _timer = BoaProfiler::global().start_event("execute_instruction", "vm");

        macro_rules! bin_op {
            ($op:ident) => {{
                let rhs = self.pop();
                let lhs = self.pop();
                let value = lhs.$op(&rhs, self.context)?;
                self.push(value)
            }};
        }

        let opcode = self.code.code[self.pc].try_into().unwrap();
        self.pc += 1;

        match opcode {
            Opcode::Nop => {}
            Opcode::Pop => {
                let _ = self.pop();
            }
            Opcode::Dup => {
                let value = self.pop();
                self.push(value.clone());
                self.push(value);
            }
            Opcode::Swap => {
                let first = self.pop();
                let second = self.pop();

                self.push(first);
                self.push(second);
            }
            Opcode::PushUndefined => self.push(JsValue::undefined()),
            Opcode::PushNull => self.push(JsValue::null()),
            Opcode::PushTrue => self.push(true),
            Opcode::PushFalse => self.push(false),
            Opcode::PushZero => self.push(0),
            Opcode::PushOne => self.push(1),
            Opcode::PushInt8 => {
                let value = self.read::<i8>();
                self.push(value as i32);
            }
            Opcode::PushInt16 => {
                let value = self.read::<i16>();
                self.push(value as i32);
            }
            Opcode::PushInt32 => {
                let value = self.read::<i32>();
                self.push(value);
            }
            Opcode::PushRational => {
                let value = self.read::<f64>();
                self.push(value);
            }
            Opcode::PushNaN => self.push(JsValue::nan()),
            Opcode::PushPositiveInfinity => self.push(JsValue::positive_inifnity()),
            Opcode::PushNegativeInfinity => self.push(JsValue::negative_inifnity()),
            Opcode::PushLiteral => {
                let index = self.read::<u32>() as usize;
                let value = self.code.literals[index].clone();
                self.push(value)
            }
            Opcode::PushEmptyObject => self.push(JsValue::new_object(self.context)),
            Opcode::PushNewArray => {
                let count = self.read::<u32>();
                let mut elements = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    elements.push(self.pop());
                }
                let array = Array::new_array(self.context);
                Array::add_to_array_object(&array, &elements, self.context)?;
                self.push(array);
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
                let rhs = self.pop();
                let lhs = self.pop();
                let value = lhs.equals(&rhs, self.context)?;
                self.push(value);
            }
            Opcode::NotEq => {
                let rhs = self.pop();
                let lhs = self.pop();
                let value = !lhs.equals(&rhs, self.context)?;
                self.push(value);
            }
            Opcode::StrictEq => {
                let rhs = self.pop();
                let lhs = self.pop();
                self.push(lhs.strict_equals(&rhs));
            }
            Opcode::StrictNotEq => {
                let rhs = self.pop();
                let lhs = self.pop();
                self.push(!lhs.strict_equals(&rhs));
            }
            Opcode::GreaterThan => bin_op!(gt),
            Opcode::GreaterThanOrEq => bin_op!(ge),
            Opcode::LessThan => bin_op!(lt),
            Opcode::LessThanOrEq => bin_op!(le),
            Opcode::In => {
                let rhs = self.pop();
                let lhs = self.pop();

                if !rhs.is_object() {
                    return Err(self.context.construct_type_error(format!(
                        "right-hand side of 'in' should be an object, got {}",
                        rhs.type_of()
                    )));
                }
                let key = lhs.to_property_key(self.context)?;
                let has_property = self.context.has_property(&rhs, &key)?;
                self.push(has_property);
            }
            Opcode::InstanceOf => {
                let y = self.pop();
                let x = self.pop();
                let value = if let Some(object) = y.as_object() {
                    let key = WellKnownSymbols::has_instance();

                    match object.get_method(self.context, key)? {
                        Some(instance_of_handler) => instance_of_handler
                            .call(&y, &[x], self.context)?
                            .to_boolean(),
                        None if object.is_callable() => {
                            object.ordinary_has_instance(self.context, &x)?
                        }
                        None => {
                            return Err(self.context.construct_type_error(
                                "right-hand side of 'instanceof' is not callable",
                            ));
                        }
                    }
                } else {
                    return Err(self.context.construct_type_error(format!(
                        "right-hand side of 'instanceof' should be an object, got {}",
                        y.type_of()
                    )));
                };

                self.push(value);
            }
            Opcode::Void => {
                let _ = self.pop();
                self.push(JsValue::undefined());
            }
            Opcode::TypeOf => {
                let value = self.pop();
                self.push(value.type_of());
            }
            Opcode::Pos => {
                let value = self.pop();
                let value = value.to_number(self.context)?;
                self.push(value);
            }
            Opcode::Neg => {
                let value = self.pop().neg(self.context)?;
                self.push(value);
            }
            Opcode::LogicalNot => {
                let value = self.pop();
                self.push(!value.to_boolean());
            }
            Opcode::BitNot => {
                let target = self.pop();
                let num = target.to_number(self.context)?;
                let value = if num.is_nan() {
                    -1
                } else {
                    // TODO: this is not spec compliant.
                    !(num as i32)
                };
                self.push(value);
            }
            Opcode::DefVar => {
                let index = self.read::<u32>();
                let name = &self.code.names[index as usize];

                self.context
                    .create_mutable_binding(name, false, VariableScope::Function)?;
            }
            Opcode::DefLet => {
                let index = self.read::<u32>();
                let name = &self.code.names[index as usize];

                self.context
                    .create_mutable_binding(name, false, VariableScope::Block)?;
            }
            Opcode::DefConst => {
                let index = self.read::<u32>();
                let name = &self.code.names[index as usize];

                self.context.create_immutable_binding(
                    name.as_ref(),
                    false,
                    VariableScope::Block,
                )?;
            }
            Opcode::InitLexical => {
                let index = self.read::<u32>();
                let value = self.pop();
                let name = &self.code.names[index as usize];

                self.context.initialize_binding(name, value)?;
            }
            Opcode::GetName => {
                let index = self.read::<u32>();
                let name = &self.code.names[index as usize];

                let value = self.context.get_binding_value(name)?;
                self.push(value);
            }
            Opcode::SetName => {
                let index = self.read::<u32>();
                let value = self.pop();
                let name = &self.code.names[index as usize];

                if self.context.has_binding(name)? {
                    // Binding already exists
                    self.context
                        .set_mutable_binding(name, value, self.context.strict())?;
                } else {
                    self.context
                        .create_mutable_binding(name, true, VariableScope::Function)?;
                    self.context.initialize_binding(name, value)?;
                }
            }
            Opcode::Jump => {
                let address = self.read::<u32>();
                self.pc = address as usize;
            }
            Opcode::JumpIfFalse => {
                let address = self.read::<u32>();
                if !self.pop().to_boolean() {
                    self.pc = address as usize;
                }
            }
            Opcode::JumpIfTrue => {
                let address = self.read::<u32>();
                if self.pop().to_boolean() {
                    self.pc = address as usize;
                }
            }
            Opcode::LogicalAnd => {
                let exit = self.read::<u32>();
                let lhs = self.pop();
                if !lhs.to_boolean() {
                    self.pc = exit as usize;
                    self.push(false);
                }
            }
            Opcode::LogicalOr => {
                let exit = self.read::<u32>();
                let lhs = self.pop();
                if lhs.to_boolean() {
                    self.pc = exit as usize;
                    self.push(true);
                }
            }
            Opcode::Coalesce => {
                let exit = self.read::<u32>();
                let lhs = self.pop();
                if !lhs.is_null_or_undefined() {
                    self.pc = exit as usize;
                    self.push(lhs);
                }
            }
            Opcode::ToBoolean => {
                let value = self.pop();
                self.push(value.to_boolean());
            }
            Opcode::GetPropertyByName => {
                let index = self.read::<u32>();

                let value = self.pop();
                let object = if let Some(object) = value.as_object() {
                    object
                } else {
                    value.to_object(self.context)?
                };

                let name = self.code.names[index as usize].clone();
                let result = object.get(name, self.context)?;

                self.push(result)
            }
            Opcode::GetPropertyByValue => {
                let value = self.pop();
                let key = self.pop();
                let object = if let Some(object) = value.as_object() {
                    object
                } else {
                    value.to_object(self.context)?
                };

                let key = key.to_property_key(self.context)?;
                let result = object.get(key, self.context)?;

                self.push(result)
            }
            Opcode::SetPropertyByName => {
                let index = self.read::<u32>();

                let object = self.pop();
                let value = self.pop();
                let object = if let Some(object) = object.as_object() {
                    object
                } else {
                    object.to_object(self.context)?
                };

                let name = self.code.names[index as usize].clone();

                object.set(name, value, true, self.context)?;
            }
            Opcode::SetPropertyByValue => {
                let object = self.pop();
                let key = self.pop();
                let value = self.pop();
                let object = if let Some(object) = object.as_object() {
                    object
                } else {
                    object.to_object(self.context)?
                };

                let key = key.to_property_key(self.context)?;
                object.set(key, value, true, self.context)?;
            }
            Opcode::Throw => {
                let value = self.pop();
                return Err(value);
            }
            Opcode::This => {
                let this = self.context.get_this_binding()?;
                self.push(this);
            }
            Opcode::Case => {
                let address = self.read::<u32>();
                let cond = self.pop();
                let value = self.pop();

                if !value.strict_equals(&cond) {
                    self.push(value);
                } else {
                    self.pc = address as usize;
                }
            }
            Opcode::Default => {
                let exit = self.read::<u32>();
                let _ = self.pop();
                self.pc = exit as usize;
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("run", "vm");

        const COLUMN_WIDTH: usize = 24;
        const TIME_COLUMN_WIDTH: usize = COLUMN_WIDTH / 2;
        const OPCODE_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const OPERAND_COLUMN_WIDTH: usize = COLUMN_WIDTH;
        const NUMBER_OF_COLUMNS: usize = 4;

        if self.is_trace {
            println!("{}\n", self.code);
            println!(
                "{:-^width$}",
                " Vm Start ",
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

        self.pc = 0;
        while self.pc < self.code.code.len() {
            if self.is_trace {
                let mut pc = self.pc;

                let instant = Instant::now();
                self.execute_instruction()?;
                let duration = instant.elapsed();

                let opcode: Opcode = self.code.read::<u8>(pc).try_into().unwrap();

                println!(
                    "{:<time_width$} {:<opcode_width$} {:<operand_width$} {}",
                    format!("{}μs", duration.as_micros()),
                    opcode.as_str(),
                    self.code.instruction_operands(&mut pc),
                    match self.stack.last() {
                        None => "<empty>".to_string(),
                        Some(value) => format!("{}", value.display()),
                    },
                    time_width = TIME_COLUMN_WIDTH,
                    opcode_width = OPCODE_COLUMN_WIDTH,
                    operand_width = OPERAND_COLUMN_WIDTH,
                );
            } else {
                self.execute_instruction()?;
            }
        }

        if self.is_trace {
            println!("\nStack:");
            if !self.stack.is_empty() {
                for (i, value) in self.stack.iter().enumerate() {
                    println!(
                        "{:04}{:<width$} {}",
                        i,
                        "",
                        value.display(),
                        width = COLUMN_WIDTH / 2 - 4,
                    );
                }
            } else {
                println!("    <empty>");
            }
            println!("\n");
        }

        if self.stack.is_empty() {
            return Ok(JsValue::undefined());
        }

        Ok(self.pop())
    }
}
