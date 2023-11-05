use crate::parser::tests::check_script_parser;
use boa_ast::{expression::literal::Literal, statement::Throw, Statement};
use boa_interner::Interner;

#[test]
fn check_throw_parsing() {
    let interner = &mut Interner::default();
    check_script_parser(
        "throw 'error';",
        vec![Statement::Throw(Throw::new(
            Literal::from(interner.get_or_intern("error")).into(),
        ))
        .into()],
        interner,
    );
}
