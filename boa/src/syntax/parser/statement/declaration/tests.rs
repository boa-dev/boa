use crate::syntax::{
    ast::{
        node::{
            ConstDecl, ConstDeclList, FunctionDecl, LetDecl, LetDeclList, Node, VarDecl,
            VarDeclList,
        },
        Const,
    },
    parser::tests::{check_invalid, check_parser},
};

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    check_parser(
        "var a = 5;",
        vec![VarDeclList::from(vec![VarDecl::new("a", Some(Const::from(5).into()))]).into()],
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    check_parser(
        "var yield = 5;",
        vec![VarDeclList::from(vec![VarDecl::new("yield", Some(Const::from(5).into()))]).into()],
    );

    check_parser(
        "var await = 5;",
        vec![VarDeclList::from(vec![VarDecl::new("await", Some(Const::from(5).into()))]).into()],
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    check_parser(
        "var a=5;",
        vec![VarDeclList::from(vec![VarDecl::new("a", Some(Const::from(5).into()))]).into()],
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    check_parser(
        "var a;",
        vec![VarDeclList::from(vec![VarDecl::new("a", None)]).into()],
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    check_parser(
        "var a = 5, b, c = 6;",
        vec![VarDeclList::from(vec![
            VarDecl::new("a", Some(Const::from(5).into())),
            VarDecl::new("b", None),
            VarDecl::new("c", Some(Const::from(6).into())),
        ])
        .into()],
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    check_parser(
        "let a = 5;",
        vec![LetDeclList::from(vec![LetDecl::new("a", Node::from(Const::from(5)))]).into()],
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    check_parser(
        "let yield = 5;",
        vec![LetDeclList::from(vec![LetDecl::new("yield", Node::from(Const::from(5)))]).into()],
    );

    check_parser(
        "let await = 5;",
        vec![LetDeclList::from(vec![LetDecl::new("await", Node::from(Const::from(5)))]).into()],
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    check_parser(
        "let a=5;",
        vec![LetDeclList::from(vec![LetDecl::new("a", Node::from(Const::from(5)))]).into()],
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    check_parser(
        "let a;",
        vec![LetDeclList::from(vec![LetDecl::new("a", None)]).into()],
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    check_parser(
        "let a = 5, b, c = 6;",
        vec![LetDeclList::from(vec![
            LetDecl::new("a", Node::from(Const::from(5))),
            LetDecl::new("b", None),
            LetDecl::new("c", Node::from(Const::from(6))),
        ])
        .into()],
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    check_parser(
        "const a = 5;",
        vec![ConstDeclList::from(ConstDecl::new("a", Const::from(5))).into()],
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    check_parser(
        "const yield = 5;",
        vec![ConstDeclList::from(ConstDecl::new("yield", Const::from(5))).into()],
    );

    check_parser(
        "const await = 5;",
        vec![ConstDeclList::from(ConstDecl::new("await", Const::from(5))).into()],
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    check_parser(
        "const a=5;",
        vec![ConstDeclList::from(ConstDecl::new("a", Const::from(5))).into()],
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
        vec![ConstDeclList::from(vec![
            ConstDecl::new("a", Const::from(5)),
            ConstDecl::new("c", Const::from(6)),
        ])
        .into()],
    );
}

/// Function declaration parsing.
#[test]
fn function_declaration() {
    check_parser(
        "function hello() {}",
        vec![FunctionDecl::new(Box::from("hello"), vec![], vec![]).into()],
    );
}

/// Function declaration parsing with keywords.
#[test]
fn function_declaration_keywords() {
    check_parser(
        "function yield() {}",
        vec![FunctionDecl::new(Box::from("yield"), vec![], vec![]).into()],
    );

    check_parser(
        "function await() {}",
        vec![FunctionDecl::new(Box::from("await"), vec![], vec![]).into()],
    );
}
