use crate::syntax::{
    ast::node::Node,
    parser::tests::{check_invalid, check_parser},
};

/// Checks `var` declaration parsing.
#[test]
fn check_var_declaration() {
    check_parser(
        "var a = 5;",
        vec![Node::var_decl(vec![(
            String::from("a"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn check_var_declaration_no_spaces() {
    check_parser(
        "var a=5;",
        vec![Node::var_decl(vec![(
            String::from("a"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn check_empty_var_declaration() {
    check_parser(
        "var a;",
        vec![Node::var_decl(vec![(String::from("a"), None)])],
    );
}

/// Checks multiple `var` declarations.
#[test]
fn check_multiple_var_declaration() {
    check_parser(
        "var a = 5, b, c = 6;",
        vec![Node::var_decl(vec![
            (String::from("a"), Some(Node::const_node(5))),
            (String::from("b"), None),
            (String::from("c"), Some(Node::const_node(6))),
        ])],
    );
}

/// Checks `let` declaration parsing.
#[test]
fn check_let_declaration() {
    check_parser(
        "let a = 5;",
        vec![Node::let_decl(vec![(
            String::from("a"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn check_let_declaration_no_spaces() {
    check_parser(
        "let a=5;",
        vec![Node::let_decl(vec![(
            String::from("a"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn check_empty_let_declaration() {
    check_parser(
        "let a;",
        vec![Node::let_decl(vec![(String::from("a"), None)])],
    );
}

/// Checks multiple `let` declarations.
#[test]
fn check_multiple_let_declaration() {
    check_parser(
        "let a = 5, b, c = 6;",
        vec![Node::let_decl(vec![
            (String::from("a"), Some(Node::const_node(5))),
            (String::from("b"), None),
            (String::from("c"), Some(Node::const_node(6))),
        ])],
    );
}

/// Checks `const` declaration parsing.
#[test]
fn check_const_declaration() {
    check_parser(
        "const a = 5;",
        vec![Node::const_decl(vec![(
            String::from("a"),
            Node::const_node(5),
        )])],
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn check_const_declaration_no_spaces() {
    check_parser(
        "const a=5;",
        vec![Node::const_decl(vec![(
            String::from("a"),
            Node::const_node(5),
        )])],
    );
}

/// Checks empty `const` declaration parsing.
#[test]
fn check_empty_const_declaration() {
    check_invalid("const a;");
}

/// Checks multiple `const` declarations.
#[test]
fn check_multiple_const_declaration() {
    check_parser(
        "const a = 5, c = 6;",
        vec![Node::const_decl(vec![
            (String::from("a"), Node::const_node(5)),
            (String::from("c"), Node::const_node(6)),
        ])],
    );
}
