use crate::syntax::{
    ast::node::Node,
    parser::tests::{check_invalid, check_parser},
};

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    check_parser(
        "var a = 5;",
        vec![Node::var_decl(vec![(
            String::from("a"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    check_parser(
        "var yield = 5;",
        vec![Node::var_decl(vec![(
            String::from("yield"),
            Some(Node::const_node(5)),
        )])],
    );

    check_parser(
        "var await = 5;",
        vec![Node::var_decl(vec![(
            String::from("await"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
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
fn empty_var_declaration() {
    check_parser(
        "var a;",
        vec![Node::var_decl(vec![(String::from("a"), None)])],
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
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
fn let_declaration() {
    check_parser(
        "let a = 5;",
        vec![Node::let_decl(vec![(
            String::from("a"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    check_parser(
        "let yield = 5;",
        vec![Node::let_decl(vec![(
            String::from("yield"),
            Some(Node::const_node(5)),
        )])],
    );

    check_parser(
        "let await = 5;",
        vec![Node::let_decl(vec![(
            String::from("await"),
            Some(Node::const_node(5)),
        )])],
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
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
fn empty_let_declaration() {
    check_parser(
        "let a;",
        vec![Node::let_decl(vec![(String::from("a"), None)])],
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
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
fn const_declaration() {
    check_parser(
        "const a = 5;",
        vec![Node::const_decl(vec![(
            String::from("a"),
            Node::const_node(5),
        )])],
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    check_parser(
        "const yield = 5;",
        vec![Node::const_decl(vec![(
            String::from("yield"),
            Node::const_node(5),
        )])],
    );

    check_parser(
        "const await = 5;",
        vec![Node::const_decl(vec![(
            String::from("await"),
            Node::const_node(5),
        )])],
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
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
fn empty_const_declaration() {
    check_invalid("const a;");
}

/// Checks multiple `const` declarations.
#[test]
fn multiple_const_declaration() {
    check_parser(
        "const a = 5, c = 6;",
        vec![Node::const_decl(vec![
            (String::from("a"), Node::const_node(5)),
            (String::from("c"), Node::const_node(6)),
        ])],
    );
}

/// Function declaration parsing.
#[test]
fn function_declaration() {
    check_parser(
        "function hello() {}",
        vec![Node::function_decl(
            "hello",
            vec![],
            Node::statement_list(vec![]),
        )],
    );
}

/// Function declaration parsing with keywords.
#[test]
fn function_declaration_keywords() {
    check_parser(
        "function yield() {}",
        vec![Node::function_decl(
            "yield",
            vec![],
            Node::statement_list(vec![]),
        )],
    );

    check_parser(
        "function await() {}",
        vec![Node::function_decl(
            "await",
            vec![],
            Node::statement_list(vec![]),
        )],
    );
}
