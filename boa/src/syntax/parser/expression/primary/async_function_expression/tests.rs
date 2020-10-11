use crate::syntax::{
    ast::{
        node::{AsyncFunctionExpr, ConstDecl, ConstDeclList, Return, StatementList},
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
        vec![ConstDeclList::from(vec![ConstDecl::new(
            "add",
            Some(
                AsyncFunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                    None,
                    [],
                    vec![Return::new::<_, _, Option<Box<str>>>(Const::from(1), None).into()].into(),
                ),
            ),
        )])
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
        vec![ConstDeclList::from(vec![ConstDecl::new(
            "a",
            Some(
                AsyncFunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                    None,
                    [],
                    vec![ConstDeclList::from(vec![ConstDecl::new(
                        "b",
                        Some(
                            AsyncFunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                                None,
                                [],
                                vec![Return::new::<_, _, Option<Box<str>>>(Const::from(1), None)
                                    .into()]
                                .into(),
                            ),
                        ),
                    )])
                    .into()]
                    .into(),
                ),
            ),
        )])
        .into()],
    );
}
