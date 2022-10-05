use crate::{
    string::utf16,
    syntax::{
        ast::op::{AssignOp, BitOp, CompOp, LogOp, NumOp},
        ast::{
            node::{BinOp, Call, Declaration, DeclarationList, Identifier, New},
            Const, Node,
        },
        parser::tests::{check_invalid, check_parser},
    },
};
use boa_interner::{Interner, Sym};

/// Checks numeric operations
#[test]
fn check_numeric_operations() {
    let mut interner = Interner::default();
    check_parser(
        "a + b",
        vec![BinOp::new(
            NumOp::Add,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a+1",
        vec![BinOp::new(
            NumOp::Add,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a-1",
        vec![BinOp::new(
            NumOp::Sub,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a/2",
        vec![BinOp::new(
            NumOp::Div,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Const::from(2),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "let myRegex = /=/;",
        vec![DeclarationList::Let(
            vec![Declaration::new_with_identifier(
                interner.get_or_intern_static("myRegex", utf16!("myRegex")),
                Node::from(New::from(Call::new(
                    Identifier::new(Sym::REGEXP),
                    vec![
                        Node::from(Const::from(interner.get_or_intern_static("=", utf16!("=")))),
                        Node::from(Const::from(Sym::EMPTY_STRING)),
                    ],
                ))),
            )]
            .into(),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a * b",
        vec![BinOp::new(
            NumOp::Mul,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a*2",
        vec![BinOp::new(
            NumOp::Mul,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a**2",
        vec![BinOp::new(
            NumOp::Exp,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a%2",
        vec![BinOp::new(
            NumOp::Mod,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
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
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
                BinOp::new(
                    NumOp::Mul,
                    Identifier::new(interner.get_or_intern_static("d", utf16!("d"))),
                    BinOp::new(
                        NumOp::Sub,
                        Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a&b",
        vec![BinOp::new(
            BitOp::And,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a | b",
        vec![BinOp::new(
            BitOp::Or,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a|b",
        vec![BinOp::new(
            BitOp::Or,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a ^ b",
        vec![BinOp::new(
            BitOp::Xor,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a^b",
        vec![BinOp::new(
            BitOp::Xor,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a << b",
        vec![BinOp::new(
            BitOp::Shl,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a<<b",
        vec![BinOp::new(
            BitOp::Shl,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >> b",
        vec![BinOp::new(
            BitOp::Shr,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a>>b",
        vec![BinOp::new(
            BitOp::Shr,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a -= b",
        vec![BinOp::new(
            AssignOp::Sub,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a *= b",
        vec![BinOp::new(
            AssignOp::Mul,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a **= b",
        vec![BinOp::new(
            AssignOp::Exp,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a /= b",
        vec![BinOp::new(
            AssignOp::Div,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a %= b",
        vec![BinOp::new(
            AssignOp::Mod,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a &= b",
        vec![BinOp::new(
            AssignOp::And,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a |= b",
        vec![BinOp::new(
            AssignOp::Or,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a ^= b",
        vec![BinOp::new(
            AssignOp::Xor,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a <<= b",
        vec![BinOp::new(
            AssignOp::Shl,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >>= b",
        vec![BinOp::new(
            AssignOp::Shr,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >>>= b",
        vec![BinOp::new(
            AssignOp::Ushr,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a %= 10 / 2",
        vec![BinOp::new(
            AssignOp::Mod,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
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
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a > b",
        vec![BinOp::new(
            CompOp::GreaterThan,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a <= b",
        vec![BinOp::new(
            CompOp::LessThanOrEqual,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "a >= b",
        vec![BinOp::new(
            CompOp::GreaterThanOrEqual,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
        )
        .into()],
        interner,
    );

    let mut interner = Interner::default();
    check_parser(
        "p in o",
        vec![BinOp::new(
            CompOp::In,
            Identifier::new(interner.get_or_intern_static("p", utf16!("p"))),
            Identifier::new(interner.get_or_intern_static("o", utf16!("o"))),
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
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
                Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
            ),
            BinOp::new(
                LogOp::Or,
                BinOp::new(
                    LogOp::And,
                    Identifier::new(interner.get_or_intern_static("c", utf16!("c"))),
                    Identifier::new(interner.get_or_intern_static("d", utf16!("d"))),
                ),
                Identifier::new(interner.get_or_intern_static("e", utf16!("e"))),
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
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))),
                Identifier::new(interner.get_or_intern_static("b", utf16!("b"))),
            ),
            Identifier::new(interner.get_or_intern_static("c", utf16!("c"))),
        )
        .into()],
        interner,
    );

    check_invalid("a ?? b && c");
    check_invalid("a && b ?? c");
    check_invalid("a ?? b || c");
    check_invalid("a || b ?? c");
}

macro_rules! check_non_reserved_identifier {
    ($keyword:literal) => {{
        let mut interner = Interner::default();
        check_parser(
            format!("({})", $keyword).as_str(),
            vec![Identifier::new(interner.get_or_intern_static($keyword, utf16!($keyword))).into()],
            interner,
        );
    }};
}

#[test]
fn check_non_reserved_identifiers() {
    // https://tc39.es/ecma262/#sec-keywords-and-reserved-words
    // Those that are always allowed as identifiers, but also appear as
    // keywords within certain syntactic productions, at places where
    // Identifier is not allowed: as, async, from, get, meta, of, set,
    // and target.

    check_non_reserved_identifier!("as");
    check_non_reserved_identifier!("async");
    check_non_reserved_identifier!("from");
    check_non_reserved_identifier!("get");
    check_non_reserved_identifier!("meta");
    check_non_reserved_identifier!("of");
    check_non_reserved_identifier!("set");
    check_non_reserved_identifier!("target");
}
