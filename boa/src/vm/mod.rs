use crate::{Context, Value};

pub(crate) mod compilation;
pub(crate) mod instructions;

pub use compilation::Compiler;
pub use instructions::Instruction;

// Virtual Machine.
#[derive(Debug)]
pub struct VM<'a> {
    ctx: &'a mut Context,
    instructions: Vec<Instruction>,
    stack: Vec<Value>,
    stack_pointer: usize,
}

impl<'a> VM<'a> {
    pub fn new(compiler: Compiler, ctx: &'a mut Context) -> Self {
        VM {
            ctx,
            instructions: compiler.instructions,
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

    pub fn run(&mut self) -> super::Result<Value> {
        let mut idx = 0;

        while idx < self.instructions.len() {
            match self.instructions[idx] {
                Instruction::Int32(i) => self.push(Value::Integer(i)),
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
            }

            idx += 1;
        }

        let res = self.pop();
        Ok(res)
    }
}
