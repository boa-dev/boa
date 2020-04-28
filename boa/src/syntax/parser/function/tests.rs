use crate::syntax::{
    ast::node::{FormalParameter, Node},
    ast::op::NumOp,
    parser::tests::check_parser,
};

/// Checks basic function declaration parsing.
#[test]
fn check_basic() {
    check_parser(
        "function foo(a) { return a; }",
        &[Node::function_decl(
            "foo",
            vec![FormalParameter::new("a", None, false)],
            Node::StatementList(vec![Node::return_node(Node::local("a"))]),
        )],
    );
}

/// Checks basic function declaration parsing with automatic semicolon insertion.
#[test]
fn check_basic_semicolon_insertion() {
    check_parser(
        "function foo(a) { return a }",
        &[Node::function_decl(
            "foo",
            vec![FormalParameter::new("a", None, false)],
            Node::StatementList(vec![Node::return_node(Node::local("a"))]),
        )],
    );
}

/// Checks functions with empty returns.
#[test]
fn check_empty_return() {
    check_parser(
        "function foo(a) { return; }",
        &[Node::function_decl(
            "foo",
            vec![FormalParameter::new("a", None, false)],
            Node::StatementList(vec![Node::Return(None)]),
        )],
    );
}

/// Checks functions with empty returns without semicolon
#[test]
fn check_empty_return_semicolon_insertion() {
    check_parser(
        "function foo(a) { return }",
        &[Node::function_decl(
            "foo",
            vec![FormalParameter::new("a", None, false)],
            Node::StatementList(vec![Node::Return(None)]),
        )],
    );
}

/// Checks rest operator parsing.
#[test]
fn check_rest_operator() {
    check_parser(
        "function foo(a, ...b) {}",
        &[Node::function_decl(
            "foo",
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, true),
            ],
            Node::StatementList(Vec::new()),
        )],
    );
}

/// Checks an arrow function with only a rest parameter.
#[test]
fn check_arrow_only_rest() {
    check_parser(
        "(...a) => {}",
        &[Node::arrow_function_decl(
            vec![FormalParameter::new("a", None, true)],
            Node::StatementList(Vec::new()),
        )],
    );
}

/// Checks an arrow function with a rest parameter.
#[test]
fn check_arrow_rest() {
    check_parser(
        "(a, b, ...c) => {}",
        &[Node::arrow_function_decl(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
                FormalParameter::new("c", None, true),
            ],
            Node::StatementList(Vec::new()),
        )],
    );
}

/// Checks an arrow function with expression return.
#[test]
fn check_arrow() {
    check_parser(
        "(a, b) => { return a + b; }",
        &[Node::arrow_function_decl(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            Node::StatementList(vec![Node::return_node(Node::bin_op(
                NumOp::Add,
                Node::local("a"),
                Node::local("b"),
            ))]),
        )],
    );
}

/// Checks an arrow function with expression return and automatic semicolon insertion
#[test]
fn check_arrow_semicolon_insertion() {
    check_parser(
        "(a, b) => { return a + b }",
        &[Node::arrow_function_decl(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            Node::StatementList(vec![Node::return_node(Node::bin_op(
                NumOp::Add,
                Node::local("a"),
                Node::local("b"),
            ))]),
        )],
    );
}

/// Checks arrow function with empty return
#[test]
fn check_arrow_epty_return() {
    check_parser(
        "(a, b) => { return; }",
        &[Node::arrow_function_decl(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            Node::StatementList(vec![Node::Return(None)]),
        )],
    );
}

/// Checks an arrow function with empty return, with automatic semicolon insertion.
#[test]
fn check_arrow_empty_return_semicolon_insertion() {
    check_parser(
        "(a, b) => { return }",
        &[Node::arrow_function_decl(
            vec![
                FormalParameter::new("a", None, false),
                FormalParameter::new("b", None, false),
            ],
            Node::StatementList(vec![Node::Return(None)]),
        )],
    );
}
