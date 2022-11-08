// This example demonstrates how to use visitors to modify an AST. Namely, the visitors shown here
// are used to swap the operands of commutable arithmetic operations. For an example which simply
// inspects the AST without modifying it, see symbol_visitor.rs.

use boa_ast::{
    expression::operator::{
        binary::{ArithmeticOp, BinaryOp},
        Binary,
    },
    visitor::{VisitWith, VisitorMut},
    Expression,
};
use boa_engine::Context;
use boa_interner::ToInternedString;
use boa_parser::Parser;
use core::ops::ControlFlow;
use std::{convert::Infallible, fs::File, io::BufReader};

/// Visitor which, when applied to a binary expression, will swap the operands. Use in other
/// circumstances is undefined.
#[derive(Default)]
struct OpExchanger<'ast> {
    lhs: Option<&'ast mut Expression>,
}

impl<'ast> VisitorMut<'ast> for OpExchanger<'ast> {
    type BreakTy = ();

    fn visit_expression_mut(&mut self, node: &'ast mut Expression) -> ControlFlow<Self::BreakTy> {
        if let Some(lhs) = self.lhs.take() {
            core::mem::swap(lhs, node);
            ControlFlow::Break(())
        } else {
            self.lhs = Some(node);
            // we do not traverse into the expression; we are only to be used with a binary op
            ControlFlow::Continue(())
        }
    }
}

/// Visitor which walks the AST and swaps the operands of commutable arithmetic binary expressions.
#[derive(Default)]
struct CommutorVisitor {}

impl<'ast> VisitorMut<'ast> for CommutorVisitor {
    type BreakTy = Infallible;

    fn visit_binary_mut(&mut self, node: &'ast mut Binary) -> ControlFlow<Self::BreakTy> {
        if let BinaryOp::Arithmetic(op) = node.op() {
            match op {
                ArithmeticOp::Add | ArithmeticOp::Mul => {
                    // set up the exchanger and swap lhs and rhs
                    let mut exchanger = OpExchanger::default();
                    assert!(matches!(
                        exchanger.visit_binary_mut(node),
                        ControlFlow::Break(_)
                    ));
                }
                _ => {}
            }
        }
        // traverse further in; there may nested binary operations
        node.visit_with_mut(self)
    }
}

fn main() {
    let mut parser = Parser::new(BufReader::new(
        File::open("boa_examples/scripts/calc.js").unwrap(),
    ));
    let mut ctx = Context::default();

    let mut statements = parser.parse_all(ctx.interner_mut()).unwrap();

    let mut visitor = CommutorVisitor::default();

    assert!(matches!(
        visitor.visit_statement_list_mut(&mut statements),
        ControlFlow::Continue(_)
    ));

    println!("{}", statements.to_interned_string(ctx.interner()));
}
