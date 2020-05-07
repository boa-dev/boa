use crate::syntax::{
    ast::node::Node,
    ast::op::{AssignOp, BinOp, BitOp, CompOp, NumOp},
    parser::tests::check_parser,
};

/// Checks numeric operations
#[test]
fn check_numeric_operations() {
    check_parser(
        "a + b",
        vec![Node::bin_op(NumOp::Add, Node::local("a"), Node::local("b"))],
    );
    check_parser(
        "a+1",
        vec![Node::bin_op(
            NumOp::Add,
            Node::local("a"),
            Node::const_node(1),
        )],
    );
    check_parser(
        "a - b",
        vec![Node::bin_op(NumOp::Sub, Node::local("a"), Node::local("b"))],
    );
    check_parser(
        "a-1",
        vec![Node::bin_op(
            NumOp::Sub,
            Node::local("a"),
            Node::const_node(1),
        )],
    );
    check_parser(
        "a / b",
        vec![Node::bin_op(NumOp::Div, Node::local("a"), Node::local("b"))],
    );
    check_parser(
        "a/2",
        vec![Node::bin_op(
            NumOp::Div,
            Node::local("a"),
            Node::const_node(2),
        )],
    );
    check_parser(
        "a * b",
        vec![Node::bin_op(NumOp::Mul, Node::local("a"), Node::local("b"))],
    );
    check_parser(
        "a*2",
        vec![Node::bin_op(
            NumOp::Mul,
            Node::local("a"),
            Node::const_node(2),
        )],
    );
    check_parser(
        "a ** b",
        vec![Node::bin_op(NumOp::Exp, Node::local("a"), Node::local("b"))],
    );
    check_parser(
        "a**2",
        vec![Node::bin_op(
            NumOp::Exp,
            Node::local("a"),
            Node::const_node(2),
        )],
    );
    check_parser(
        "a % b",
        vec![Node::bin_op(NumOp::Mod, Node::local("a"), Node::local("b"))],
    );
    check_parser(
        "a%2",
        vec![Node::bin_op(
            NumOp::Mod,
            Node::local("a"),
            Node::const_node(2),
        )],
    );
}

// Checks complex numeric operations.
#[test]
fn check_complex_numeric_operations() {
    check_parser(
        "a + d*(b-3)+1",
        vec![Node::bin_op(
            NumOp::Add,
            Node::bin_op(
                NumOp::Add,
                Node::local("a"),
                Node::bin_op(
                    NumOp::Mul,
                    Node::local("d"),
                    Node::bin_op(NumOp::Sub, Node::local("b"), Node::const_node(3)),
                ),
            ),
            Node::const_node(1),
        )],
    );
}

/// Checks bitwise operations.
#[test]
fn check_bitwise_operations() {
    check_parser(
        "a & b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::And),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a&b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::And),
            Node::local("a"),
            Node::local("b"),
        )],
    );

    check_parser(
        "a | b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Or),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a|b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Or),
            Node::local("a"),
            Node::local("b"),
        )],
    );

    check_parser(
        "a ^ b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Xor),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a^b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Xor),
            Node::local("a"),
            Node::local("b"),
        )],
    );

    check_parser(
        "a << b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Shl),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a<<b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Shl),
            Node::local("a"),
            Node::local("b"),
        )],
    );

    check_parser(
        "a >> b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Shr),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a>>b",
        vec![Node::bin_op(
            BinOp::Bit(BitOp::Shr),
            Node::local("a"),
            Node::local("b"),
        )],
    );
}

/// Checks assignment operations.
#[test]
fn check_assign_operations() {
    check_parser(
        "a += b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Add),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a -= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Sub),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a *= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Mul),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a **= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Exp),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a /= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Div),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a %= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Mod),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a &= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::And),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a |= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Or),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a ^= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Xor),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a <<= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Shl),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a >>= b",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Shr),
            Node::local("a"),
            Node::local("b"),
        )],
    );
    check_parser(
        "a %= 10 / 2",
        vec![Node::bin_op(
            BinOp::Assign(AssignOp::Mod),
            Node::local("a"),
            Node::bin_op(NumOp::Div, Node::const_node(10), Node::const_node(2)),
        )],
    );
}

#[test]
fn check_relational_operations() {
    check_parser(
        "a < b",
        vec![Node::bin_op(
            BinOp::Comp(CompOp::LessThan),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a > b",
        vec![Node::bin_op(
            BinOp::Comp(CompOp::GreaterThan),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a <= b",
        vec![Node::bin_op(
            BinOp::Comp(CompOp::LessThanOrEqual),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a >= b",
        vec![Node::bin_op(
            BinOp::Comp(CompOp::GreaterThanOrEqual),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "p in o",
        vec![Node::bin_op(
            BinOp::Comp(CompOp::In),
            Node::Local(String::from("p")),
            Node::Local(String::from("o")),
        )],
    );
}
