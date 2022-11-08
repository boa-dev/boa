use crate::parser::tests::{check_invalid, check_parser};
use boa_ast::{
    declaration::{LexicalDeclaration, Variable},
    expression::{
        literal::Literal,
        operator::{
            assign::AssignOp,
            binary::{ArithmeticOp, BitwiseOp, LogicalOp, RelationalOp},
            Assign, Binary,
        },
        Call, Identifier, New,
    },
    Declaration, Expression, Statement,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;

/// Checks numeric operations
#[test]
fn check_numeric_operations() {
    let interner = &mut Interner::default();
    check_parser(
        "a + b",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Add.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a+1",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Add.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Literal::from(1).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a - b",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Sub.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a-1",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Sub.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Literal::from(1).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a / b",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Div.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a/2",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Div.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Literal::from(2).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "let myRegex = /=/;",
        vec![Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                interner
                    .get_or_intern_static("myRegex", utf16!("myRegex"))
                    .into(),
                Some(
                    New::from(Call::new(
                        Identifier::new(Sym::REGEXP).into(),
                        vec![
                            Literal::from(interner.get_or_intern_static("=", utf16!("="))).into(),
                            Literal::from(Sym::EMPTY_STRING).into(),
                        ]
                        .into(),
                    ))
                    .into(),
                ),
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "fn(/=/);",
        vec![Statement::Expression(Expression::from(Call::new(
            Identifier::new(interner.get_or_intern_static("fn", utf16!("fn"))).into(),
            vec![New::from(Call::new(
                Identifier::new(Sym::REGEXP).into(),
                vec![
                    Literal::from(interner.get_or_intern_static("=", utf16!("="))).into(),
                    Literal::from(Sym::EMPTY_STRING).into(),
                ]
                .into(),
            ))
            .into()]
            .into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "fn(a / b);",
        vec![Statement::Expression(Expression::from(Call::new(
            Identifier::new(interner.get_or_intern_static("fn", utf16!("fn"))).into(),
            vec![Expression::from(Binary::new(
                ArithmeticOp::Div.into(),
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
            ))]
            .into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "fn(a) / b;",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Div.into(),
            Call::new(
                Identifier::new(interner.get_or_intern_static("fn", utf16!("fn"))).into(),
                vec![Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into()]
                    .into(),
            )
            .into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a * b",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Mul.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a*2",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Mul.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Literal::from(2).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a ** b",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Exp.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a**2",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Exp.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Literal::from(2).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a % b",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Mod.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a%2",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Mod.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Literal::from(2).into(),
        )))
        .into()],
        interner,
    );
}

// Checks complex numeric operations.
#[test]
fn check_complex_numeric_operations() {
    let interner = &mut Interner::default();
    check_parser(
        "a + d*(b-3)+1",
        vec![Statement::Expression(Expression::from(Binary::new(
            ArithmeticOp::Add.into(),
            Binary::new(
                ArithmeticOp::Add.into(),
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                Binary::new(
                    ArithmeticOp::Mul.into(),
                    Identifier::new(interner.get_or_intern_static("d", utf16!("d"))).into(),
                    Binary::new(
                        ArithmeticOp::Sub.into(),
                        Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
                        Literal::from(3).into(),
                    )
                    .into(),
                )
                .into(),
            )
            .into(),
            Literal::from(1).into(),
        )))
        .into()],
        interner,
    );
}

/// Checks bitwise operations.
#[test]
fn check_bitwise_operations() {
    let interner = &mut Interner::default();
    check_parser(
        "a & b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::And.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a&b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::And.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a | b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Or.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a|b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Or.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a ^ b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Xor.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a^b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Xor.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a << b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Shl.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a<<b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Shl.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a >> b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Shr.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a>>b",
        vec![Statement::Expression(Expression::from(Binary::new(
            BitwiseOp::Shr.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );
}

/// Checks assignment operations.
#[test]
fn check_assign_operations() {
    let interner = &mut Interner::default();
    check_parser(
        "a += b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Add,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a -= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Sub,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a *= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Mul,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a **= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Exp,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a /= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Div,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a %= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Mod,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a &= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::And,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a |= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Or,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a ^= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Xor,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a <<= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Shl,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a >>= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Shr,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a >>>= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Ushr,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a %= 10 / 2",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Mod,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Binary::new(
                ArithmeticOp::Div.into(),
                Literal::from(10).into(),
                Literal::from(2).into(),
            )
            .into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a ??= b",
        vec![Statement::Expression(Expression::from(Assign::new(
            AssignOp::Coalesce,
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn check_relational_operations() {
    let interner = &mut Interner::default();
    check_parser(
        "a < b",
        vec![Statement::Expression(Expression::from(Binary::new(
            RelationalOp::LessThan.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a > b",
        vec![Statement::Expression(Expression::from(Binary::new(
            RelationalOp::GreaterThan.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a <= b",
        vec![Statement::Expression(Expression::from(Binary::new(
            RelationalOp::LessThanOrEqual.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a >= b",
        vec![Statement::Expression(Expression::from(Binary::new(
            RelationalOp::GreaterThanOrEqual.into(),
            Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
            Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "p in o",
        vec![Statement::Expression(Expression::from(Binary::new(
            RelationalOp::In.into(),
            Identifier::new(interner.get_or_intern_static("p", utf16!("p"))).into(),
            Identifier::new(interner.get_or_intern_static("o", utf16!("o"))).into(),
        )))
        .into()],
        interner,
    );
}

#[test]
fn check_logical_expressions() {
    let interner = &mut Interner::default();
    check_parser(
        "a && b || c && d || e",
        vec![Statement::Expression(Expression::from(Binary::new(
            LogicalOp::Or.into(),
            Binary::new(
                LogicalOp::And.into(),
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
            )
            .into(),
            Binary::new(
                LogicalOp::Or.into(),
                Binary::new(
                    LogicalOp::And.into(),
                    Identifier::new(interner.get_or_intern_static("c", utf16!("c"))).into(),
                    Identifier::new(interner.get_or_intern_static("d", utf16!("d"))).into(),
                )
                .into(),
                Identifier::new(interner.get_or_intern_static("e", utf16!("e"))).into(),
            )
            .into(),
        )))
        .into()],
        interner,
    );

    let interner = &mut Interner::default();
    check_parser(
        "a ?? b ?? c",
        vec![Statement::Expression(Expression::from(Binary::new(
            LogicalOp::Coalesce.into(),
            Binary::new(
                LogicalOp::Coalesce.into(),
                Identifier::new(interner.get_or_intern_static("a", utf16!("a"))).into(),
                Identifier::new(interner.get_or_intern_static("b", utf16!("b"))).into(),
            )
            .into(),
            Identifier::new(interner.get_or_intern_static("c", utf16!("c"))).into(),
        )))
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
        let interner = &mut Interner::default();
        check_parser(
            format!("({})", $keyword).as_str(),
            vec![Statement::Expression(Expression::from(Identifier::new(
                interner.get_or_intern_static($keyword, utf16!($keyword)),
            )))
            .into()],
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
