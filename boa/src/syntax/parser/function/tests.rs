use crate::{
    syntax::{
        ast::node::{
            ArrowFunctionDecl, BinOp, Declaration, DeclarationList, FormalParameter, FunctionDecl,
            Identifier, Node, Return,
        },
        ast::op::NumOp,
        parser::{tests::check_parser, Parser},
    },
    Interner,
};

/// Checks basic function declaration parsing.
#[test]
fn check_basic() {
    let mut interner = Interner::new();
    check_parser(
        "function foo(a) { return a; }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new(
                Declaration::new_with_identifier("a", None),
                false,
            )],
            vec![Return::new(Identifier::from("a"), None).into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks if duplicate parameter names are allowed with strict mode off.
#[test]
fn check_duplicates_strict_off() {
    let mut interner = Interner::new();
    check_parser(
        "function foo(a, a) { return a; }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
            ],
            vec![Return::new(Identifier::from("a"), None).into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks if duplicate parameter names are an error with strict mode on.
#[test]
fn check_duplicates_strict_on() {
    let js = "'use strict'; function foo(a, a) {}";
    let mut interner = Interner::new();

    let res = Parser::new(js.as_bytes(), false).parse_all(&mut interner);
    dbg!(&res);
    assert!(res.is_err());
}

/// Checks basic function declaration parsing with automatic semicolon insertion.
#[test]
fn check_basic_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "function foo(a) { return a }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new(
                Declaration::new_with_identifier("a", None),
                false,
            )],
            vec![Return::new(Identifier::from("a"), None).into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks functions with empty returns.
#[test]
fn check_empty_return() {
    let mut interner = Interner::new();
    check_parser(
        "function foo(a) { return; }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new(
                Declaration::new_with_identifier("a", None),
                false,
            )],
            vec![Return::new::<Node, Option<Node>, Option<_>>(None, None).into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks functions with empty returns without semicolon
#[test]
fn check_empty_return_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "function foo(a) { return }",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![FormalParameter::new(
                Declaration::new_with_identifier("a", None),
                false,
            )],
            vec![Return::new::<Node, Option<Node>, Option<_>>(None, None).into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks rest operator parsing.
#[test]
fn check_rest_operator() {
    let mut interner = Interner::new();
    check_parser(
        "function foo(a, ...b) {}",
        vec![FunctionDecl::new(
            Box::from("foo"),
            vec![
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
                FormalParameter::new(Declaration::new_with_identifier("b", None), true),
            ],
            vec![],
        )
        .into()],
        &mut interner,
    );
}

/// Checks an arrow function with only a rest parameter.
#[test]
fn check_arrow_only_rest() {
    let mut interner = Interner::new();
    check_parser(
        "(...a) => {}",
        vec![ArrowFunctionDecl::new(
            vec![FormalParameter::new(
                Declaration::new_with_identifier("a", None),
                true,
            )],
            vec![],
        )
        .into()],
        &mut interner,
    );
}

/// Checks an arrow function with a rest parameter.
#[test]
fn check_arrow_rest() {
    let mut interner = Interner::new();
    check_parser(
        "(a, b, ...c) => {}",
        vec![ArrowFunctionDecl::new(
            vec![
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
                FormalParameter::new(Declaration::new_with_identifier("b", None), false),
                FormalParameter::new(Declaration::new_with_identifier("c", None), true),
            ],
            vec![],
        )
        .into()],
        &mut interner,
    );
}

/// Checks an arrow function with expression return.
#[test]
fn check_arrow() {
    let mut interner = Interner::new();
    check_parser(
        "(a, b) => { return a + b; }",
        vec![ArrowFunctionDecl::new(
            vec![
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
                FormalParameter::new(Declaration::new_with_identifier("b", None), false),
            ],
            vec![Return::new(
                BinOp::new(NumOp::Add, Identifier::from("a"), Identifier::from("b")),
                None,
            )
            .into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks an arrow function with expression return and automatic semicolon insertion
#[test]
fn check_arrow_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "(a, b) => { return a + b }",
        vec![ArrowFunctionDecl::new(
            vec![
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
                FormalParameter::new(Declaration::new_with_identifier("b", None), false),
            ],
            vec![Return::new(
                BinOp::new(NumOp::Add, Identifier::from("a"), Identifier::from("b")),
                None,
            )
            .into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks arrow function with empty return
#[test]
fn check_arrow_epty_return() {
    let mut interner = Interner::new();
    check_parser(
        "(a, b) => { return; }",
        vec![ArrowFunctionDecl::new(
            vec![
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
                FormalParameter::new(Declaration::new_with_identifier("b", None), false),
            ],
            vec![Return::new::<Node, Option<_>, Option<_>>(None, None).into()],
        )
        .into()],
        &mut interner,
    );
}

/// Checks an arrow function with empty return, with automatic semicolon insertion.
#[test]
fn check_arrow_empty_return_semicolon_insertion() {
    let mut interner = Interner::new();
    check_parser(
        "(a, b) => { return }",
        vec![ArrowFunctionDecl::new(
            vec![
                FormalParameter::new(Declaration::new_with_identifier("a", None), false),
                FormalParameter::new(Declaration::new_with_identifier("b", None), false),
            ],
            vec![Return::new::<Node, Option<_>, Option<_>>(None, None).into()],
        )
        .into()],
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = (a) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new(
                            Declaration::new_with_identifier("a", None),
                            false,
                        )],
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
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment_nobrackets() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = (a) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new(
                            Declaration::new_with_identifier("a", None),
                            false,
                        )],
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
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment_noparenthesis() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = a => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new(
                            Declaration::new_with_identifier("a", None),
                            false,
                        )],
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
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment_noparenthesis_nobrackets() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = a => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![FormalParameter::new(
                            Declaration::new_with_identifier("a", None),
                            false,
                        )],
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
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment_2arg() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = (a, b) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new(
                                Declaration::new_with_identifier("a", None),
                                false,
                            ),
                            FormalParameter::new(
                                Declaration::new_with_identifier("b", None),
                                false,
                            ),
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
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment_2arg_nobrackets() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = (a, b) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new(
                                Declaration::new_with_identifier("a", None),
                                false,
                            ),
                            FormalParameter::new(
                                Declaration::new_with_identifier("b", None),
                                false,
                            ),
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
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment_3arg() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = (a, b, c) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new(
                                Declaration::new_with_identifier("a", None),
                                false,
                            ),
                            FormalParameter::new(
                                Declaration::new_with_identifier("b", None),
                                false,
                            ),
                            FormalParameter::new(
                                Declaration::new_with_identifier("c", None),
                                false,
                            ),
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
        &mut interner,
    );
}

#[test]
fn check_arrow_assignment_3arg_nobrackets() {
    let mut interner = Interner::new();
    check_parser(
        "let foo = (a, b, c) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                Identifier::from("foo"),
                Some(
                    ArrowFunctionDecl::new(
                        vec![
                            FormalParameter::new(
                                Declaration::new_with_identifier("a", None),
                                false,
                            ),
                            FormalParameter::new(
                                Declaration::new_with_identifier("b", None),
                                false,
                            ),
                            FormalParameter::new(
                                Declaration::new_with_identifier("c", None),
                                false,
                            ),
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
        &mut interner,
    );
}
