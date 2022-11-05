use crate::parser::tests::check_parser;
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::{literal::Literal, Yield},
    function::{FormalParameterList, Generator},
    Declaration, Expression, Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_generator_function_expression() {
    let interner = &mut Interner::default();
    let gen = interner.get_or_intern_static("gen", utf16!("gen"));
    check_parser(
        "const gen = function*() {
            yield 1;
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                gen.into(),
                Some(
                    Generator::new(
                        Some(gen.into()),
                        FormalParameterList::default(),
                        vec![StatementListItem::Statement(Statement::Expression(
                            Expression::from(Yield::new(Some(Literal::from(1).into()), false)),
                        ))]
                        .into(),
                        false,
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_generator_function_delegate_yield_expression() {
    let interner = &mut Interner::default();
    let gen = interner.get_or_intern_static("gen", utf16!("gen"));
    check_parser(
        "const gen = function*() {
            yield* 1;
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                gen.into(),
                Some(
                    Generator::new(
                        Some(gen.into()),
                        FormalParameterList::default(),
                        vec![StatementListItem::Statement(Statement::Expression(
                            Expression::from(Yield::new(Some(Literal::from(1).into()), true)),
                        ))]
                        .into(),
                        false,
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}
