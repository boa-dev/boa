use crate::{
    syntax::{
        ast::{
            node::{Declaration, DeclarationList, FunctionExpr, Return, StatementList},
            Const,
        },
        parser::tests::check_parser,
    },
    Interner,
};

/// Checks async expression parsing.
#[test]
fn check_function_expression() {
    let mut interner = Interner::new();
    check_parser(
        "const add = function() {
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "add",
                Some(
                    FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![Return::new::<_, _, Option<Box<str>>>(Const::from(1), None).into()]
                            .into(),
                    )
                    .into(),
                ),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn check_nested_function_expression() {
    let mut interner = Interner::new();
    check_parser(
        "const a = function() {
            const b = function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "a",
                Some(
                    FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![DeclarationList::Const(
                            vec![Declaration::new_with_identifier(
                                "b",
                                Some(
                                    FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                                        None,
                                        [],
                                        vec![Return::new::<_, _, Option<Box<str>>>(
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
        &mut interner,
    );
}
