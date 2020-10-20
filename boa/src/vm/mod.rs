use crate::{Context, Result, Value};

pub(crate) mod compilation;
pub(crate) mod instructions;

use crate::BoaProfiler;
pub use compilation::Compiler;
pub use instructions::Instruction;

// Virtual Machine.
#[derive(Debug)]
pub struct VM<'a> {
    ctx: &'a mut Context,
    instructions: Vec<Instruction>,
    pool: Vec<Value>,
    stack: Vec<Value>,
    stack_pointer: usize,
}

impl<'a> VM<'a> {
    pub fn new(compiler: Compiler, ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            instructions: compiler.instructions,
            pool: compiler.pool,
            stack: vec![],
            stack_pointer: 0,
        }
    }

    /// Push a value on the stack.
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop a value off the stack.
    ///
    /// # Panics
    ///
    /// If there is nothing to pop, then this will panic.
    #[inline]
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn run(&mut self) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("runVM", "vm");
        let mut idx = 0;

        while idx < self.instructions.len() {
            let _timer =
                BoaProfiler::global().start_event(&self.instructions[idx].to_string(), "vm");
            match self.instructions[idx] {
                Instruction::Undefined => self.push(Value::undefined()),
                Instruction::Null => self.push(Value::null()),
                Instruction::True => self.push(Value::boolean(true)),
                Instruction::False => self.push(Value::boolean(false)),
                Instruction::Zero => self.push(Value::integer(0)),
                Instruction::One => self.push(Value::integer(1)),
                Instruction::Int32(i) => self.push(Value::integer(i)),
                Instruction::Rational(r) => self.push(Value::rational(r)),
                Instruction::String(index) => {
                    let value = self.pool[index].clone();
                    self.push(value)
                }
                Instruction::BigInt(index) => {
                    let value = self.pool[index].clone();
                    self.push(value)
                }
                Instruction::Add => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.add(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Sub => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.sub(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Mul => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.mul(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Div => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.div(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Pow => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.pow(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Mod => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.rem(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::BitAnd => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.bitand(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::BitOr => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.bitor(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::BitXor => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.bitxor(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Shl => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.shl(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Shr => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.shr(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::UShr => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.ushr(&r, self.ctx)?;

                    self.push(val);
                }
                Instruction::Eq => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.equals(&r, self.ctx)?;

                    self.push(val.into());
                }
                Instruction::NotEq => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = !l.equals(&r, self.ctx)?;

                    self.push(val.into());
                }
                Instruction::StrictEq => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.strict_equals(&r);

                    self.push(val.into());
                }
                Instruction::StrictNotEq => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = !l.strict_equals(&r);

                    self.push(val.into());
                }
                Instruction::Gt => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.ge(&r, self.ctx)?;

                    self.push(val.into());
                }
                Instruction::Ge => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.ge(&r, self.ctx)?;

                    self.push(val.into());
                }
                Instruction::Lt => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.lt(&r, self.ctx)?;

                    self.push(val.into());
                }
                Instruction::Le => {
                    let r = self.pop();
                    let l = self.pop();
                    let val = l.le(&r, self.ctx)?;

                    self.push(val.into());
                }
                Instruction::In => {
                    let r = self.pop();
                    let l = self.pop();

                    if !r.is_object() {
                        return self.ctx.throw_type_error(format!(
                            "right-hand side of 'in' should be an object, got {}",
                            r.get_type().as_str()
                        ));
                    }
                    let key = l.to_property_key(self.ctx)?;
                    let val = self.ctx.has_property(&r, &key);

                    self.push(val.into());
                }
                Instruction::InstanceOf => {
                    let r = self.pop();
                    let _l = self.pop();
                    if !r.is_object() {
                        return self.ctx.throw_type_error(format!(
                            "right-hand side of 'instanceof' should be an object, got {}",
                            r.get_type().as_str()
                        ));
                    }

                    // spec: https://tc39.es/ecma262/#sec-instanceofoperator
                    todo!("instanceof operator")
                }
                Instruction::Void => {
                    let _value = self.pop();
                    self.push(Value::undefined());
                }
                Instruction::TypeOf => {
                    let value = self.pop();
                    self.push(value.get_type().as_str().into());
                }
                Instruction::Pos => {
                    let value = self.pop();
                    let value = value.to_number(self.ctx)?;
                    self.push(value.into());
                }
                Instruction::Neg => {
                    let value = self.pop();
                    self.push(Value::from(!value.to_boolean()));
                }
                Instruction::Not => {
                    let value = self.pop();
                    self.push((!value.to_boolean()).into());
                }
                Instruction::BitNot => {
                    let target = self.pop();
                    let num = target.to_number(self.ctx)?;
                    let value = if num.is_nan() {
                        -1
                    } else {
                        // TODO: this is not spec compliant.
                        !(num as i32)
                    };
                    self.push(value.into());
                }
            }

            idx += 1;
        }

        let res = self.pop();
        Ok(res)
    }
}
