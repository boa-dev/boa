use crate::{
    syntax::{
        ast::{
            node::{Declaration, DeclarationList, Node},
            Const,
        },
        parser::tests::{check_invalid, check_parser},
    },
    Interner,
};

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "var a = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::new_with_identifier(
                "a",
                Some(Const::from(5).into()),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    let mut interner = Interner::new();
    check_parser(
        "var yield = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::new_with_identifier(
                "yield",
                Some(Const::from(5).into()),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );

    let mut interner = Interner::new();
    check_parser(
        "var await = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::new_with_identifier(
                "await",
                Some(Const::from(5).into()),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    let mut interner = Interner::new();
    check_parser(
        "var a=5;",
        vec![DeclarationList::Var(
            vec![Declaration::new_with_identifier(
                "a",
                Some(Const::from(5).into()),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "var a;",
        vec![DeclarationList::Var(vec![Declaration::new_with_identifier("a", None)].into()).into()],
        &mut interner,
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "var a = 5, b, c = 6;",
        vec![DeclarationList::Var(
            vec![
                Declaration::new_with_identifier("a", Some(Const::from(5).into())),
                Declaration::new_with_identifier("b", None),
                Declaration::new_with_identifier("c", Some(Const::from(6).into())),
            ]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "let a = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                "a",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    let mut interner = Interner::new();
    check_parser(
        "let yield = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                "yield",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );

    let mut interner = Interner::new();
    check_parser(
        "let await = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                "await",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    let mut interner = Interner::new();
    check_parser(
        "let a=5;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                "a",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "let a;",
        vec![DeclarationList::Let(vec![Declaration::new_with_identifier("a", None)].into()).into()],
        &mut interner,
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "let a = 5, b, c = 6;",
        vec![DeclarationList::Let(
            vec![
                Declaration::new_with_identifier("a", Node::from(Const::from(5))),
                Declaration::new_with_identifier("b", None),
                Declaration::new_with_identifier("c", Node::from(Const::from(6))),
            ]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    let mut interner = Interner::new();
    check_parser(
        "const a = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "a",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    let mut interner = Interner::new();
    check_parser(
        "const yield = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "yield",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );

    let mut interner = Interner::new();
    check_parser(
        "const await = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "await",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    let mut interner = Interner::new();
    check_parser(
        "const a=5;",
        vec![DeclarationList::Const(
            vec![Declaration::new_with_identifier(
                "a",
                Node::from(Const::from(5)),
            )]
            .into(),
        )
        .into()],
        &mut interner,
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
    let mut interner = Interner::new();
    check_parser(
        "const a = 5, c = 6;",
        vec![DeclarationList::Const(
            vec![
                Declaration::new_with_identifier("a", Node::from(Const::from(5))),
                Declaration::new_with_identifier("c", Node::from(Const::from(6))),
            ]
            .into(),
        )
        .into()],
        &mut interner,
    );
}
