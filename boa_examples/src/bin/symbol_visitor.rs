// This example demonstrates how to use a visitor to perform simple operations over the Javascript
// AST, namely: finding all the `Sym`s present in a script. See commuter_visitor.rs for an example
// which mutates the AST.

use boa_ast::visitor::Visitor;
use boa_engine::Context;
use boa_interner::Sym;
use boa_parser::Parser;
use core::ops::ControlFlow;
use std::{collections::HashSet, convert::Infallible, fs::File, io::BufReader};

#[derive(Debug, Clone, Default)]
struct SymbolVisitor {
    observed: HashSet<Sym>,
}

impl<'ast> Visitor<'ast> for SymbolVisitor {
    type BreakTy = Infallible;

    fn visit_sym(&mut self, node: &'ast Sym) -> ControlFlow<Self::BreakTy> {
        self.observed.insert(*node);
        ControlFlow::Continue(())
    }
}

fn main() {
    let mut parser = Parser::new(BufReader::new(
        File::open("boa_examples/scripts/calc.js").unwrap(),
    ));
    let mut ctx = Context::default();

    let statements = parser.parse_all(ctx.interner_mut()).unwrap();

    let mut visitor = SymbolVisitor::default();

    assert!(matches!(
        visitor.visit_statement_list(&statements),
        ControlFlow::Continue(_)
    ));

    println!(
        "Observed {} unique strings/symbols:",
        visitor.observed.len()
    );
    for sym in visitor.observed {
        println!("  - {}", ctx.interner().resolve(sym).unwrap());
    }
}
