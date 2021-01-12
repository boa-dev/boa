use crate::syntax::{
    ast::node::{
        AsyncArrowFunctionDecl, BinOp, FormalParameter, Identifier, LetDecl, LetDeclList, Node,
        Return,
    },
    ast::op::NumOp,
    parser::tests::check_parser,
};

/// Checks an arrow function with expression return.
#[test]
fn check_async_arrow_bare() {
    check_parser(
        "async (a, b) => { return a + b; }",
        vec![AsyncArrowFunctionDecl::new(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            vec![Return::new(
                BinOp::new(NumOp::Add, Identifier::from("a"), Identifier::from("b")),
                None,
            )
            .into()],
        )
        .into()],
    );
}

#[test]
fn check_async_arrow_assignment() {
    check_parser(
        "let foo = async (a) => { return a };",
        vec![LetDeclList::from(vec![LetDecl::new::<_, Option<Node>>(
            Identifier::from("foo"),
            Some(
                AsyncArrowFunctionDecl::new(
                    vec![FormalParameter::new("a", None, false)],
                    vec![Return::new::<Node, Option<_>, Option<_>>(
                        Some(Identifier::from("a").into()),
                        None,
                    )
                    .into()],
                )
                .into(),
            ),
        )])
        .into()],
    );
}

#[test]
fn check_async_arrow_assignment_noparenthesis() {
    check_parser(
        "let foo = async a => { return a };",
        vec![LetDeclList::from(vec![LetDecl::new::<_, Option<Node>>(
            Identifier::from("foo"),
            Some(
                AsyncArrowFunctionDecl::new(
                    vec![FormalParameter::new("a", None, false)],
                    vec![Return::new::<Node, Option<_>, Option<_>>(
                        Some(Identifier::from("a").into()),
                        None,
                    )
                    .into()],
                )
                .into(),
            ),
        )])
        .into()],
    );
}
