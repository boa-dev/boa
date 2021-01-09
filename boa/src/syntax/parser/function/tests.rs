use crate::syntax::{
    ast::node::{
        ArrowFunctionDecl, BinOp, FormalParameter, FunctionDecl, Identifier, Node, Return, LetDecl, LetDeclList
    },
    ast::op::NumOp,
    parser::tests::check_parser,
};

/// Checks basic function declaration parsing.
#[test]
fn check_basic() {
    check_parser(
        "function foo(a) { return a; }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new(Identifier::from("a"), None).into()],
        )
        .into()],
    );
}

/// Checks basic function declaration parsing with automatic semicolon insertion.
#[test]
fn check_basic_semicolon_insertion() {
    check_parser(
        "function foo(a) { return a }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new(Identifier::from("a"), None).into()],
        )
        .into()],
    );
}

/// Checks functions with empty returns.
#[test]
fn check_empty_return() {
    check_parser(
        "function foo(a) { return; }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new::<Node, Option<Node>, Option<_>>(None, None).into()],
        )
        .into()],
    );
}

/// Checks functions with empty returns without semicolon
#[test]
fn check_empty_return_semicolon_insertion() {
    check_parser(
        "function foo(a) { return }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new("a", None, false)],
            vec![Return::new::<Node, Option<Node>, Option<_>>(None, None).into()],
        )
        .into()],
    );
}

/// Checks rest operator parsing.
#[test]
fn check_rest_operator() {
    check_parser(
        "function foo(a, ...b) {}",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, true),
            ],
            vec![],
        )
        .into()],
    );
}

/// Checks an arrow function with only a rest parameter.
#[test]
fn check_arrow_only_rest() {
    check_parser(
        "(...a) => {}",
        vec![ArrowFunctionDecl::new(vec![FormalParameter::new("a", None, true)], vec![]).into()],
    );
}

/// Checks an arrow function with a rest parameter.
#[test]
fn check_arrow_rest() {
    check_parser(
        "(a, b, ...c) => {}",
        vec![ArrowFunctionDecl::new(
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

/// Checks an arrow function with expression return.
#[test]
fn check_arrow() {
    check_parser(
        "(a, b) => { return a + b; }",
        vec![ArrowFunctionDecl::new(
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

/// Checks an arrow function with expression return and automatic semicolon insertion
#[test]
fn check_arrow_semicolon_insertion() {
    check_parser(
        "(a, b) => { return a + b }",
        vec![ArrowFunctionDecl::new(
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

/// Checks arrow function with empty return
#[test]
fn check_arrow_epty_return() {
    check_parser(
        "(a, b) => { return; }",
        vec![ArrowFunctionDecl::new(
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
fn check_arrow_empty_return_semicolon_insertion() {
    check_parser(
        "(a, b) => { return }",
        vec![ArrowFunctionDecl::new(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            vec![Return::new::<Node, Option<_>, Option<_>>(None, None).into()],
        )
        .into()],
    );
}

/// Checks an arrow function assignment.
#[test]
fn check_arrow_assignment() {
    check_parser(
        "let foo = a => { return a };",
        vec![LetDeclList::from(
            vec![
                LetDecl::new::<_, Option<Node>>(
                     Identifier::from("foo"),
                     Some(ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new("a", None, false),
                        ],
                        vec![Return::new::<Node, Option<_>, Option<_>>(Some(Identifier::from("a").into()), None).into()],
                    ).into())
                )
            ]
        ).into()]
    );
}

/// Checks an arrow function assignment without {}.
#[test]
fn check_arrow_assignment_nobrackets() {
    check_parser(
        "let foo = a => return a;",
        vec![LetDeclList::from(
            vec![
                LetDecl::new::<_, Option<Node>>(
                     Identifier::from("foo"),
                     Some(ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new("a", None, false),
                        ],
                        vec![Return::new::<Node, Option<_>, Option<_>>(Some(Identifier::from("a").into()), None).into()],
                    ).into())
                )
            ]
        ).into()]
    );
}

/// Checks an arrow function assignment with parenthesis.
#[test]
fn check_arrow_assignment_parenthesis() {
    check_parser(
        "let foo = (a) => { return a };",
        vec![LetDeclList::from(
            vec![
                LetDecl::new::<_, Option<Node>>(
                     Identifier::from("foo"),
                     Some(ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new("a", None, false),
                        ],
                        vec![Return::new::<Node, Option<_>, Option<_>>(Some(Identifier::from("a").into()), None).into()],
                    ).into())
                )
            ]
        ).into()]
    );
}

/// Checks an arrow function assignment with parenthesis without {}.
#[test]
fn check_arrow_assignment_parenthesis_nobrackets() {
    check_parser(
        "let foo = (a) => return a;",
        vec![LetDeclList::from(
            vec![
                LetDecl::new::<_, Option<Node>>(
                     Identifier::from("foo"),
                     Some(ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new("a", None, false),
                        ],
                        vec![Return::new::<Node, Option<_>, Option<_>>(Some(Identifier::from("a").into()), None).into()],
                    ).into())
                )
            ]
        ).into()]
    );
}

/// Checks an arrow function assignment with 2 arguments with parenthesis.
#[test]
fn check_arrow_assignment_2arg_parenthesis() {
    check_parser(
        "let foo = (a, b) => { return a };",
        vec![LetDeclList::from(
            vec![
                LetDecl::new::<_, Option<Node>>(
                     Identifier::from("foo"),
                     Some(ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new("a", None, false),
                            FormalParameter::new("b", None, false)
                        ],
                        vec![Return::new::<Node, Option<_>, Option<_>>(Some(Identifier::from("a").into()), None).into()],
                    ).into())
                )
            ]
        ).into()]
    );
}

/// Checks an arrow function assignment with 2 arguments with parenthesis.
#[test]
fn check_arrow_assignment_2arg_parenthesis_nobrackets() {
    check_parser(
        "let foo = (a, b) => return a;",
        vec![LetDeclList::from(
            vec![
                LetDecl::new::<_, Option<Node>>(
                     Identifier::from("foo"),
                     Some(ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new("a", None, false),
                            FormalParameter::new("b", None, false)
                        ],
                        vec![Return::new::<Node, Option<_>, Option<_>>(Some(Identifier::from("a").into()), None).into()],
                    ).into())
                )
            ]
        ).into()]
    );
}
