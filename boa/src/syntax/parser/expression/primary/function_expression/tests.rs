use crate::syntax::{
    ast::{
        node::{Declaration, DeclarationList, FunctionExpr, Return, StatementList},
        Const,
    },
    parser::tests::check_parser,
};

/// Checks async expression parsing.
#[test]
fn check_function_expression() {
    check_parser(
        "const add = function() {
            return 1;
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new(
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
    );
}

#[test]
fn check_nested_function_expression() {
    check_parser(
        "const a = function() {
            const b = function() {
                return 1;
            };
        };
        ",
        vec![DeclarationList::Const(
            vec![Declaration::new(
                "a",
                Some(
                    FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![DeclarationList::Const(
                            vec![Declaration::new(
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
    );
}
