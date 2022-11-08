use std::convert::TryInto;

use crate::parser::tests::{check_invalid, check_parser};
use boa_ast::{
    declaration::{VarDeclaration, Variable},
    expression::{literal::Literal, Identifier},
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    property::PropertyName,
    statement::{Block, Catch, ErrorHandler, Finally, Try},
    Statement, StatementListItem,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_inline_with_empty_try_catch() {
    let interner = &mut Interner::default();
    check_parser(
        "try { } catch(e) {}",
        vec![Statement::Try(Try::new(
            Block::default(),
            ErrorHandler::Catch(Catch::new(
                Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                Block::default(),
            )),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_var_decl_inside_try() {
    let interner = &mut Interner::default();
    check_parser(
        "try { var x = 1; } catch(e) {}",
        vec![Statement::Try(Try::new(
            vec![Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(Literal::from(1).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into()]
            .into(),
            ErrorHandler::Catch(Catch::new(
                Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                Block::default(),
            )),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_var_decl_inside_catch() {
    let interner = &mut Interner::default();
    check_parser(
        "try { var x = 1; } catch(e) { var x = 1; }",
        vec![Statement::Try(Try::new(
            vec![Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(Literal::from(1).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into()]
            .into(),
            ErrorHandler::Catch(Catch::new(
                Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                vec![Statement::Var(VarDeclaration(
                    vec![Variable::from_identifier(
                        interner.get_or_intern_static("x", utf16!("x")).into(),
                        Some(Literal::from(1).into()),
                    )]
                    .try_into()
                    .unwrap(),
                ))
                .into()]
                .into(),
            )),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_empty_try_catch_finally() {
    let interner = &mut Interner::default();
    check_parser(
        "try {} catch(e) {} finally {}",
        vec![Statement::Try(Try::new(
            Block::default(),
            ErrorHandler::Full(
                Catch::new(
                    Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                    Block::default(),
                ),
                Finally::from(Block::default()),
            ),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_empty_try_finally() {
    check_parser(
        "try {} finally {}",
        vec![Statement::Try(Try::new(
            Block::default(),
            ErrorHandler::Finally(Finally::from(Block::default())),
        ))
        .into()],
        &mut Interner::default(),
    );
}

#[test]
fn check_inline_with_empty_try_var_decl_in_finally() {
    let interner = &mut Interner::default();
    check_parser(
        "try {} finally { var x = 1; }",
        vec![Statement::Try(Try::new(
            Block::default(),
            ErrorHandler::Finally(Finally::from(Block::from(vec![
                StatementListItem::Statement(Statement::Var(VarDeclaration(
                    vec![Variable::from_identifier(
                        interner.get_or_intern_static("x", utf16!("x")).into(),
                        Some(Literal::from(1).into()),
                    )]
                    .try_into()
                    .unwrap(),
                ))),
            ]))),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_empty_try_paramless_catch() {
    let interner = &mut Interner::default();
    check_parser(
        "try {} catch { var x = 1; }",
        vec![Statement::Try(Try::new(
            Block::default(),
            ErrorHandler::Catch(Catch::new(
                None,
                vec![Statement::Var(VarDeclaration(
                    vec![Variable::from_identifier(
                        interner.get_or_intern_static("x", utf16!("x")).into(),
                        Some(Literal::from(1).into()),
                    )]
                    .try_into()
                    .unwrap(),
                ))
                .into()]
                .into(),
            )),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_binding_pattern_object() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_parser(
        "try {} catch ({ a, b: c }) {}",
        vec![Statement::Try(Try::new(
            Block::default(),
            ErrorHandler::Catch(Catch::new(
                Some(
                    Pattern::from(vec![
                        ObjectPatternElement::SingleName {
                            ident: a.into(),
                            name: PropertyName::Literal(a),
                            default_init: None,
                        },
                        ObjectPatternElement::SingleName {
                            ident: interner.get_or_intern_static("c", utf16!("c")).into(),
                            name: PropertyName::Literal(
                                interner.get_or_intern_static("b", utf16!("b")),
                            ),
                            default_init: None,
                        },
                    ])
                    .into(),
                ),
                Block::default(),
            )),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_binding_pattern_array() {
    let interner = &mut Interner::default();
    check_parser(
        "try {} catch ([a, b]) {}",
        vec![Statement::Try(Try::new(
            Block::from(vec![]),
            ErrorHandler::Catch(Catch::new(
                Some(
                    Pattern::from(vec![
                        ArrayPatternElement::SingleName {
                            ident: interner.get_or_intern_static("a", utf16!("a")).into(),
                            default_init: None,
                        },
                        ArrayPatternElement::SingleName {
                            ident: interner.get_or_intern_static("b", utf16!("b")).into(),
                            default_init: None,
                        },
                    ])
                    .into(),
                ),
                Block::default(),
            )),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_catch_with_var_redeclaration() {
    let interner = &mut Interner::default();
    check_parser(
        "try {} catch(e) { var e = 'oh' }",
        vec![Statement::Try(Try::new(
            Block::from(vec![]),
            ErrorHandler::Catch(Catch::new(
                Some(Identifier::new(interner.get_or_intern_static("e", utf16!("e"))).into()),
                vec![Statement::Var(VarDeclaration(
                    vec![Variable::from_identifier(
                        interner.get_or_intern_static("e", utf16!("e")).into(),
                        Some(
                            Literal::from(interner.get_or_intern_static("oh", utf16!("oh"))).into(),
                        ),
                    )]
                    .try_into()
                    .unwrap(),
                ))
                .into()]
                .into(),
            )),
        ))
        .into()],
        interner,
    );
}

#[test]
fn check_inline_invalid_catch() {
    check_invalid("try {} catch");
}

#[test]
fn check_inline_invalid_catch_without_closing_paren() {
    check_invalid("try {} catch(e {}");
}

#[test]
fn check_inline_invalid_catch_parameter() {
    check_invalid("try {} catch(1) {}");
}

#[test]
fn check_invalid_try_no_catch_finally() {
    check_invalid("try {} let a = 10;");
}

#[test]
fn check_invalid_catch_with_empty_paren() {
    check_invalid("try {} catch() {}");
}

#[test]
fn check_invalid_catch_with_duplicate_params() {
    check_invalid("try {} catch({ a, b: a }) {}");
}

#[test]
fn check_invalid_catch_with_lexical_redeclaration() {
    check_invalid("try {} catch(e) { let e = 'oh' }");
}
