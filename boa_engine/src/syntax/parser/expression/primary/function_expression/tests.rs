use crate::{
    string::utf16,
    syntax::{
        ast::{
            node::{
                Declaration, DeclarationList, FormalParameterList, FunctionExpr, Return,
                StatementList,
            },
            Const,
        },
        parser::tests::check_parser,
    },
};
use boa_interner::{Interner, Sym};

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
            vec![Declaration::new_with_identifier(
                add,
                Some(
                    FunctionExpr::new::<_, _, StatementList>(
                        Some(add),
                        FormalParameterList::default(),
                        vec![Return::new::<_, _, Option<Sym>>(Const::from(1), None).into()].into(),
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
            vec![Declaration::new_with_identifier(
                a,
                Some(
                    FunctionExpr::new::<_, _, StatementList>(
                        Some(a),
                        FormalParameterList::default(),
                        vec![DeclarationList::Const(
                            vec![Declaration::new_with_identifier(
                                b,
                                Some(
                                    FunctionExpr::new::<_, _, StatementList>(
                                        Some(b),
                                        FormalParameterList::default(),
                                        vec![Return::new::<_, _, Option<Sym>>(
                                            Const::from(1),
                                            None,
                                        )
                                        .into()]
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
                vec![Declaration::new_with_identifier(
                    $interner.get_or_intern_static("add", utf16!("add")),
                    Some(
                        FunctionExpr::new::<_, _, StatementList>(
                            Some($interner.get_or_intern_static($keyword, utf16!($keyword))),
                            FormalParameterList::default(),
                            vec![Return::new::<_, _, Option<Sym>>(Const::from(1), None).into()].into(),
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
