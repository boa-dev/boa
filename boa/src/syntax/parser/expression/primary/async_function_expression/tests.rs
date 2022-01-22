use crate::{
    syntax::{
        ast::{
            node::{AsyncFunctionExpr, Declaration, DeclarationList, Return, StatementList},
            Const,
        },
        parser::tests::check_parser,
    },
    Interner,
};

/// Checks async expression parsing.
#[test]
fn check_async_expression() {
    let mut interner = Interner::new();
    check_parser(
        "const add = async function() {
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "add",
                Some(
                    AsyncFunctionExpr::new::<Option<Box<str>>, _, StatementList>(
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
fn check_nested_async_expression() {
    let mut interner = Interner::new();
    check_parser(
        "const a = async function() {
            const b = async function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "a",
                Some(
                    AsyncFunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![DeclarationList::Const(
                            vec![Declaration::new_with_identifier(
                                "b",
                                Some(
                                    AsyncFunctionExpr::new::<Option<Box<str>>, _, StatementList>(
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
