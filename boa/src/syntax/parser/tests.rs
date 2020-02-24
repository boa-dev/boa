//! Tests for the parser.

use super::*;
use crate::syntax::ast::{constant::Const, op::BinOp};
use crate::syntax::{
    ast::expr::{Expr, ExprDef},
    lexer::Lexer,
};

fn create_bin_op(op: BinOp, exp1: Expr, exp2: Expr) -> Expr {
    Expr::new(ExprDef::BinOp(op, Box::new(exp1), Box::new(exp2)))
}

#[allow(clippy::result_unwrap_used)]
fn check_parser(js: &str, expr: &[Expr]) {
    let mut lexer = Lexer::new(js);
    lexer.lex().expect("failed to lex");

    assert_eq!(
        Parser::new(lexer.tokens).parse_all().unwrap(),
        Expr::new(ExprDef::Block(expr.into()))
    );
}

fn check_invalid(js: &str) {
    let mut lexer = Lexer::new(js);
    lexer.lex().expect("failed to lex");

    assert!(Parser::new(lexer.tokens).parse_all().is_err());
}

#[test]
fn check_string() {
    use crate::syntax::ast::constant::Const;

    // Check empty string
    check_parser(
        "\"\"",
        &[Expr::new(ExprDef::Const(Const::String(String::new())))],
    );

    // Check non-empty string
    check_parser(
        "\"hello\"",
        &[Expr::new(ExprDef::Const(Const::String(String::from(
            "hello",
        ))))],
    );
}
#[test]
fn check_object_short_function() {
    // Testing short function syntax
    let mut object_properties: BTreeMap<String, Expr> = BTreeMap::new();
    object_properties.insert(
        String::from("a"),
        Expr::new(ExprDef::Const(Const::Bool(true))),
    );
    object_properties.insert(
        String::from("b"),
        Expr::new(ExprDef::FunctionDecl(
            None,
            vec![],
            Box::new(Expr::new(ExprDef::Block(vec![]))),
        )),
    );

    check_parser(
        "{
              a: true,
              b() {}
            };
            ",
        &[Expr::new(ExprDef::ObjectDecl(Box::new(object_properties)))],
    );
}

