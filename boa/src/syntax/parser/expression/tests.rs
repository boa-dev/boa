use crate::syntax::{
    ast::op::{AssignOp, BitOp, CompOp, NumOp},
    ast::{
        node::{BinOp, Identifier},
        Const,
    },
    parser::tests::check_parser,
};

/// Checks numeric operations
#[test]
fn check_numeric_operations() {
    check_parser(
        "a + b",
        vec![BinOp::new(NumOp::Add, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a+1",
        vec![BinOp::new(NumOp::Add, Identifier::from("a"), Const::from(1)).into()],
    );
    check_parser(
        "a - b",
        vec![BinOp::new(NumOp::Sub, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a-1",
        vec![BinOp::new(NumOp::Sub, Identifier::from("a"), Const::from(1)).into()],
    );
    check_parser(
        "a / b",
        vec![BinOp::new(NumOp::Div, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a/2",
        vec![BinOp::new(NumOp::Div, Identifier::from("a"), Const::from(2)).into()],
    );
    check_parser(
        "a * b",
        vec![BinOp::new(NumOp::Mul, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a*2",
        vec![BinOp::new(NumOp::Mul, Identifier::from("a"), Const::from(2)).into()],
    );
    check_parser(
        "a ** b",
        vec![BinOp::new(NumOp::Exp, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a**2",
        vec![BinOp::new(NumOp::Exp, Identifier::from("a"), Const::from(2)).into()],
    );
    check_parser(
        "a % b",
        vec![BinOp::new(NumOp::Mod, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a%2",
        vec![BinOp::new(NumOp::Mod, Identifier::from("a"), Const::from(2)).into()],
    );
}

// Checks complex numeric operations.
#[test]
fn check_complex_numeric_operations() {
    check_parser(
        "a + d*(b-3)+1",
        vec![BinOp::new(
            NumOp::Add,
            BinOp::new(
                NumOp::Add,
                Identifier::from("a"),
                BinOp::new(
                    NumOp::Mul,
                    Identifier::from("d"),
                    BinOp::new(NumOp::Sub, Identifier::from("b"), Const::from(3)),
                ),
            ),
            Const::from(1),
        )
        .into()],
    );
}

/// Checks bitwise operations.
#[test]
fn check_bitwise_operations() {
    check_parser(
        "a & b",
        vec![BinOp::new(BitOp::And, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a&b",
        vec![BinOp::new(BitOp::And, Identifier::from("a"), Identifier::from("b")).into()],
    );

    check_parser(
        "a | b",
        vec![BinOp::new(BitOp::Or, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a|b",
        vec![BinOp::new(BitOp::Or, Identifier::from("a"), Identifier::from("b")).into()],
    );

    check_parser(
        "a ^ b",
        vec![BinOp::new(BitOp::Xor, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a^b",
        vec![BinOp::new(BitOp::Xor, Identifier::from("a"), Identifier::from("b")).into()],
    );

    check_parser(
        "a << b",
        vec![BinOp::new(BitOp::Shl, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a<<b",
        vec![BinOp::new(BitOp::Shl, Identifier::from("a"), Identifier::from("b")).into()],
    );

    check_parser(
        "a >> b",
        vec![BinOp::new(BitOp::Shr, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a>>b",
        vec![BinOp::new(BitOp::Shr, Identifier::from("a"), Identifier::from("b")).into()],
    );
}

/// Checks assignment operations.
#[test]
fn check_assign_operations() {
    check_parser(
        "a += b",
        vec![BinOp::new(AssignOp::Add, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a -= b",
        vec![BinOp::new(AssignOp::Sub, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a *= b",
        vec![BinOp::new(AssignOp::Mul, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a **= b",
        vec![BinOp::new(AssignOp::Exp, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a /= b",
        vec![BinOp::new(AssignOp::Div, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a %= b",
        vec![BinOp::new(AssignOp::Mod, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a &= b",
        vec![BinOp::new(AssignOp::And, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a |= b",
        vec![BinOp::new(AssignOp::Or, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a ^= b",
        vec![BinOp::new(AssignOp::Xor, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a <<= b",
        vec![BinOp::new(AssignOp::Shl, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a >>= b",
        vec![BinOp::new(AssignOp::Shr, Identifier::from("a"), Identifier::from("b")).into()],
    );
    check_parser(
        "a %= 10 / 2",
        vec![BinOp::new(
            AssignOp::Mod,
            Identifier::from("a"),
            BinOp::new(NumOp::Div, Const::from(10), Const::from(2)),
        )
        .into()],
    );
}

#[test]
fn check_relational_operations() {
    check_parser(
        "a < b",
        vec![BinOp::new(
            CompOp::LessThan,
            Identifier::from("a"),
            Identifier::from("b"),
        )
        .into()],
    );
    check_parser(
        "a > b",
        vec![BinOp::new(
            CompOp::GreaterThan,
            Identifier::from("a"),
            Identifier::from("b"),
        )
        .into()],
    );
    check_parser(
        "a <= b",
        vec![BinOp::new(
            CompOp::LessThanOrEqual,
            Identifier::from("a"),
            Identifier::from("b"),
        )
        .into()],
    );
    check_parser(
        "a >= b",
        vec![BinOp::new(
            CompOp::GreaterThanOrEqual,
            Identifier::from("a"),
            Identifier::from("b"),
        )
        .into()],
    );
    check_parser(
        "p in o",
        vec![BinOp::new(CompOp::In, Identifier::from("p"), Identifier::from("o")).into()],
    );
}
