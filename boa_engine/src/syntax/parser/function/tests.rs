use crate::{
    string::utf16,
    syntax::{
        ast::{
            expression::{
                operator::{binary::op::ArithmeticOp, Binary},
                Identifier,
            },
            function::{
                ArrowFunction, FormalParameter, FormalParameterList, FormalParameterListFlags,
                Function,
            },
            statement::{
                declaration::{Declaration, DeclarationList},
                Return, StatementList,
            },
            Expression,
        },
        parser::tests::check_parser,
        Parser,
    },
    Context,
};
use boa_interner::Interner;

/// Checks basic function declaration parsing.
#[test]
fn check_basic() {
    let mut interner = Interner::default();
    check_parser(
        "function foo(a) { return a; }",
        vec![Function::new(
            Some(interner.get_or_intern_static("foo", utf16!("foo"))),
            FormalParameterList {
                parameters: Box::new([FormalParameter::new(
                    Declaration::from_identifier(
                        interner.get_or_intern_static("a", utf16!("a")).into(),
                        None,
                    ),
                    false,
                )]),
                flags: FormalParameterListFlags::default(),
                length: 1,
            },
            vec![Return::new(
                Some(Identifier::from(interner.get_or_intern_static("a", utf16!("a"))).into()),
                None,
            )
            .into()]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks if duplicate parameter names are allowed with strict mode off.
#[test]
fn check_duplicates_strict_off() {
    let mut interner = Interner::default();
    check_parser(
        "function foo(a, a) { return a; }",
        vec![Function::new(
            interner.get_or_intern_static("foo", utf16!("foo")).into(),
            FormalParameterList {
                parameters: Box::new([
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                ]),
                flags: FormalParameterListFlags::default()
                    .union(FormalParameterListFlags::HAS_DUPLICATES),
                length: 2,
            },
            vec![Return::new(
                Some(Identifier::from(interner.get_or_intern_static("a", utf16!("a"))).into()),
                None,
            )
            .into()]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks if duplicate parameter names are an error with strict mode on.
#[test]
fn check_duplicates_strict_on() {
    let js = "'use strict'; function foo(a, a) {}";
    let mut context = Context::default();

    let res = Parser::new(js.as_bytes()).parse_all(&mut context);
    assert!(res.is_err());
}

/// Checks basic function declaration parsing with automatic semicolon insertion.
#[test]
fn check_basic_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        "function foo(a) { return a }",
        vec![Function::new(
            interner.get_or_intern_static("foo", utf16!("foo")).into(),
            FormalParameterList {
                parameters: Box::new([FormalParameter::new(
                    Declaration::from_identifier(
                        interner.get_or_intern_static("a", utf16!("a")).into(),
                        None,
                    ),
                    false,
                )]),
                flags: FormalParameterListFlags::default(),
                length: 1,
            },
            vec![Return::new(
                Some(Identifier::from(interner.get_or_intern_static("a", utf16!("a"))).into()),
                None,
            )
            .into()]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks functions with empty returns.
#[test]
fn check_empty_return() {
    let mut interner = Interner::default();
    check_parser(
        "function foo(a) { return; }",
        vec![Function::new(
            interner.get_or_intern_static("foo", utf16!("foo")).into(),
            FormalParameterList {
                parameters: Box::new([FormalParameter::new(
                    Declaration::from_identifier(
                        interner.get_or_intern_static("a", utf16!("a")).into(),
                        None,
                    ),
                    false,
                )]),
                flags: FormalParameterListFlags::default(),
                length: 1,
            },
            vec![Return::new(None, None).into()].into(),
        )
        .into()],
        interner,
    );
}

/// Checks functions with empty returns without semicolon
#[test]
fn check_empty_return_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        "function foo(a) { return }",
        vec![Function::new(
            interner.get_or_intern_static("foo", utf16!("foo")).into(),
            FormalParameterList {
                parameters: Box::new([FormalParameter::new(
                    Declaration::from_identifier(
                        interner.get_or_intern_static("a", utf16!("a")).into(),
                        None,
                    ),
                    false,
                )]),
                flags: FormalParameterListFlags::default(),
                length: 1,
            },
            vec![Return::new(None, None).into()].into(),
        )
        .into()],
        interner,
    );
}

/// Checks rest operator parsing.
#[test]
fn check_rest_operator() {
    let mut interner = Interner::default();
    check_parser(
        "function foo(a, ...b) {}",
        vec![Function::new(
            interner.get_or_intern_static("foo", utf16!("foo")).into(),
            FormalParameterList {
                parameters: Box::new([
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("b", utf16!("b")).into(),
                            None,
                        ),
                        true,
                    ),
                ]),
                flags: FormalParameterListFlags::empty()
                    .union(FormalParameterListFlags::HAS_REST_PARAMETER),
                length: 1,
            },
            StatementList::default(),
        )
        .into()],
        interner,
    );
}

/// Checks an arrow function with only a rest parameter.
#[test]
fn check_arrow_only_rest() {
    let mut interner = Interner::default();
    check_parser(
        "(...a) => {}",
        vec![Expression::from(ArrowFunction::new(
            None,
            FormalParameterList {
                parameters: Box::new([FormalParameter::new(
                    Declaration::from_identifier(
                        interner.get_or_intern_static("a", utf16!("a")).into(),
                        None,
                    ),
                    true,
                )]),
                flags: FormalParameterListFlags::empty()
                    .union(FormalParameterListFlags::HAS_REST_PARAMETER),
                length: 0,
            },
            StatementList::default(),
        ))
        .into()],
        interner,
    );
}

/// Checks an arrow function with a rest parameter.
#[test]
fn check_arrow_rest() {
    let mut interner = Interner::default();
    check_parser(
        "(a, b, ...c) => {}",
        vec![Expression::from(ArrowFunction::new(
            None,
            FormalParameterList {
                parameters: Box::new([
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("b", utf16!("b")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("c", utf16!("c")).into(),
                            None,
                        ),
                        true,
                    ),
                ]),
                flags: FormalParameterListFlags::empty()
                    .union(FormalParameterListFlags::HAS_REST_PARAMETER),
                length: 2,
            },
            StatementList::default(),
        ))
        .into()],
        interner,
    );
}

/// Checks an arrow function with expression return.
#[test]
fn check_arrow() {
    let mut interner = Interner::default();
    check_parser(
        "(a, b) => { return a + b; }",
        vec![Expression::from(ArrowFunction::new(
            None,
            FormalParameterList {
                parameters: Box::new([
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("b", utf16!("b")).into(),
                            None,
                        ),
                        false,
                    ),
                ]),
                flags: FormalParameterListFlags::default(),
                length: 2,
            },
            vec![Return::new(
                Some(
                    Binary::new(
                        ArithmeticOp::Add.into(),
                        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                        Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
                    )
                    .into(),
                ),
                None,
            )
            .into()]
            .into(),
        ))
        .into()],
        interner,
    );
}

/// Checks an arrow function with expression return and automatic semicolon insertion
#[test]
fn check_arrow_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        "(a, b) => { return a + b }",
        vec![Expression::from(ArrowFunction::new(
            None,
            FormalParameterList {
                parameters: Box::new([
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("b", utf16!("b")).into(),
                            None,
                        ),
                        false,
                    ),
                ]),
                flags: FormalParameterListFlags::default(),
                length: 2,
            },
            vec![Return::new(
                Some(
                    Binary::new(
                        ArithmeticOp::Add.into(),
                        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                        Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
                    )
                    .into(),
                ),
                None,
            )
            .into()]
            .into(),
        ))
        .into()],
        interner,
    );
}