#[test]
fn check_object_short_function_arguments() {
    // Testing short function syntax
    let mut object_properties: BTreeMap<String, Expr> = BTreeMap::new();
    object_properties.insert(
        String::from("a"),
        Expr::new(ExprDef::Const(Const::Bool(true))),
    );
    object_properties.insert(
        String::from("b"),
        Expr::new(ExprDef::FunctionDecl(
            None,
            vec![Expr::new(ExprDef::Local(String::from("test")))],
            Box::new(Expr::new(ExprDef::Block(vec![]))),
        )),
    );

    check_parser(
        "{
              a: true,
              b(test) {}
            };
            ",
        &[Expr::new(ExprDef::ObjectDecl(Box::new(object_properties)))],
    );
}
#[test]
fn check_array() {
    use crate::syntax::ast::constant::Const;

    // Check empty array
    check_parser("[]", &[Expr::new(ExprDef::ArrayDecl(vec![]))]);

    // Check array with empty slot
    check_parser(
        "[,]",
        &[Expr::new(ExprDef::ArrayDecl(vec![Expr::new(
            ExprDef::Const(Const::Undefined),
        )]))],
    );

    // Check numeric array
    check_parser(
        "[1, 2, 3]",
        &[Expr::new(ExprDef::ArrayDecl(vec![
            Expr::new(ExprDef::Const(Const::Num(1.0))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
            Expr::new(ExprDef::Const(Const::Num(3.0))),
        ]))],
    );

    // Check numeric array with trailing comma
    check_parser(
        "[1, 2, 3,]",
        &[Expr::new(ExprDef::ArrayDecl(vec![
            Expr::new(ExprDef::Const(Const::Num(1.0))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
            Expr::new(ExprDef::Const(Const::Num(3.0))),
        ]))],
    );

    // Check numeric array with an elision
    check_parser(
        "[1, 2, , 3]",
        &[Expr::new(ExprDef::ArrayDecl(vec![
            Expr::new(ExprDef::Const(Const::Num(1.0))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
            Expr::new(ExprDef::Const(Const::Undefined)),
            Expr::new(ExprDef::Const(Const::Num(3.0))),
        ]))],
    );

    // Check numeric array with repeated elision
    check_parser(
        "[1, 2, ,, 3]",
        &[Expr::new(ExprDef::ArrayDecl(vec![
            Expr::new(ExprDef::Const(Const::Num(1.0))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
            Expr::new(ExprDef::Const(Const::Undefined)),
            Expr::new(ExprDef::Const(Const::Undefined)),
            Expr::new(ExprDef::Const(Const::Num(3.0))),
        ]))],
    );

    // Check combined array
    check_parser(
        "[1, \"a\", 2]",
        &[Expr::new(ExprDef::ArrayDecl(vec![
            Expr::new(ExprDef::Const(Const::Num(1.0))),
            Expr::new(ExprDef::Const(Const::String(String::from("a")))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
        ]))],
    );

    // Check combined array with empty string
    check_parser(
        "[1, \"\", 2]",
        &[Expr::new(ExprDef::ArrayDecl(vec![
            Expr::new(ExprDef::Const(Const::Num(1.0))),
            Expr::new(ExprDef::Const(Const::String(String::new()))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
        ]))],
    );
}

#[test]
fn check_declarations() {
    use crate::syntax::ast::constant::Const;

    // Check `var` declaration
    check_parser(
        "var a = 5;",
        &[Expr::new(ExprDef::VarDecl(vec![(
            String::from("a"),
            Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
        )]))],
    );

    // Check `var` declaration with no spaces
    check_parser(
        "var a=5;",
        &[Expr::new(ExprDef::VarDecl(vec![(
            String::from("a"),
            Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
        )]))],
    );

    // Check empty `var` declaration
    check_parser(
        "var a;",
        &[Expr::new(ExprDef::VarDecl(vec![(String::from("a"), None)]))],
    );

    // Check multiple `var` declaration
    check_parser(
        "var a = 5, b, c = 6;",
        &[Expr::new(ExprDef::VarDecl(vec![
            (
                String::from("a"),
                Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
            ),
            (String::from("b"), None),
            (
                String::from("c"),
                Some(Expr::new(ExprDef::Const(Const::Num(6.0)))),
            ),
        ]))],
    );

    // Check `let` declaration
    check_parser(
        "let a = 5;",
        &[Expr::new(ExprDef::LetDecl(vec![(
            String::from("a"),
            Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
        )]))],
    );

    // Check `let` declaration with no spaces
    check_parser(
        "let a=5;",
        &[Expr::new(ExprDef::LetDecl(vec![(
            String::from("a"),
            Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
        )]))],
    );

    // Check empty `let` declaration
    check_parser(
        "let a;",
        &[Expr::new(ExprDef::LetDecl(vec![(String::from("a"), None)]))],
    );

    // Check multiple `let` declaration
    check_parser(
        "let a = 5, b, c = 6;",
        &[Expr::new(ExprDef::LetDecl(vec![
            (
                String::from("a"),
                Some(Expr::new(ExprDef::Const(Const::Num(5.0)))),
            ),
            (String::from("b"), None),
            (
                String::from("c"),
                Some(Expr::new(ExprDef::Const(Const::Num(6.0)))),
            ),
        ]))],
    );

    // Check `const` declaration
    check_parser(
        "const a = 5;",
        &[Expr::new(ExprDef::ConstDecl(vec![(
            String::from("a"),
            Expr::new(ExprDef::Const(Const::Num(5.0))),
        )]))],
    );

    // Check `const` declaration with no spaces
    check_parser(
        "const a=5;",
        &[Expr::new(ExprDef::ConstDecl(vec![(
            String::from("a"),
            Expr::new(ExprDef::Const(Const::Num(5.0))),
        )]))],
    );

    // Check empty `const` declaration
    check_invalid("const a;");

    // Check multiple `const` declaration
    check_parser(
        "const a = 5, c = 6;",
        &[Expr::new(ExprDef::ConstDecl(vec![
            (
                String::from("a"),
                Expr::new(ExprDef::Const(Const::Num(5.0))),
            ),
            (
                String::from("c"),
                Expr::new(ExprDef::Const(Const::Num(6.0))),
            ),
        ]))],
    );
}

#[test]
fn check_operations() {
    use crate::syntax::ast::{constant::Const, op::BinOp};

    fn create_bin_op(op: BinOp, exp1: Expr, exp2: Expr) -> Expr {
        Expr::new(ExprDef::BinOp(op, Box::new(exp1), Box::new(exp2)))
    }

    // Check numeric operations
    check_parser(
        "a + b",
        &[create_bin_op(
            BinOp::Num(NumOp::Add),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a+1",
        &[create_bin_op(
            BinOp::Num(NumOp::Add),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Const(Const::Num(1.0))),
        )],
    );
    check_parser(
        "a - b",
        &[create_bin_op(
            BinOp::Num(NumOp::Sub),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a-1",
        &[create_bin_op(
            BinOp::Num(NumOp::Sub),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Const(Const::Num(1.0))),
        )],
    );
    check_parser(
        "a / b",
        &[create_bin_op(
            BinOp::Num(NumOp::Div),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a/2",
        &[create_bin_op(
            BinOp::Num(NumOp::Div),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
        )],
    );
    check_parser(
        "a * b",
        &[create_bin_op(
            BinOp::Num(NumOp::Mul),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a*2",
        &[create_bin_op(
            BinOp::Num(NumOp::Mul),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
        )],
    );
    check_parser(
        "a ** b",
        &[create_bin_op(
            BinOp::Num(NumOp::Pow),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a**2",
        &[create_bin_op(
            BinOp::Num(NumOp::Pow),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
        )],
    );
    check_parser(
        "a % b",
        &[create_bin_op(
            BinOp::Num(NumOp::Mod),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a%2",
        &[create_bin_op(
            BinOp::Num(NumOp::Mod),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Const(Const::Num(2.0))),
        )],
    );

    // Check complex numeric operations
    check_parser(
        "a + d*(b-3)+1",
        &[create_bin_op(
            BinOp::Num(NumOp::Add),
            Expr::new(ExprDef::Local(String::from("a"))),
            create_bin_op(
                BinOp::Num(NumOp::Add),
                // FIXME: shouldn't the last addition be on the right?
                Expr::new(ExprDef::Const(Const::Num(1.0))),
                create_bin_op(
                    BinOp::Num(NumOp::Mul),
                    Expr::new(ExprDef::Local(String::from("d"))),
                    create_bin_op(
                        BinOp::Num(NumOp::Sub),
                        Expr::new(ExprDef::Local(String::from("b"))),
                        Expr::new(ExprDef::Const(Const::Num(3.0))),
                    ),
                ),
            ),
        )],
    );

    // Check bitwise operations
    check_parser(
        "a & b",
        &[create_bin_op(
            BinOp::Bit(BitOp::And),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a&b",
        &[create_bin_op(
            BinOp::Bit(BitOp::And),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );

    check_parser(
        "a | b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Or),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a|b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Or),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );

    check_parser(
        "a ^ b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Xor),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a^b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Xor),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );

    check_parser(
        "a << b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shl),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a<<b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shl),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );

    check_parser(
        "a >> b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shr),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a>>b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shr),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );

    // Check assign ops
    check_parser(
        "a += b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Add),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a -= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Sub),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a *= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Mul),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a **= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Pow),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a /= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Div),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a %= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Mod),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a &= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::And),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a |= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Or),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a ^= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Xor),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a <<= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Shl),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a >>= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Shr),
            Expr::new(ExprDef::Local(String::from("a"))),
            Expr::new(ExprDef::Local(String::from("b"))),
        )],
    );
    check_parser(
        "a %= 10 / 2",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Mod),
            Expr::new(ExprDef::Local(String::from("a"))),
            create_bin_op(
                BinOp::Num(NumOp::Div),
                Expr::new(ExprDef::Const(Const::Num(10.0))),
                Expr::new(ExprDef::Const(Const::Num(2.0))),
            ),
        )],
    );
}

#[test]
fn check_function_declarations() {
    check_parser(
        "function foo(a) { return a; }",
        &[Expr::new(ExprDef::FunctionDecl(
            Some(String::from("foo")),
            vec![Expr::new(ExprDef::Local(String::from("a")))],
            Box::new(Expr::new(ExprDef::Block(vec![Expr::new(ExprDef::Return(
                Some(Box::new(Expr::new(ExprDef::Local(String::from("a"))))),
            ))]))),
        ))],
    );

    check_parser(
        "function (a, ...b) {}",
        &[Expr::new(ExprDef::FunctionDecl(
            None,
            vec![
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::UnaryOp(
                    UnaryOp::Spread,
                    Box::new(Expr::new(ExprDef::Local(String::from("b")))),
                )),
            ],
            Box::new(Expr::new(ExprDef::ObjectDecl(Box::new(BTreeMap::new())))),
        ))],
    );

    check_parser(
        "(...a) => {}",
        &[Expr::new(ExprDef::ArrowFunctionDecl(
            vec![Expr::new(ExprDef::UnaryOp(
                UnaryOp::Spread,
                Box::new(Expr::new(ExprDef::Local(String::from("a")))),
            ))],
            Box::new(Expr::new(ExprDef::ObjectDecl(Box::new(BTreeMap::new())))),
        ))],
    );

    check_parser(
        "(a, b, ...c) => {}",
        &[Expr::new(ExprDef::ArrowFunctionDecl(
            vec![
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
                Expr::new(ExprDef::UnaryOp(
                    UnaryOp::Spread,
                    Box::new(Expr::new(ExprDef::Local(String::from("c")))),
                )),
            ],
            Box::new(Expr::new(ExprDef::ObjectDecl(Box::new(BTreeMap::new())))),
        ))],
    );

    check_parser(
        "(a, b) => { return a + b; }",
        &[Expr::new(ExprDef::ArrowFunctionDecl(
            vec![
                Expr::new(ExprDef::Local(String::from("a"))),
                Expr::new(ExprDef::Local(String::from("b"))),
            ],
            Box::new(Expr::new(ExprDef::Block(vec![Expr::new(ExprDef::Return(
                Some(Box::new(create_bin_op(
                    BinOp::Num(NumOp::Add),
                    Expr::new(ExprDef::Local(String::from("a"))),
                    Expr::new(ExprDef::Local(String::from("b"))),
                ))),
            ))]))),
        ))],
    );
}
