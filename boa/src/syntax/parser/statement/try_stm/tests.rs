use crate::syntax::{
    ast::node::Node,
    parser::tests::{check_invalid, check_parser},
};

#[test]
fn check_inline_with_empty_try_catch() {
    check_parser(
        "try { } catch(e) {}",
        vec![Node::try_node::<_, _, _, _, Node, Node, Node>(
            Node::block(vec![]),
            Node::block(vec![]),
            Node::local("e"),
            None,
        )],
    );
}

#[test]
fn check_inline_with_var_decl_inside_try() {
    check_parser(
        "try { var x = 1; } catch(e) {}",
        vec![Node::try_node::<_, _, _, _, Node, Node, Node>(
            Node::block(vec![Node::var_decl(vec![(
                String::from("x"),
                Some(Node::const_node(1)),
            )])]),
            Node::block(vec![]),
            Node::local("e"),
            None,
        )],
    );
}

#[test]
fn check_inline_with_var_decl_inside_catch() {
    check_parser(
        "try { var x = 1; } catch(e) { var x = 1; }",
        vec![Node::try_node::<_, _, _, _, Node, Node, Node>(
            Node::block(vec![Node::var_decl(vec![(
                String::from("x"),
                Some(Node::const_node(1)),
            )])]),
            Node::block(vec![Node::var_decl(vec![(
                String::from("x"),
                Some(Node::const_node(1)),
            )])]),
            Node::local("e"),
            None,
        )],
    );
}

#[test]
fn check_inline_with_empty_try_catch_finally() {
    check_parser(
        "try {} catch(e) {} finally {}",
        vec![Node::try_node::<_, _, _, _, Node, Node, Node>(
            Node::block(vec![]),
            Node::block(vec![]),
            Node::local("e"),
            Node::block(vec![]),
        )],
    );
}

#[test]
fn check_inline_with_empty_try_finally() {
    check_parser(
        "try {} finally {}",
        vec![Node::try_node::<_, _, _, _, Node, Node, Node>(
            Node::block(vec![]),
            None,
            None,
            Node::block(vec![]),
        )],
    );
}

#[test]
fn check_inline_with_empty_try_var_decl_in_finally() {
    check_parser(
        "try {} finally { var x = 1; }",
        vec![Node::try_node::<_, _, _, _, Node, Node, Node>(
            Node::block(vec![]),
            None,
            None,
            Node::block(vec![Node::var_decl(vec![(
                String::from("x"),
                Some(Node::const_node(1)),
            )])]),
        )],
    );
}

#[test]
fn check_inline_empty_try_paramless_catch() {
    check_parser(
        "try {} catch { var x = 1; }",
        vec![Node::try_node::<_, _, _, _, Node, Node, Node>(
            Node::block(vec![]),
            Node::block(vec![Node::var_decl(vec![(
                String::from("x"),
                Some(Node::const_node(1)),
            )])]),
            None,
            None,
        )],
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
