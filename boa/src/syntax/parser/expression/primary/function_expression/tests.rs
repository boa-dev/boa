use crate::syntax::{
    ast::{
        node::{ConstDecl, ConstDeclList, FunctionExpr, Return, StatementList},
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
        vec![ConstDeclList::from(vec![ConstDecl::new(
            "add",
            Some(FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                None,
                [],
                vec![Return::new::<_, _, Option<Box<str>>>(Const::from(1), None).into()].into(),
            )),
        )])
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
        vec![ConstDeclList::from(vec![ConstDecl::new(
            "a",
            Some(FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                None,
                [],
                vec![ConstDeclList::from(vec![ConstDecl::new(
                    "b",
                    Some(FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                        None,
                        [],
                        vec![Return::new::<_, _, Option<Box<str>>>(Const::from(1), None).into()]
                            .into(),
                    )),
                )])
                .into()]
                .into(),
            )),
        )])
        .into()],
    );
}
