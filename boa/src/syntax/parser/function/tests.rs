use crate::syntax::{
    ast::node::{
        ArrowFunctionDecl, BinOp, Declaration, DeclarationList, FormalParameter, FunctionDecl,
        Identifier, Node, Return,
    },
    ast::op::NumOp,
    parser::{tests::check_parser, Parser},
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

/// Checks for duplicate parameter names.
#[test]
fn check_duplicates() {
    let js = "function foo(a, a) {}";

    let res = Parser::new(js.as_bytes(), false).parse_all();
    dbg!(&res);
    assert!(res.is_err());
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

#[test]
fn check_arrow_assignment() {
    check_parser(
        "let foo = (a) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new("a", None, false)],
                        vec![Return::new::<Node, Option<_>, Option<_>>(
                            Some(Identifier::from("a").into()),
                            None,
                        )
                        .into()],
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
fn check_arrow_assignment_nobrackets() {
    check_parser(
        "let foo = (a) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new("a", None, false)],
                        vec![Return::new::<Node, Option<_>, Option<_>>(
                            Some(Identifier::from("a").into()),
                            None,
                        )
                        .into()],
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
fn check_arrow_assignment_noparenthesis() {
    check_parser(
        "let foo = a => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new("a", None, false)],
                        vec![Return::new::<Node, Option<_>, Option<_>>(
                            Some(Identifier::from("a").into()),
                            None,
                        )
                        .into()],
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
fn check_arrow_assignment_noparenthesis_nobrackets() {
    check_parser(
        "let foo = a => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new("a", None, false)],
                        vec![Return::new::<Node, Option<_>, Option<_>>(
                            Some(Identifier::from("a").into()),
                            None,
                        )
                        .into()],
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
fn check_arrow_assignment_2arg() {
    check_parser(
        "let foo = (a, b) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
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
            )]
            .into(),
        )
        .into()],
    );
}

#[test]
fn check_arrow_assignment_2arg_nobrackets() {
    check_parser(
        "let foo = (a, b) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
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
            )]
            .into(),
        )
        .into()],
    );
}

#[test]
fn check_arrow_assignment_3arg() {
    check_parser(
        "let foo = (a, b, c) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
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
            )]
            .into(),
        )
        .into()],
    );
}

#[test]
fn check_arrow_assignment_3arg_nobrackets() {
    check_parser(
        "let foo = (a, b, c) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
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
            )]
            .into(),
        )
        .into()],
    );
}
