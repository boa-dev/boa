use crate::parser::tests::check_script_parser;
use boa_ast::{
    declaration::{Declaration, LexicalDeclaration, Variable},
    expression::literal::Literal,
    function::{AsyncFunctionExpression, FormalParameterList, FunctionBody},
    statement::Return,
    Span, Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;
use indoc::indoc;

const PSEUDO_LINEAR_POS: boa_ast::LinearPosition = boa_ast::LinearPosition::new(0);
const EMPTY_LINEAR_SPAN: boa_ast::LinearSpan =
    boa_ast::LinearSpan::new(PSEUDO_LINEAR_POS, PSEUDO_LINEAR_POS);

/// Checks async expression parsing.
#[test]
fn check_async_expression() {
    let interner = &mut Interner::default();
    let add = interner.get_or_intern_static("add", utf16!("add"));
    check_script_parser(
        indoc! {"
        const add = async function() {
            return 1;
        };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                add.into(),
                Some(
                    AsyncFunctionExpression::new(
                        Some(add.into()),
                        FormalParameterList::default(),
                        FunctionBody::new(
                            [StatementListItem::Statement(Statement::Return(
                                Return::new(Some(
                                    Literal::new(1, Span::new((2, 12), (2, 13))).into(),
                                )),
                            ))],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        EMPTY_LINEAR_SPAN,
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
    check_script_parser(
        indoc! {"
        const a = async function() {
            const b = async function() {
                return 1;
            };
        };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                a.into(),
                Some(
                    AsyncFunctionExpression::new(
                        Some(a.into()),
                        FormalParameterList::default(),
                        FunctionBody::new(
                            [Declaration::Lexical(LexicalDeclaration::Const(
                                vec![Variable::from_identifier(
                                    b.into(),
                                    Some(
                                        AsyncFunctionExpression::new(
                                            Some(b.into()),
                                            FormalParameterList::default(),
                                            FunctionBody::new(
                                                [Statement::Return(Return::new(Some(
                                                    Literal::new(1, Span::new((3, 16), (3, 17)))
                                                        .into(),
                                                )))
                                                .into()],
                                                PSEUDO_LINEAR_POS,
                                                false,
                                            ),
                                            EMPTY_LINEAR_SPAN,
                                            false,
                                        )
                                        .into(),
                                    ),
                                )]
                                .try_into()
                                .unwrap(),
                            ))
                            .into()],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        EMPTY_LINEAR_SPAN,
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
