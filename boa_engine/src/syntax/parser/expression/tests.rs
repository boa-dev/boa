use crate::syntax::{
    ast::op::{AssignOp, BitOp, CompOp, LogOp, NumOp},
    ast::{
        node::{BinOp, Identifier},
        Const,
    },
    parser::tests::{check_invalid, check_parser},
};
use boa_interner::Interner;

/// Checks numeric operations
#[test]
fn check_numeric_operations() {
    let mut interner = Interner::default();
    check_parser(
        "a + b",
        vec![BinOp::new(
            NumOp::Add,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a+1",
        vec![BinOp::new(
            NumOp::Add,
            Identifier::new(interner.get_or_intern_static("a")),
            Const::from(1),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a - b",
        vec![BinOp::new(
            NumOp::Sub,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a-1",
        vec![BinOp::new(
            NumOp::Sub,
            Identifier::new(interner.get_or_intern_static("a")),
            Const::from(1),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a / b",
        vec![BinOp::new(
            NumOp::Div,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a/2",
        vec![BinOp::new(
            NumOp::Div,
            Identifier::new(interner.get_or_intern_static("a")),
            Const::from(2),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a * b",
        vec![BinOp::new(
            NumOp::Mul,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a*2",
        vec![BinOp::new(
            NumOp::Mul,
            Identifier::new(interner.get_or_intern_static("a")),
            Const::from(2),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a ** b",
        vec![BinOp::new(
            NumOp::Exp,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a**2",
        vec![BinOp::new(
            NumOp::Exp,
            Identifier::new(interner.get_or_intern_static("a")),
            Const::from(2),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a % b",
        vec![BinOp::new(
            NumOp::Mod,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a%2",
        vec![BinOp::new(
            NumOp::Mod,
            Identifier::new(interner.get_or_intern_static("a")),
            Const::from(2),
        )
        .into()],
        interner,
    );
}

// Checks complex numeric operations.
#[test]
fn check_complex_numeric_operations() {
    let mut interner = Interner::default();
    check_parser(
        "a + d*(b-3)+1",
        vec![BinOp::new(
            NumOp::Add,
            BinOp::new(
                NumOp::Add,
                Identifier::new(interner.get_or_intern_static("a")),
                BinOp::new(
                    NumOp::Mul,
                    Identifier::new(interner.get_or_intern_static("d")),
                    BinOp::new(
                        NumOp::Sub,
                        Identifier::new(interner.get_or_intern_static("b")),
                        Const::from(3),
                    ),
                ),
            ),
            Const::from(1),
        )
        .into()],
        interner,
    );
}

/// Checks bitwise operations.
#[test]
fn check_bitwise_operations() {
    let mut interner = Interner::default();
    check_parser(
        "a & b",
        vec![BinOp::new(
            BitOp::And,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a&b",
        vec![BinOp::new(
            BitOp::And,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a | b",
        vec![BinOp::new(
            BitOp::Or,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a|b",
        vec![BinOp::new(
            BitOp::Or,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a ^ b",
        vec![BinOp::new(
            BitOp::Xor,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a^b",
        vec![BinOp::new(
            BitOp::Xor,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a << b",
        vec![BinOp::new(
            BitOp::Shl,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a<<b",
        vec![BinOp::new(
            BitOp::Shl,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >> b",
        vec![BinOp::new(
            BitOp::Shr,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a>>b",
        vec![BinOp::new(
            BitOp::Shr,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );
}

/// Checks assignment operations.
#[test]
fn check_assign_operations() {
    let mut interner = Interner::default();
    check_parser(
        "a += b",
        vec![BinOp::new(
            AssignOp::Add,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a -= b",
        vec![BinOp::new(
            AssignOp::Sub,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a *= b",
        vec![BinOp::new(
            AssignOp::Mul,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a **= b",
        vec![BinOp::new(
            AssignOp::Exp,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a /= b",
        vec![BinOp::new(
            AssignOp::Div,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a %= b",
        vec![BinOp::new(
            AssignOp::Mod,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a &= b",
        vec![BinOp::new(
            AssignOp::And,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a |= b",
        vec![BinOp::new(
            AssignOp::Or,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a ^= b",
        vec![BinOp::new(
            AssignOp::Xor,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a <<= b",
        vec![BinOp::new(
            AssignOp::Shl,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >>= b",
        vec![BinOp::new(
            AssignOp::Shr,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >>>= b",
        vec![BinOp::new(
            AssignOp::Ushr,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a %= 10 / 2",
        vec![BinOp::new(
            AssignOp::Mod,
            Identifier::new(interner.get_or_intern_static("a")),
            BinOp::new(NumOp::Div, Const::from(10), Const::from(2)),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a ??= b",
        vec![BinOp::new(
            AssignOp::Coalesce,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_relational_operations() {
    let mut interner = Interner::default();
    check_parser(
        "a < b",
        vec![BinOp::new(
            CompOp::LessThan,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a > b",
        vec![BinOp::new(
            CompOp::GreaterThan,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a <= b",
        vec![BinOp::new(
            CompOp::LessThanOrEqual,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >= b",
        vec![BinOp::new(
            CompOp::GreaterThanOrEqual,
            Identifier::new(interner.get_or_intern_static("a")),
            Identifier::new(interner.get_or_intern_static("b")),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "p in o",
        vec![BinOp::new(
            CompOp::In,
            Identifier::new(interner.get_or_intern_static("p")),
            Identifier::new(interner.get_or_intern_static("o")),
        )
        .into()],
        interner,
    );
}

#[test]
fn check_logical_expressions() {
    let mut interner = Interner::default();
    check_parser(
        "a && b || c && d || e",
        vec![BinOp::new(
            LogOp::Or,
            BinOp::new(
                LogOp::And,
                Identifier::new(interner.get_or_intern_static("a")),
                Identifier::new(interner.get_or_intern_static("b")),
            ),
            BinOp::new(
                LogOp::Or,
                BinOp::new(
                    LogOp::And,
                    Identifier::new(interner.get_or_intern_static("c")),
                    Identifier::new(interner.get_or_intern_static("d")),
                ),
                Identifier::new(interner.get_or_intern_static("e")),
            ),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a ?? b ?? c",
        vec![BinOp::new(
            LogOp::Coalesce,
            BinOp::new(
                LogOp::Coalesce,
                Identifier::new(interner.get_or_intern_static("a")),
                Identifier::new(interner.get_or_intern_static("b")),
            ),
            Identifier::new(interner.get_or_intern_static("c")),
        )
        .into()],
        interner,
    );

    check_invalid("a ?? b && c");
    check_invalid("a && b ?? c");
    check_invalid("a ?? b || c");
    check_invalid("a || b ?? c");
}
