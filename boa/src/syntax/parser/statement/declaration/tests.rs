use crate::syntax::{
    ast::{
        node::{Block, Declaration, DeclarationList, Node},
        Const,
    },
    parser::tests::{check_invalid, check_parser, check_valid},
};

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    check_parser(
        "var a = 5;",
        vec![
            DeclarationList::Var(vec![Declaration::new("a", Some(Const::from(5).into()))].into())
                .into(),
        ],
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    check_parser(
        "var yield = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::new("yield", Some(Const::from(5).into()))].into(),
        )
        .into()],
    );

    check_parser(
        "var await = 5;",
        vec![DeclarationList::Var(
            vec![Declaration::new("await", Some(Const::from(5).into()))].into(),
        )
        .into()],
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    check_parser(
        "var a=5;",
        vec![
            DeclarationList::Var(vec![Declaration::new("a", Some(Const::from(5).into()))].into())
                .into(),
        ],
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    check_parser(
        "var a;",
        vec![DeclarationList::Var(vec![Declaration::new("a", None)].into()).into()],
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    check_parser(
        "var a = 5, b, c = 6;",
        vec![DeclarationList::Var(
            vec![
                Declaration::new("a", Some(Const::from(5).into())),
                Declaration::new("b", None),
                Declaration::new("c", Some(Const::from(6).into())),
            ]
            .into(),
        )
        .into()],
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    check_parser(
        "let a = 5;",
        vec![
            DeclarationList::Let(vec![Declaration::new("a", Node::from(Const::from(5)))].into())
                .into(),
        ],
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    check_parser(
        "let yield = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::new("yield", Node::from(Const::from(5)))].into(),
        )
        .into()],
    );

    check_parser(
        "let await = 5;",
        vec![DeclarationList::Let(
            vec![Declaration::new("await", Node::from(Const::from(5)))].into(),
        )
        .into()],
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    check_parser(
        "let a=5;",
        vec![
            DeclarationList::Let(vec![Declaration::new("a", Node::from(Const::from(5)))].into())
                .into(),
        ],
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    check_parser(
        "let a;",
        vec![DeclarationList::Let(vec![Declaration::new("a", None)].into()).into()],
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    check_parser(
        "let a = 5, b, c = 6;",
        vec![DeclarationList::Let(
            vec![
                Declaration::new("a", Node::from(Const::from(5))),
                Declaration::new("b", None),
                Declaration::new("c", Node::from(Const::from(6))),
            ]
            .into(),
        )
        .into()],
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    check_parser(
        "const a = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::new("a", Node::from(Const::from(5)))].into(),
        )
        .into()],
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    check_parser(
        "const yield = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::new("yield", Node::from(Const::from(5)))].into(),
        )
        .into()],
    );

    check_parser(
        "const await = 5;",
        vec![DeclarationList::Const(
            vec![Declaration::new("await", Node::from(Const::from(5)))].into(),
        )
        .into()],
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    check_parser(
        "const a=5;",
        vec![DeclarationList::Const(
            vec![Declaration::new("a", Node::from(Const::from(5)))].into(),
        )
        .into()],
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
    check_parser(
        "const a = 5, c = 6;",
        vec![DeclarationList::Const(
            vec![
                Declaration::new("a", Node::from(Const::from(5))),
                Declaration::new("c", Node::from(Const::from(6))),
            ]
            .into(),
        )
        .into()],
    );
}

/// Checks for redeclaration errors.
#[test]
fn redeclaration_errors() {
    // Throughout this test, we test four combinations:
    //
    // - let-var
    // - var-let
    // - let-let
    // - var-var
    //
    // Depending on the block/function scope, these are sometimes valid, and sometimes not valid.

    // Same scope
    check_invalid("let a; var a;");
    check_invalid("var a; let a;");
    check_invalid("let a; let a;");
    check_valid("var a; var a;");
    // First in block
    check_valid("{ let a }; var a;");
    check_invalid("{ var a }; let a;");
    check_valid("{ let a }; let a;");
    check_valid("{ var a }; var a;");
    // Second in block
    check_invalid("let a; { var a }");
    check_valid("var a; { let a }");
    check_valid("let a; { let a }");
    check_valid("var a; { var a }");
    panic!();
}
