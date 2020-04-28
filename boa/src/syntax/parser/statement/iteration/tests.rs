use crate::syntax::{
    ast::node::Node,
    ast::op::{AssignOp, BinOp, CompOp, UnaryOp},
    parser::tests::check_parser,
};

/// Checks do-while statement parsing.
#[test]
fn check_do_while() {
    check_parser(
        r#"do {
            a += 1;
        } while (true)"#,
        &[Node::do_while_loop(
            Node::Block(vec![Node::bin_op(
                BinOp::Assign(AssignOp::Add),
                Node::local("a"),
                Node::const_node(1.0),
            )]),
            Node::const_node(true),
        )],
    );
}

// Checks automatic semicolon insertion after do-while.
#[test]
fn check_do_while_semicolon_insertion() {
    check_parser(
        r#"var i = 0;
        do {console.log("hello");} while(i++ < 10) console.log("end");"#,
        &[
            Node::VarDecl(vec![(String::from("i"), Some(Node::const_node(0.0)))]),
            Node::do_while_loop(
                Node::Block(vec![Node::call(
                    Node::get_const_field(Node::local("console"), "log"),
                    vec![Node::const_node("hello")],
                )]),
                Node::bin_op(
                    BinOp::Comp(CompOp::LessThan),
                    Node::unary_op(UnaryOp::IncrementPost, Node::local("i")),
                    Node::const_node(10.0),
                ),
            ),
            Node::call(
                Node::get_const_field(Node::local("console"), "log"),
                vec![Node::const_node("end")],
            ),
        ],
    );
}
