use crate::syntax::{
    ast::{
        node::{Declaration, DeclarationList, Node},
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
    // Most of these tests seem crazy at first. But they follow
    // a few simple rules:
    //
    // - Let statements stay within their block scope.
    // - Var statements stay within their function scope.
    // - The only time you can redeclare is when you use `var` and `var`.
    // - We always check the innermost block first.
    //
    // This explains most of the tests. For example, `var a; { let a; }`
    // seems like it should be invalid, but because we always check the
    // innermost block first, it ends up evaluating to this: `{ let a; }; var a`.
    // And now the solution is clear: the `let` goes out of scope before
    // the var is declared.
    //
    // I think this entire system makes no sense, and I would never implement
    // it this way if it were up to me. However, this is how all js parsers
    // work, so this is how its implemented.

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
    // Multiple blocks
    check_invalid("let a; { var a } { let a }");
    check_valid("var a; { let a } { let a }");
    check_valid("let a; { let a } { let a }");
    check_valid("var a; { var a } { let a }");
    check_invalid("let a; { var a } { var a }");
    check_valid("var a; { let a } { var a }");
    check_invalid("let a; { let a } { var a }");
    check_valid("var a; { var a } { var a }");

    // Inside a function, everything is simple. The parameters act like `var`
    // declarations, and they have the same scope as the outermost function
    // block. Any surrounding variables do not exist when inside a function.

    // Function scoping
    check_invalid("function f(a) { let a; }");
    check_valid("function f(a) { var a; }");
    check_valid("function f(a) { { let a; } }");
    check_valid("function f(a) { { var a; } }");

    check_valid("let a; function f(a) {}");
    check_valid("var a; function f(a) {}");

    check_valid("let a; function f() { var a; }");
    check_valid("var a; function f() { let a; }");
    check_valid("let a; function f() { let a; }");
    check_valid("var a; function f() { var a; }");

    // Functions names are a bit more complex. They have the scope of `let`, but
    // they can only be redeclared with themselves. So they will conflict with
    // existing `let` and `var` statements. Once again, innermost blocks are
    // evaluated first, so `function f() {}; { var f }` is invalid.

    // Function definitions
    check_invalid("let f; function f() {}");
    check_invalid("var f; function f() {}");
    check_invalid("function f() {}; let f");
    check_invalid("function f() {}; var f");
    check_valid("function f() {}; function f() {}");

    check_valid("{ function f() {} }; let f");
    check_valid("{ function f() {} }; var f");
    check_valid("let f; { function f() {} }");
    check_valid("var f; { function f() {} }");

    check_invalid("{ var f }; function f() {}");
    check_valid("{ let f }; function f() {}");
    check_invalid("function f() {}; { var f }");
    check_valid("function f() {}; { let f }");
}
