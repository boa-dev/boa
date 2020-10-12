use super::*;
use crate::{syntax::ast::Const, syntax::ast::Node, Context};

#[derive(Debug, Default)]
pub(crate) struct Compiler {
    res: Vec<Instruction>,
    next_free: u8,
}

pub(crate) trait CodeGen {
    fn compile(&self, ctx: &mut Context);
}

impl CodeGen for Node {
    fn compile(&self, ctx: &mut Context) {
        match *self {
            Node::BinOp(ref op) => op.compile(ctx),
            Node::Const(Const::Int(num)) => ctx.add_instruction(Instruction::Int32(num)),
            _ => unimplemented!(),
        }
    }
}
