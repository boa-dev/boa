use super::*;

use crate::syntax::ast::{constant::Const, node::*, Node};

// this..?
#[derive(Debug, Default)]
pub(crate) struct Compiler {
    res: Vec<In>,
    next_free: u8,
}

// or maybe..
// `impl CodeGen for BinOp` ?
trait CodeGen {
    fn compile(&self);
}

impl Compiler {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn compile(&mut self, list: &StatementList) -> Vec<In> {
        for stmt in list.statements() {
            self.compile_node(stmt);
        }

        std::mem::replace(&mut self.res, Vec::new())
    }

    fn compile_node(&mut self, node: &Node) {
        match node {
            Node::Const(Const::Int(x)) => self
                .res
                .push(In::Ld(Reg(self.next_free), Value::integer(*x))),

            Node::Const(Const::Num(x)) => self
                .res
                .push(In::Ld(Reg(self.next_free), Value::number(*x))),

            Node::BinOp(node) => {
                let dest = self.next_free;
                let src = dest + 1;

                self.compile_node(node.lhs());
                self.next_free = src;
                self.compile_node(node.rhs());
                self.next_free = dest;

                self.res.push(In::Add {
                    dest: Reg(dest),
                    src: Reg(src),
                });
            }

            Node::ConstDeclList(xs) => {
                for x in xs.as_ref() {
                    self.compile_node(x.init());

                    self.res
                        .push(In::Bind(Reg(self.next_free), x.name().to_owned()));
                    // FIXME: remove .to_owned()
                }
            }

            _ => {
                dbg!(node);
                panic!("unsupported Node");
            }
        }
    }
}
