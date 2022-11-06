use crate::parser::tests::{check_invalid, check_parser};
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::{
        operator::{binary::ArithmeticOp, Binary},
        Identifier,
    },
    function::{
        ArrowFunction, FormalParameter, FormalParameterList, FormalParameterListFlags, Function,
    },
    statement::Return,
    Declaration, Expression, Statement, StatementList, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks basic function declaration parsing.
#[test]
fn check_basic() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);

    check_parser(
        "function foo(a) { return a; }",
        vec![Declaration::Function(Function::new(
            Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(Some(
                    Identifier::from(interner.get_or_intern_static("a", utf16!("a"))).into(),
                )),
            ))]
            .into(),
        ))
        .into()],
        interner,
    );
}

/// Checks if duplicate parameter names are allowed with strict mode off.
#[test]
fn check_duplicates_strict_off() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
    ]);
    assert_eq!(
        params.flags(),
        FormalParameterListFlags::default().union(FormalParameterListFlags::HAS_DUPLICATES)
    );
    assert_eq!(params.length(), 2);
    check_parser(
        "function foo(a, a) { return a; }",
        vec![Declaration::Function(Function::new(
            Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(Some(
                    Identifier::from(interner.get_or_intern_static("a", utf16!("a"))).into(),
                )),
            ))]
            .into(),
        ))
        .into()],
        interner,
    );
}

/// Checks if duplicate parameter names are an error with strict mode on.
#[test]
fn check_duplicates_strict_on() {
    check_invalid("'use strict'; function foo(a, a) {}");
}

/// Checks basic function declaration parsing with automatic semicolon insertion.
#[test]
fn check_basic_semicolon_insertion() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);

    check_parser(
        "function foo(a) { return a }",
        vec![Declaration::Function(Function::new(
            Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(Some(
                    Identifier::from(interner.get_or_intern_static("a", utf16!("a"))).into(),
                )),
            ))]
            .into(),
        ))
        .into()],
        interner,
    );
}

/// Checks functions with empty returns.
#[test]
fn check_empty_return() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);
    check_parser(
        "function foo(a) { return; }",
        vec![Declaration::Function(Function::new(
            Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(None),
            ))]
            .into(),
        ))
        .into()],
        interner,
    );
}

/// Checks functions with empty returns without semicolon
#[test]
fn check_empty_return_semicolon_insertion() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);
    check_parser(
        "function foo(a) { return }",
        vec![Declaration::Function(Function::new(
            Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(None),
            ))]
            .into(),
        ))
        .into()],
        interner,
    );
}

/// Checks rest operator parsing.
#[test]
fn check_rest_operator() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            true,
        ),
    ]);
    assert_eq!(
        params.flags(),
        FormalParameterListFlags::empty().union(FormalParameterListFlags::HAS_REST_PARAMETER)
    );
    assert_eq!(params.length(), 1);
    check_parser(
        "function foo(a, ...b) {}",
        vec![Declaration::Function(Function::new(
            Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
            params,
            StatementList::default(),
        ))
        .into()],
        interner,
    );
}

/// Checks an arrow function with only a rest parameter.
#[test]
fn check_arrow_only_rest() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        true,
    ));
    assert_eq!(
        params.flags(),
        FormalParameterListFlags::empty().union(FormalParameterListFlags::HAS_REST_PARAMETER)
    );
    assert_eq!(params.length(), 0);
    check_parser(
        "(...a) => {}",
        vec![Statement::Expression(Expression::from(ArrowFunction::new(
            None,
            params,
            StatementList::default(),
        )))
        .into()],
        interner,
    );
}

/// Checks an arrow function with a rest parameter.
#[test]
fn check_arrow_rest() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("c", utf16!("c")).into(), None),
            true,
        ),
    ]);
    assert_eq!(
        params.flags(),
        FormalParameterListFlags::empty().union(FormalParameterListFlags::HAS_REST_PARAMETER)
    );
    assert_eq!(params.length(), 2);
    check_parser(
        "(a, b, ...c) => {}",
        vec![Statement::Expression(Expression::from(ArrowFunction::new(
            None,
            params,
            StatementList::default(),
        )))
        .into()],
        interner,
    );
}

/// Checks an arrow function with expression return.
#[test]
fn check_arrow() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
    ]);
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 2);
    check_parser(
        "(a, b) => { return a + b; }",
        vec![Statement::Expression(Expression::from(ArrowFunction::new(
            None,
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(Some(
                    Binary::new(
                        ArithmeticOp::Add.into(),
                        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                        Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
                    )
                    .into(),
                )),
            ))]
            .into(),
        )))
        .into()],
        interner,
    );
}

/// Checks an arrow function with expression return and automatic semicolon insertion
#[test]
fn check_arrow_semicolon_insertion() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
    ]);
    check_parser(
        "(a, b) => { return a + b }",
        vec![Statement::Expression(Expression::from(ArrowFunction::new(
            None,
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(Some(
                    Binary::new(
                        ArithmeticOp::Add.into(),
                        Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                        Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
                    )
                    .into(),
                )),
            ))]
            .into(),
        )))
        .into()],
        interner,
    );
}

/// Checks arrow function with empty return
#[test]
fn check_arrow_epty_return() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
    ]);
    check_parser(
        "(a, b) => { return; }",
        vec![Statement::Expression(Expression::from(ArrowFunction::new(
            None,
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(None),
            ))]
            .into(),
        )))
        .into()],
        interner,
    );
}

/// Checks an arrow function with empty return, with automatic semicolon insertion.
#[test]
fn check_arrow_empty_return_semicolon_insertion() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
    ]);
    check_parser(
        "(a, b) => { return }",
        vec![Statement::Expression(Expression::from(ArrowFunction::new(
            None,
            params,
            vec![StatementListItem::Statement(Statement::Return(
                Return::new(None),
            ))]
            .into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);
    check_parser(
        "let foo = (a) => { return a };",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_nobrackets() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);
    check_parser(
        "let foo = (a) => a;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("foo", utf16!("foo")).into(),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_noparenthesis() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);
    check_parser(
        "let foo = a => { return a };",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("foo", utf16!("foo")).into(),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_noparenthesis_nobrackets() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
        false,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 1);
    check_parser(
        "let foo = a => a;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_2arg() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
    ]);
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 2);
    check_parser(
        "let foo = (a, b) => { return a };",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_2arg_nobrackets() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
    ]);
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 2);
    check_parser(
        "let foo = (a, b) => a;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_3arg() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("c", utf16!("c")).into(), None),
            false,
        ),
    ]);
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 3);
    check_parser(
        "let foo = (a, b, c) => { return a };",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_arrow_assignment_3arg_nobrackets() {
    let interner = &mut Interner::default();
    let params = FormalParameterList::from(vec![
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("a", utf16!("a")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("b", utf16!("b")).into(), None),
            false,
        ),
        FormalParameter::new(
            Variable::from_identifier(interner.get_or_intern_static("c", utf16!("c")).into(), None),
            false,
        ),
    ]);
    assert_eq!(params.flags(), FormalParameterListFlags::default());
    assert_eq!(params.length(), 3);
    check_parser(
        "let foo = (a, b, c) => a;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(interner.get_or_intern_static("foo", utf16!("foo"))),
                Some(
                    ArrowFunction::new(
                        Some(interner.get_or_intern_static("foo", utf16!("foo")).into()),
                        params,
                        vec![StatementListItem::Statement(Statement::Return(
                            Return::new(Some(
                                Identifier::new(interner.get_or_intern_static("a", utf16!("a")))
                                    .into(),
                            )),
                        ))]
                        .into(),
                    )
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}
