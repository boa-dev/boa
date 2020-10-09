use super::*;
use crate::{syntax::ast::Node, Context};
use std::result::Result;

#[derive(Debug, Default)]
pub(crate) struct Compiler {
    res: Vec<Instruction>,
    next_free: u8,
}

pub(crate) trait CodeGen {
    fn compile(&self, ctx: &mut Context) -> Result<(), &str>;
}

impl CodeGen for Node {
    fn compile(&self, compiler: &mut Context) -> Result<(), &str> {
        unimplemented!();
    }
}
