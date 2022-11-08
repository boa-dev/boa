use std::convert::TryInto;

use crate::parser::tests::{check_invalid, check_parser};
use boa_ast::{
    declaration::{LexicalDeclaration, VarDeclaration, Variable},
    expression::literal::Literal,
    Declaration, Statement,
};
use boa_interner::Interner;
use boa_macros::utf16;

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "var a = 5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    let interner = &mut Interner::default();
    check_parser(
        "var yield = 5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                interner
                    .get_or_intern_static("yield", utf16!("yield"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "var await = 5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                interner
                    .get_or_intern_static("await", utf16!("await"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_parser(
        "var a=5;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "var a;",
        vec![Statement::Var(VarDeclaration(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                None,
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "var a = 5, b, c = 6;",
        vec![Statement::Var(VarDeclaration(
            vec![
                Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(5).into()),
                ),
                Variable::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    None,
                ),
                Variable::from_identifier(
                    interner.get_or_intern_static("c", utf16!("c")).into(),
                    Some(Literal::from(6).into()),
                ),
            ]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "let a = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    let interner = &mut Interner::default();
    check_parser(
        "let yield = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner
                    .get_or_intern_static("yield", utf16!("yield"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "let await = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner
                    .get_or_intern_static("await", utf16!("await"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_parser(
        "let a=5;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "let a;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                None,
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "let a = 5, b, c = 6;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![
                Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(5).into()),
                ),
                Variable::from_identifier(
                    interner.get_or_intern_static("b", utf16!("b")).into(),
                    None,
                ),
                Variable::from_identifier(
                    interner.get_or_intern_static("c", utf16!("c")).into(),
                    Some(Literal::from(6).into()),
                ),
            ]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    let interner = &mut Interner::default();
    check_parser(
        "const a = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    let interner = &mut Interner::default();
    check_parser(
        "const yield = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner
                    .get_or_intern_static("yield", utf16!("yield"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "const await = 5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner
                    .get_or_intern_static("await", utf16!("await"))
                    .into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_parser(
        "const a=5;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                interner.get_or_intern_static("a", utf16!("a")).into(),
                Some(Literal::from(5).into()),
            )]
            .try_into()
            .unwrap(),
        ))
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
    let interner = &mut Interner::default();
    check_parser(
        "const a = 5, c = 6;",
        vec![Declaration::Lexical(LexicalDeclaration::Const(
            vec![
                Variable::from_identifier(
                    interner.get_or_intern_static("a", utf16!("a")).into(),
                    Some(Literal::from(5).into()),
                ),
                Variable::from_identifier(
                    interner.get_or_intern_static("c", utf16!("c")).into(),
                    Some(Literal::from(6).into()),
                ),
            ]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );
}
