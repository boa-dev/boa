use crate::syntax::{
    ast::node::{
        AsyncArrowFunctionDecl, BinOp, FormalParameter, Identifier, LetDecl, LetDeclList, Node,
        Return,
    },
    ast::op::NumOp,
    parser::tests::{check_invalid, check_parser},
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
fn check_async_arrow_only_rest() {
    check_parser(
        "async (...a) => {}",
        vec![
            AsyncArrowFunctionDecl::new(vec![FormalParameter::new("a", None, true)], vec![]).into(),
        ],
    );
}

#[test]
fn check_async_arrow_rest() {
    check_parser(
        "async (a, b, ...c) => {}",
        vec![AsyncArrowFunctionDecl::new(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
                FormalParameter::new("c", None, true),
            ],
            vec![],
        )
        .into()],
    );
}

/// Checks an arrow function with expression return and automatic semicolon insertion
#[test]
fn check_async_arrow_semicolon_insertion() {
    check_parser(
        "async (a, b) => { return a + b }",
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
fn check_async_arrow_empty_return() {
    check_parser(
        "async(a, b) => { return; }",
        vec![AsyncArrowFunctionDecl::new(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            vec![Return::new::<Node, Option<_>, Option<_>>(None, None).into()],
        )
        .into()],
    );
}

/// Checks an arrow function with empty return, with automatic semicolon insertion.
#[test]
fn check_async_arrow_empty_return_semicolon_insertion() {
    check_parser(
        "async (a, b) => { return }",
        vec![AsyncArrowFunctionDecl::new(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            vec![Return::new::<Node, Option<_>, Option<_>>(None, None).into()],
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

#[test]
fn check_async_arrow_assignment_nobrackets() {
    check_parser(
        "let foo = async (a) => a;",
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
fn check_async_arrow_assignment_noparenthesis_nobrackets() {
    check_parser(
        "let foo = async a => a;",
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
fn check_async_arrow_assignment_2arg() {
    check_parser(
        "let foo = async (a, b) => { return a };",
        vec![LetDeclList::from(vec![LetDecl::new::<_, Option<Node>>(
            Identifier::from("foo"),
            Some(
                AsyncArrowFunctionDecl::new(
                    vec![
                        FormalParameter::new("a", None, false),
                        FormalParameter::new("b", None, false),
                    ],
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
fn check_async_arrow_assignment_2arg_nobrackets() {
    check_parser(
        "let foo = async (a, b) => a;",
        vec![LetDeclList::from(vec![LetDecl::new::<_, Option<Node>>(
            Identifier::from("foo"),
            Some(
                AsyncArrowFunctionDecl::new(
                    vec![
                        FormalParameter::new("a", None, false),
                        FormalParameter::new("b", None, false),
                    ],
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
fn check_async_arrow_assignment_3arg() {
    check_parser(
        "let foo = async (a, b, c) => { return a };",
        vec![LetDeclList::from(vec![LetDecl::new::<_, Option<Node>>(
            Identifier::from("foo"),
            Some(
                AsyncArrowFunctionDecl::new(
                    vec![
                        FormalParameter::new("a", None, false),
                        FormalParameter::new("b", None, false),
                        FormalParameter::new("c", None, false),
                    ],
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
fn check_async_arrow_assignment_3arg_nobrackets() {
    check_parser(
        "let foo = async (a, b, c) => a;",
        vec![LetDeclList::from(vec![LetDecl::new::<_, Option<Node>>(
            Identifier::from("foo"),
            Some(
                AsyncArrowFunctionDecl::new(
                    vec![
                        FormalParameter::new("a", None, false),
                        FormalParameter::new("b", None, false),
                        FormalParameter::new("c", None, false),
                    ],
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
fn check_async_arrow_unique_formal_param() {
    check_invalid("async (a, a) => a;");
}