/// Checks arrow function with empty return
#[test]
fn check_arrow_epty_return() {
    let mut interner = Interner::default();
    check_parser(
        "(a, b) => { return; }",
        vec![Expression::from(ArrowFunction::new(
            None,
            FormalParameterList {
                parameters: Box::new([
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("b", utf16!("b")).into(),
                            None,
                        ),
                        false,
                    ),
                ]),
                flags: FormalParameterListFlags::default(),
                length: 2,
            },
            vec![Return::new(None, None).into()].into(),
        ))
        .into()],
        interner,
    );
}

/// Checks an arrow function with empty return, with automatic semicolon insertion.
#[test]
fn check_arrow_empty_return_semicolon_insertion() {
    let mut interner = Interner::default();
    check_parser(
        "(a, b) => { return }",
        vec![Expression::from(ArrowFunction::new(
            None,
            FormalParameterList {
                parameters: Box::new([
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("a", utf16!("a")).into(),
                            None,
                        ),
                        false,
                    ),
                    FormalParameter::new(
                        Declaration::from_identifier(
                            interner.get_or_intern_static("b", utf16!("b")).into(),
                            None,
                        ),
                        false,
                    ),
                ]),
                flags: FormalParameterListFlags::default(),
                length: 2,
            },
            vec![Return::new(None, None).into()].into(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = (a) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo"))),
                        FormalParameterList {
                            parameters: Box::new([FormalParameter::new(
                                Declaration::from_identifier(
                                    interner.get_or_intern_static("a", utf16!("a")).into(),
                                    None,
                                ),
                                false,
                            )]),
                            flags: FormalParameterListFlags::default(),
                            length: 1,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_nobrackets() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = (a) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("foo", utf16!("foo")).into(),
                Some(
                    ArrowFunction::new(
                        interner.get_or_intern_static("foo", utf16!("foo")).into(),
                        FormalParameterList {
                            parameters: Box::new([FormalParameter::new(
                                Declaration::from_identifier(
                                    interner.get_or_intern_static("a", utf16!("a")).into(),
                                    None,
                                ),
                                false,
                            )]),
                            flags: FormalParameterListFlags::default(),
                            length: 1,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_noparenthesis() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = a => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("foo", utf16!("foo")).into(),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo"))),
                        FormalParameterList {
                            parameters: Box::new([FormalParameter::new(
                                Declaration::from_identifier(
                                    interner.get_or_intern_static("a", utf16!("a")).into(),
                                    None,
                                ),
                                false,
                            )]),
                            flags: FormalParameterListFlags::default(),
                            length: 1,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_noparenthesis_nobrackets() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = a => a;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo"))),
                        FormalParameterList {
                            parameters: Box::new([FormalParameter::new(
                                Declaration::from_identifier(
                                    interner.get_or_intern_static("a", utf16!("a")).into(),
                                    None,
                                ),
                                false,
                            )]),
                            flags: FormalParameterListFlags::default(),
                            length: 1,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_2arg() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = (a, b) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo"))),
                        FormalParameterList {
                            parameters: Box::new([
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("a", utf16!("a")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("b", utf16!("b")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                            ]),
                            flags: FormalParameterListFlags::default(),
                            length: 2,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_2arg_nobrackets() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = (a, b) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo"))),
                        FormalParameterList {
                            parameters: Box::new([
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("a", utf16!("a")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("b", utf16!("b")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                            ]),
                            flags: FormalParameterListFlags::default(),
                            length: 2,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_3arg() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = (a, b, c) => { return a };",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo"))),
                        FormalParameterList {
                            parameters: Box::new([
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("a", utf16!("a")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("b", utf16!("b")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("c", utf16!("c")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                            ]),
                            flags: FormalParameterListFlags::default(),
                            length: 3,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_3arg_nobrackets() {
    let mut interner = Interner::default();
    check_parser(
        "let foo = (a, b, c) => a;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo"))),
                        FormalParameterList {
                            parameters: Box::new([
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("a", utf16!("a")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("b", utf16!("b")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                                FormalParameter::new(
                                    Declaration::from_identifier(
                                        interner.get_or_intern_static("c", utf16!("c")).into(),
                                        None,
                                    ),
                                    false,
                                ),
                            ]),
                            flags: FormalParameterListFlags::default(),
                            length: 3,
                        },
                        vec![Return::new(
                            Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            ),
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
        .into()],
        interner,
    );
}
