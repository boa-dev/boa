use crate::syntax::{
    ast::{
        expression::{literal::Literal, Identifier},
        pattern::{Pattern, PatternArrayElement, PatternObjectElement},
        property::PropertyName,
        statement::{
            declaration::{Declaration, DeclarationList},
            Block, Catch, Finally, Try,
        },
    },
    parser::tests::{check_invalid, check_parser},
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn check_inline_with_empty_try_catch() {
    let mut interner = Interner::default();
    check_parser(
        "try { } catch(e) {}",
        vec![Try::new(
            Block::default(),
            Some(Catch::new(
                Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                Block::default(),
            )),
            None,
        )
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_var_decl_inside_try() {
    let mut interner = Interner::default();
    check_parser(
        "try { var x = 1; } catch(e) {}",
        vec![Try::new(
            vec![DeclarationList::Var(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(Literal::from(1).into()),
                )]
                .into(),
            )
            .into()]
            .into(),
            Some(Catch::new(
                Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                Block::default(),
            )),
            None,
        )
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_var_decl_inside_catch() {
    let mut interner = Interner::default();
    check_parser(
        "try { var x = 1; } catch(e) { var x = 1; }",
        vec![Try::new(
            vec![DeclarationList::Var(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(Literal::from(1).into()),
                )]
                .into(),
            )
            .into()]
            .into(),
            Some(Catch::new(
                Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                vec![DeclarationList::Var(
                    vec![Declaration::from_identifier(
                        interner.get_or_intern_static("x", utf16!("x")).into(),
                        Some(Literal::from(1).into()),
                    )]
                    .into(),
                )
                .into()]
                .into(),
            )),
            None,
        )
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_empty_try_catch_finally() {
    let mut interner = Interner::default();
    check_parser(
        "try {} catch(e) {} finally {}",
        vec![Try::new(
            Block::default(),
            Some(Catch::new(
                Some(Identifier::from(interner.get_or_intern_static("e", utf16!("e"))).into()),
                Block::default(),
            )),
            Some(Finally::from(Block::default())),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_empty_try_finally() {
    check_parser(
        "try {} finally {}",
        vec![Try::new(
            Block::default(),
            None,
            Some(Finally::from(Block::default())),
        )
        .into()],
        Interner::default(),
    );
}

#[test]
fn check_inline_with_empty_try_var_decl_in_finally() {
    let mut interner = Interner::default();
    check_parser(
        "try {} finally { var x = 1; }",
        vec![Try::new(
            Block::default(),
            None,
            Some(Finally::from(vec![DeclarationList::Var(
                vec![Declaration::from_identifier(
                    interner.get_or_intern_static("x", utf16!("x")).into(),
                    Some(Literal::from(1).into()),
                )]
                .into(),
            )
            .into()])),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_inline_empty_try_paramless_catch() {
    let mut interner = Interner::default();
    check_parser(
        "try {} catch { var x = 1; }",
        vec![Try::new(
            Block::default(),
            Some(Catch::new(
                None,
                vec![DeclarationList::Var(
                    vec![Declaration::from_identifier(
                        interner.get_or_intern_static("x", utf16!("x")).into(),
                        Some(Literal::from(1).into()),
                    )]
                    .into(),
                )
                .into()]
                .into(),
            )),
            None,
        )
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_binding_pattern_object() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_parser(
        "try {} catch ({ a, b: c }) {}",
        vec![Try::new(
            Block::default(),
            Some(Catch::new(
                Some(
                    Pattern::from(vec![
                        PatternObjectElement::SingleName {
                            ident: a,
                            name: PropertyName::Literal(a),
                            default_init: None,
                        },
                        PatternObjectElement::SingleName {
                            ident: interner.get_or_intern_static("c", utf16!("c")),
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
            None,
        )
        .into()],
        interner,
    );
}

#[test]
fn check_inline_with_binding_pattern_array() {
    let mut interner = Interner::default();
    check_parser(
        "try {} catch ([a, b]) {}",
        vec![Try::new(
            Block::from(vec![]),
            Some(Catch::new(
                Some(
                    Pattern::from(vec![
                        PatternArrayElement::SingleName {
                            ident: interner.get_or_intern_static("a", utf16!("a")),
                            default_init: None,
                        },
                        PatternArrayElement::SingleName {
                            ident: interner.get_or_intern_static("b", utf16!("b")),
                            default_init: None,
                        },
                    ])
                    .into(),
                ),
                Block::default(),
            )),
            None,
        )
        .into()],
        interner,
    );
}

#[test]
fn check_catch_with_var_redeclaration() {
    let mut interner = Interner::default();
    check_parser(
        "try {} catch(e) { var e = 'oh' }",
        vec![Try::new(
            Block::from(vec![]),
            Some(Catch::new(
                Some(Identifier::new(interner.get_or_intern_static("e", utf16!("e"))).into()),
                vec![DeclarationList::Var(
                    vec![Declaration::from_identifier(
                        interner.get_or_intern_static("e", utf16!("e")).into(),
                        Some(
                            Literal::from(interner.get_or_intern_static("oh", utf16!("oh"))).into(),
                        ),
                    )]
                    .into(),
                )
                .into()]
                .into(),
            )),
            None,
        )
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
