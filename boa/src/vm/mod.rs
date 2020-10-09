use crate::{Context, Value};

pub(crate) mod compilation;
pub(crate) mod instructions;

pub use compilation::Compiler;
pub use instructions::Instruction;

// === Execution
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

    pub fn run(&mut self) -> super::Result<Value> {
        let mut idx = 0;

        while idx < self.instructions.len() {
            match self.instructions[idx] {
                Instruction::Int32(i) => self.stack.push(Value::Integer(i)),
                Instruction::Add => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.add(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Sub => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.sub(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Mul => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.mul(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Div => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.div(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Pow => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.pow(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Mod => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.rem(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::BitAnd => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.bitand(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::BitOr => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.bitor(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::BitXor => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.bitxor(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Shl => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.shl(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Shr => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.shr(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::UShr => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.ushr(&r, self.ctx)?;

                    self.stack.push(val);
                }
                Instruction::Eq => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.equals(&r, self.ctx)?;

                    self.stack.push(val.into());
                }
                Instruction::NotEq => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = !l.equals(&r, self.ctx)?;

                    self.stack.push(val.into());
                }
                Instruction::StrictEq => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.strict_equals(&r);

                    self.stack.push(val.into());
                }
                Instruction::StrictNotEq => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = !l.strict_equals(&r);

                    self.stack.push(val.into());
                }
                Instruction::Gt => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.ge(&r, self.ctx)?;

                    self.stack.push(val.into());
                }
                Instruction::Ge => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.ge(&r, self.ctx)?;

                    self.stack.push(val.into());
                }
                Instruction::Lt => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.lt(&r, self.ctx)?;

                    self.stack.push(val.into());
                }
                Instruction::Le => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();
                    let val = l.le(&r, self.ctx)?;

                    self.stack.push(val.into());
                }
                Instruction::In => {
                    let r = self.stack.pop().unwrap();
                    let l = self.stack.pop().unwrap();

                    if !r.is_object() {
                        return self.ctx.throw_type_error(format!(
                            "right-hand side of 'in' should be an object, got {}",
                            r.get_type().as_str()
                        ));
                    }
                    let key = l.to_property_key(self.ctx)?;
                    let val = self.ctx.has_property(&r, &key);

                    self.stack.push(val.into());
                }
                Instruction::InstanceOf => {
                    let r = self.stack.pop().unwrap();
                    let _l = self.stack.pop().unwrap();
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

        let res = self.stack.pop().unwrap();
        Ok(res)
    }
}
