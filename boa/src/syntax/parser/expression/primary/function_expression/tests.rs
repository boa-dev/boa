use crate::syntax::{
    ast::{
        node::{FunctionExpr, ConstDecl, ConstDeclList, Return, StatementList},
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
            Some(
                FunctionExpr::new::<Option<Box<str>>, _, StatementList>(
                    None,
                    [],
                    vec![Return::new::<_, _, Option<Box<str>>>(Const::from(1), None).into()].into(),
                ),
            ),
        )])
        .into()],
    );
}
