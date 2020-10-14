use super::*;
use crate::{syntax::ast::Const, syntax::ast::Node};

#[derive(Debug, Default)]
pub struct Compiler {
    pub(super) instructions: Vec<Instruction>,
}

impl Compiler {
    // Add a new instruction.
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
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
            Node::BinOp(ref op) => op.compile(compiler),
            _ => unimplemented!(),
        }
    }
}
