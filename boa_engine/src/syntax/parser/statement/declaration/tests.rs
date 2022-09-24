use crate::syntax::{
    ast::{
        expression::literal::Literal,
        statement::declaration::{Declaration, DeclarationList},
    },
    parser::tests::{check_invalid, check_parser},
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "var a = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    let mut interner = Interner::default();
    check_parser(
        "var yield = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::from_identifier(
                interner
                    .get_or_intern_static("yield", utf16!("yield"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "var await = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::from_identifier(
                interner
                    .get_or_intern_static("await", utf16!("await"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    let mut interner = Interner::default();
    check_parser(
        "var a=5;",
        vec![DeclarationList::Var(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "var a;",
        vec![DeclarationList::Var(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                None,
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "var a = 5, b, c = 6;",
        vec![DeclarationList::Var(
            vec![
                Declaration::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(5).into()),
                ),
                Declaration::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    None,
                ),
                Declaration::from_identifier(
                    interner.get_or_intern_static("c", utf16!("c")).into(),
                    Some(Literal::from(6).into()),
                ),
            ]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "let a = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    let mut interner = Interner::default();
    check_parser(
        "let yield = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                interner
                    .get_or_intern_static("yield", utf16!("yield"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "let await = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                interner
                    .get_or_intern_static("await", utf16!("await"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    let mut interner = Interner::default();
    check_parser(
        "let a=5;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "let a;",
        vec![DeclarationList::Let(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                None,
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "let a = 5, b, c = 6;",
        vec![DeclarationList::Let(
            vec![
                Declaration::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(5).into()),
                ),
                Declaration::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    None,
                ),
                Declaration::from_identifier(
                    interner.get_or_intern_static("c", utf16!("c")).into(),
                    Some(Literal::from(6).into()),
                ),
            ]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "const a = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    let mut interner = Interner::default();
    check_parser(
        "const yield = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner
                    .get_or_intern_static("yield", utf16!("yield"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "const await = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner
                    .get_or_intern_static("await", utf16!("await"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    let mut interner = Interner::default();
    check_parser(
        "const a=5;",
        vec![DeclarationList::Const(
            vec![Declaration::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .into(),
        )
        .into()],
        interner,
    );
}

/// Checks empty `const` declaration parsing.
#[test]
fn empty_const_declaration() {
    check_invalid("const a;");
}

/// Checks multiple `const` declarations.
#[test]
fn multiple_const_declaration() {
    let mut interner = Interner::default();
    check_parser(
        "const a = 5, c = 6;",
        vec![DeclarationList::Const(
            vec![
                Declaration::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(5).into()),
                ),
                Declaration::from_identifier(
                    interner.get_or_intern_static("c", utf16!("c")).into(),
                    Some(Literal::from(6).into()),
                ),
            ]
            .into(),
        )
        .into()],
        interner,
    );
}
