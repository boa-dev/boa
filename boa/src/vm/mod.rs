use self::instructions::Instruction;
use crate::{Context, Value};

pub(crate) mod compilation;
pub(crate) mod instructions;

// === Execution
#[derive(Debug)]
pub struct VM<'a> {
    ctx: &'a mut Context,
    instructions: Vec<Instruction>,
    stack: Vec<Value>,
    stack_pointer: usize,
}

impl<'a> VM<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        let instr = ctx.instructions_mut().clone();
        VM {
            ctx,
            instructions: instr,
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

                _ => unimplemented!(),
            }

            idx += 1;
        }

        let res = self.stack.pop().unwrap();
        Ok(res)
    }
}
