use crate::parser::tests::check_parser;
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::literal::Literal,
    function::{FormalParameterList, Function},
    statement::Return,
    Declaration, Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks async expression parsing.
#[test]
fn check_function_expression() {
    let interner = &mut Interner::default();
    let add = interner.get_or_intern_static("add", utf16!("add"));
    check_parser(
        "const add = function() {
            return 1;
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                add.into(),
                Some(
                    Function::new(
                        Some(add.into()),
                        FormalParameterList::default(),
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(Literal::from(1).into())),
                        ))]
                        .into(),
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
    check_parser(
        "const a = function() {
            const b = function() {
                return 1;
            };
        };
        ",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                a.into(),
                Some(
                    Function::new(
                        Some(a.into()),
                        FormalParameterList::default(),
                        vec![Declaration::Lexical(LexicalDeclaration::Const(
                            vec![Variable::from_identifier(
                                b.into(),
                                Some(
                                    Function::new(
                                        Some(b.into()),
                                        FormalParameterList::default(),
                                        vec![StatementListItem::Statement(Statement::Return(
                                            Return::new(Some(Literal::from(1).into())),
                                        ))]
                                        .into(),
                                    )
                                    .into(),
                                ),
                            )]
                            .try_into()
                            .unwrap(),
                        ))
                        .into()]
                        .into(),
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
        ($keyword:literal, $interner:expr) => {
            vec![Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    $interner.get_or_intern_static("add", utf16!("add")).into(),
                    Some(
                        Function::new_with_binding_identifier(
                            Some($interner.get_or_intern_static($keyword, utf16!($keyword)).into()),
                            FormalParameterList::default(),
                            vec![StatementListItem::Statement(Statement::Return(Return::new(Some(Literal::from(1).into()))))].into(),
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
    let ast = genast!("as", interner);
    check_parser("const add = function as() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("async", interner);
    check_parser("const add = function async() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("from", interner);
    check_parser("const add = function from() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("get", interner);
    check_parser("const add = function get() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("meta", interner);
    check_parser("const add = function meta() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("of", interner);
    check_parser("const add = function of() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("set", interner);
    check_parser("const add = function set() { return 1; };", ast, interner);

    let interner = &mut Interner::default();
    let ast = genast!("target", interner);
    check_parser(
        "const add = function target() { return 1; };",
        ast,
        interner,
    );
}
