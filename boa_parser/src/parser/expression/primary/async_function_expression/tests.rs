use crate::parser::tests::check_parser;
use boa_ast::{
    declaration::{Declaration, LexicalDeclaration, Variable},
    expression::literal::Literal,
    function::{AsyncFunction, FormalParameterList},
    statement::Return,
    Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks async expression parsing.
#[test]
fn check_async_expression() {
    let interner = &mut Interner::default();
    let add = interner.get_or_intern_static("add", utf16!("add"));
    check_parser(
        "const add = async function() {
            return 1;
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                add.into(),
                Some(
                    AsyncFunction::new(
                        Some(add.into()),
                        FormalParameterList::default(),
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(Literal::from(1).into())),
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
fn check_nested_async_expression() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    let b = interner.get_or_intern_static("b", utf16!("b"));
    check_parser(
        "const a = async function() {
            const b = async function() {
                return 1;
            };
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                a.into(),
                Some(
                    AsyncFunction::new(
                        Some(a.into()),
                        FormalParameterList::default(),
                        vec![Declaration::Lexical(LexicalDeclaration::Const(
                            vec![Variable::from_identifier(
                                b.into(),
                                Some(
                                    AsyncFunction::new(
                                        Some(b.into()),
                                        FormalParameterList::default(),
                                        vec![Statement::Return(Return::new(Some(
                                            Literal::from(1).into(),
                                        )))
                                        .into()]
                                        .into(),
                                        false,
                                    )
                                    .into(),
                                ),
                            )]
                            .try_into()
                            .unwrap(),
                        ))
                        .into()]
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
