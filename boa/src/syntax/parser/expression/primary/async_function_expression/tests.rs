use crate::syntax::{
    ast::{
        node::{AsyncFunctionExpr, Declaration, DeclarationList, Return, StatementList},
        Const,
    },
    parser::tests::check_parser,
};

/// Checks async expression parsing.
#[test]
fn check_async_expression() {
    check_parser(
        "const add = async function() {
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new(
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
    );
}

#[test]
fn check_nested_async_expression() {
    check_parser(
        "const a = async function() {
            const b = async function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new(
                "a",
                Some(
                    AsyncFunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![DeclarationList::Const(
                            vec![Declaration::new(
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
    );
}
