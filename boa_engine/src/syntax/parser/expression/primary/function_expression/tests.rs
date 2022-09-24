use crate::syntax::{
    ast::{
        expression::literal::Literal,
        function::{FormalParameterList, Function},
        statement::{
            declaration::{Declaration, DeclarationList},
            Return,
        },
    },
    parser::tests::check_parser,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks async expression parsing.
#[test]
fn check_function_expression() {
    let mut interner = Interner::default();
    let add = interner.get_or_intern_static("add", utf16!("add"));
    check_parser(
        "const add = function() {
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                add.into(),
                Some(
                    Function::new(
                        Some(add),
                        FormalParameterList::default(),
                        vec![Return::new(Some(Literal::from(1).into()), None).into()].into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_nested_function_expression() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    let b = interner.get_or_intern_static("b", utf16!("b"));
    check_parser(
        "const a = function() {
            const b = function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                a.into(),
                Some(
                    Function::new(
                        Some(a),
                        FormalParameterList::default(),
                        vec![DeclarationList::Const(
                            vec![Declaration::from_identifier(
                                b.into(),
                                Some(
                                    Function::new(
                                        Some(b),
                                        FormalParameterList::default(),
                                        vec![
                                            Return::new(Some(Literal::from(1).into()), None).into()
                                        ]
                                        .into(),
                                    )
                                    .into(),
                                ),
                            )]
                            .into(),
                        )
                        .into()]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_function_non_reserved_keyword() {
    macro_rules! genast {
        ($keyword:literal, $interner:expr) => {
            vec![DeclarationList::Const(
                vec![Declaration::from_identifier(
                    $interner.get_or_intern_static("add", utf16!("add")).into(),
                    Some(
                        Function::new(
                            Some($interner.get_or_intern_static($keyword, utf16!($keyword))),
                            FormalParameterList::default(),
                            vec![Return::new(Some(Literal::from(1).into()), None).into()].into(),
                        )
                        .into(),
                    ),
                )]
                .into(),
            )
            .into()]
        };
    }

    let mut interner = Interner::default();
    let ast = genast!("as", interner);
    check_parser("const add = function as() { return 1; };", ast, interner);

    let mut interner = Interner::default();
    let ast = genast!("async", interner);
    check_parser("const add = function async() { return 1; };", ast, interner);

    let mut interner = Interner::default();
    let ast = genast!("from", interner);
    check_parser("const add = function from() { return 1; };", ast, interner);

    let mut interner = Interner::default();
    let ast = genast!("get", interner);
    check_parser("const add = function get() { return 1; };", ast, interner);

    let mut interner = Interner::default();
    let ast = genast!("meta", interner);
    check_parser("const add = function meta() { return 1; };", ast, interner);

    let mut interner = Interner::default();
    let ast = genast!("of", interner);
    check_parser("const add = function of() { return 1; };", ast, interner);

    let mut interner = Interner::default();
    let ast = genast!("set", interner);
    check_parser("const add = function set() { return 1; };", ast, interner);

    let mut interner = Interner::default();
    let ast = genast!("target", interner);
    check_parser(
        "const add = function target() { return 1; };",
        ast,
        interner,
    );
}
