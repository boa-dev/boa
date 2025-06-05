use crate::parser::tests::check_script_parser;
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::literal::Literal,
    function::{FormalParameterList, FunctionBody, FunctionExpression},
    statement::Return,
    Declaration, Span, Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;
use indoc::indoc;

const PSEUDO_LINEAR_POS: boa_ast::LinearPosition = boa_ast::LinearPosition::new(0);

/// Checks async expression parsing.
#[test]
fn check_function_expression() {
    let interner = &mut Interner::default();
    let add = interner.get_or_intern_static("add", utf16!("add"));
    check_script_parser(
        indoc! {"
        const add = function() {
            return 1;
        };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                add.into(),
                Some(
                    FunctionExpression::new(
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
                        None,
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
fn check_nested_function_expression() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    let b = interner.get_or_intern_static("b", utf16!("b"));
    check_script_parser(
        indoc! {"
        const a = function() {
            const b = function() {
                return 1;
            };
        };
        "},
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                a.into(),
                Some(
                    FunctionExpression::new(
                        Some(a.into()),
                        FormalParameterList::default(),
                        FunctionBody::new(
                            [Declaration::Lexical(LexicalDeclaration::Const(
                                vec![Variable::from_identifier(
                                    b.into(),
                                    Some(
                                        FunctionExpression::new(
                                            Some(b.into()),
                                            FormalParameterList::default(),
                                            FunctionBody::new(
                                                [StatementListItem::Statement(Statement::Return(
                                                    Return::new(Some(
                                                        Literal::new(
                                                            1,
                                                            Span::new((3, 16), (3, 17)),
                                                        )
                                                        .into(),
                                                    )),
                                                ))],
                                                PSEUDO_LINEAR_POS,
                                                false,
                                            ),
                                            None,
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
                        None,
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
fn check_function_non_reserved_keyword() {
    macro_rules! genast {
        ($keyword:literal, $interner:expr, $span:expr) => {
            vec![Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    $interner.get_or_intern_static("add", utf16!("add")).into(),
                    Some(
                        FunctionExpression::new(
                            Some($interner.get_or_intern_static($keyword, utf16!($keyword)).into()),
                            FormalParameterList::default(),
                            FunctionBody::new(
                                [StatementListItem::Statement(
                                    Statement::Return(
                                        Return::new(
                                            Some(Literal::new(1, $span).into())
                                        )
                                    )
                                )],
                                PSEUDO_LINEAR_POS,
                                false,
                            ),
                            None,
                            true,
                        )
                        .into(),
                    ),
                )]
                .try_into().unwrap(),
            ))
            .into()]
        };
    }

    let interner = &mut Interner::default();
    let ast = genast!("as", interner, Span::new((1, 36), (1, 37)));
    check_script_parser("const add = function as() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("async", interner, Span::new((1, 39), (1, 40)));
    check_script_parser("const add = function async() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("from", interner, Span::new((1, 38), (1, 39)));
    check_script_parser("const add = function from() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("get", interner, Span::new((1, 37), (1, 38)));
    check_script_parser("const add = function get() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("meta", interner, Span::new((1, 38), (1, 39)));
    check_script_parser("const add = function meta() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("of", interner, Span::new((1, 36), (1, 37)));
    check_script_parser("const add = function of() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("set", interner, Span::new((1, 37), (1, 38)));
    check_script_parser("const add = function set() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("target", interner, Span::new((1, 40), (1, 41)));
    check_script_parser(
        "const add = function target() { return 1; };",
        ast,
        interner,
    );
}
