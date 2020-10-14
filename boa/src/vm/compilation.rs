use super::*;
use crate::{syntax::ast::Const, syntax::ast::Node, value::RcString};

#[derive(Debug, Default)]
pub struct Compiler {
    pub(super) instructions: Vec<Instruction>,
    pub(super) pool: Vec<RcString>,
}

impl Compiler {
    // Add a new instruction.
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    pub fn add_string_instruction<S>(&mut self, string: S)
    where
        S: Into<RcString>,
    {
        let index = self.pool.len();
        self.add_instruction(Instruction::String(index));
        self.pool.push(string.into());
    }
}

pub(crate) trait CodeGen {
    fn compile(&self, compiler: &mut Compiler);
}

impl CodeGen for Node {
    fn compile(&self, compiler: &mut Compiler) {
        match *self {
            Node::Const(Const::Undefined) => compiler.add_instruction(Instruction::Undefined),
            Node::Const(Const::Null) => compiler.add_instruction(Instruction::Null),
            Node::Const(Const::Bool(value)) => compiler.add_instruction(Instruction::Bool(value)),
            Node::Const(Const::Int(num)) => compiler.add_instruction(Instruction::Int32(num)),
            Node::Const(Const::String(ref string)) => {
                compiler.add_string_instruction(string.clone())
            }
            Node::BinOp(ref op) => op.compile(compiler),
            _ => unimplemented!(),
        }
    }
}
