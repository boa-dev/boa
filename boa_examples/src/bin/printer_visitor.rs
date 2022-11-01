use boa_engine::syntax::ast::visitor::{VisitWith, Visitor};
use boa_engine::syntax::ast::{Declaration, Statement, StatementList, StatementListItem};
use boa_engine::syntax::Parser;
use boa_engine::Context;
use std::convert::Infallible;
use std::fs::File;
use std::io::BufReader;
use core::ops::ControlFlow;

#[derive(Debug, Clone, Default)]
struct PrinterVisitor {
    indent: String,
}

impl<'ast> Visitor<'ast> for PrinterVisitor {
    type BreakTy = Infallible;

    fn visit_statement_list(&mut self, node: &'ast StatementList) -> ControlFlow<Self::BreakTy> {
        println!(
            "{}StatementList (strict: {}) {{",
            self.indent,
            node.strict()
        );
        self.indent.push(' ');
        let res = node.visit_with(self);
        self.indent.pop();
        println!("{}}}", self.indent);
        res
    }

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        print!("{}StatementListItem: ", self.indent);
        self.indent.push(' ');
        let res = node.visit_with(self);
        self.indent.pop();
        res
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        println!("Statement: {:?}", node);
        ControlFlow::Continue(())
    }

    fn visit_declaration(&mut self, node: &'ast Declaration) -> ControlFlow<Self::BreakTy> {
        println!("Declaration: {:?}", node);
        ControlFlow::Continue(())
    }
}

fn main() {
    let mut parser = Parser::new(BufReader::new(
        File::open("boa_examples/scripts/calctest.js").unwrap(),
    ));
    let mut ctx = Context::default();

    let statements = parser.parse_all(&mut ctx).unwrap();

    let mut visitor = PrinterVisitor::default();

    visitor.visit_statement_list(&statements);
}
