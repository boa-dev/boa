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
            Node::BinOp(ref op) => op.compile(compiler),
            Node::Const(Const::Int(num)) => compiler.add_instruction(Instruction::Int32(num)),
            _ => unimplemented!(),
        }
    }
}
