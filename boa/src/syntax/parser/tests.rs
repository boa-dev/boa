//! Tests for the parser.

use super::*;
use crate::syntax::ast::{constant::Const, op::BinOp};
use crate::syntax::{
    ast::node::{FormalParameter, Node},
    lexer::Lexer,
};

fn create_bin_op(op: BinOp, exp1: Node, exp2: Node) -> Node {
    Node::BinOp(op, Box::new(exp1), Box::new(exp2))
}

#[allow(clippy::result_unwrap_used)]
fn check_parser(js: &str, expr: &[Node]) {
    let mut lexer = Lexer::new(js);
    lexer.lex().expect("failed to lex");

    assert_eq!(
        Parser::new(lexer.tokens).parse_all().unwrap(),
        Node::StatementList(expr.into())
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
    check_parser("\"\"", &[Node::Const(Const::String(String::new()))]);

    // Check non-empty string
    check_parser(
        "\"hello\"",
        &[Node::Const(Const::String(String::from("hello")))],
    );
}
#[test]
fn check_object_short_function() {
    // Testing short function syntax
    let mut object_properties: BTreeMap<String, Node> = BTreeMap::new();
    object_properties.insert(String::from("a"), Node::Const(Const::Bool(true)));
    object_properties.insert(
        String::from("b"),
        Node::FunctionDecl(None, vec![], Box::new(Node::Block(vec![]))),
    );

    check_parser(
        "{
              a: true,
              b() {}
            };
            ",
        &[Node::ObjectDecl(Box::new(object_properties))],
    );
}

#[test]
fn check_object_short_function_arguments() {
    // Testing short function syntax
    let mut object_properties: BTreeMap<String, Node> = BTreeMap::new();
    object_properties.insert(String::from("a"), Node::Const(Const::Bool(true)));
    object_properties.insert(
        String::from("b"),
        Node::FunctionDecl(
            None,
            vec![FormalParameter::new(
                String::from("name"),
                Some(Box::new(Node::Local(String::from("test")))),
                false,
            )],
            Box::new(Node::Block(vec![])),
        ),
    );

    check_parser(
        "{
              a: true,
              b(test) {}
            };
            ",
        &[Node::ObjectDecl(Box::new(object_properties))],
    );
}
#[test]
fn check_array() {
    use crate::syntax::ast::constant::Const;

    // Check empty array
    check_parser("[]", &[Node::ArrayDecl(vec![])]);

    // Check array with empty slot
    check_parser(
        "[,]",
        &[Node::ArrayDecl(vec![Node::Const(Const::Undefined)])],
    );

    // Check numeric array
    check_parser(
        "[1, 2, 3]",
        &[Node::ArrayDecl(vec![
            Node::Const(Const::Num(1.0)),
            Node::Const(Const::Num(2.0)),
            Node::Const(Const::Num(3.0)),
        ])],
    );

    // Check numeric array with trailing comma
    check_parser(
        "[1, 2, 3,]",
        &[Node::ArrayDecl(vec![
            Node::Const(Const::Num(1.0)),
            Node::Const(Const::Num(2.0)),
            Node::Const(Const::Num(3.0)),
        ])],
    );

    // Check numeric array with an elision
    check_parser(
        "[1, 2, , 3]",
        &[Node::ArrayDecl(vec![
            Node::Const(Const::Num(1.0)),
            Node::Const(Const::Num(2.0)),
            Node::Const(Const::Undefined),
            Node::Const(Const::Num(3.0)),
        ])],
    );

    // Check numeric array with repeated elision
    check_parser(
        "[1, 2, ,, 3]",
        &[Node::ArrayDecl(vec![
            Node::Const(Const::Num(1.0)),
            Node::Const(Const::Num(2.0)),
            Node::Const(Const::Undefined),
            Node::Const(Const::Undefined),
            Node::Const(Const::Num(3.0)),
        ])],
    );

    // Check combined array
    check_parser(
        "[1, \"a\", 2]",
        &[Node::ArrayDecl(vec![
            Node::Const(Const::Num(1.0)),
            Node::Const(Const::String(String::from("a"))),
            Node::Const(Const::Num(2.0)),
        ])],
    );

    // Check combined array with empty string
    check_parser(
        "[1, \"\", 2]",
        &[Node::ArrayDecl(vec![
            Node::Const(Const::Num(1.0)),
            Node::Const(Const::String(String::new())),
            Node::Const(Const::Num(2.0)),
        ])],
    );
}

#[test]
fn check_declarations() {
    use crate::syntax::ast::constant::Const;

    // Check `var` declaration
    check_parser(
        "var a = 5;",
        &[Node::VarDecl(vec![(
            String::from("a"),
            Some(Node::Const(Const::Num(5.0))),
        )])],
    );

    // Check `var` declaration with no spaces
    check_parser(
        "var a=5;",
        &[Node::VarDecl(vec![(
            String::from("a"),
            Some(Node::Const(Const::Num(5.0))),
        )])],
    );

    // Check empty `var` declaration
    check_parser("var a;", &[Node::VarDecl(vec![(String::from("a"), None)])]);

    // Check multiple `var` declaration
    check_parser(
        "var a = 5, b, c = 6;",
        &[Node::VarDecl(vec![
            (String::from("a"), Some(Node::Const(Const::Num(5.0)))),
            (String::from("b"), None),
            (String::from("c"), Some(Node::Const(Const::Num(6.0)))),
        ])],
    );

    // Check `let` declaration
    check_parser(
        "let a = 5;",
        &[Node::LetDecl(vec![(
            String::from("a"),
            Some(Node::Const(Const::Num(5.0))),
        )])],
    );

    // Check `let` declaration with no spaces
    check_parser(
        "let a=5;",
        &[Node::LetDecl(vec![(
            String::from("a"),
            Some(Node::Const(Const::Num(5.0))),
        )])],
    );

    // Check empty `let` declaration
    check_parser("let a;", &[Node::LetDecl(vec![(String::from("a"), None)])]);

    // Check multiple `let` declaration
    check_parser(
        "let a = 5, b, c = 6;",
        &[Node::LetDecl(vec![
            (String::from("a"), Some(Node::Const(Const::Num(5.0)))),
            (String::from("b"), None),
            (String::from("c"), Some(Node::Const(Const::Num(6.0)))),
        ])],
    );

    // Check `const` declaration
    check_parser(
        "const a = 5;",
        &[Node::ConstDecl(vec![(
            String::from("a"),
            Node::Const(Const::Num(5.0)),
        )])],
    );

    // Check `const` declaration with no spaces
    check_parser(
        "const a=5;",
        &[Node::ConstDecl(vec![(
            String::from("a"),
            Node::Const(Const::Num(5.0)),
        )])],
    );

    // Check empty `const` declaration
    check_invalid("const a;");

    // Check multiple `const` declaration
    check_parser(
        "const a = 5, c = 6;",
        &[Node::ConstDecl(vec![
            (String::from("a"), Node::Const(Const::Num(5.0))),
            (String::from("c"), Node::Const(Const::Num(6.0))),
        ])],
    );
}

#[test]
fn check_operations() {
    use crate::syntax::ast::{constant::Const, op::BinOp};

    fn create_bin_op(op: BinOp, exp1: Node, exp2: Node) -> Node {
        Node::BinOp(op, Box::new(exp1), Box::new(exp2))
    }

    // Check numeric operations
    check_parser(
        "a + b",
        &[create_bin_op(
            BinOp::Num(NumOp::Add),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a+1",
        &[create_bin_op(
            BinOp::Num(NumOp::Add),
            Node::Local(String::from("a")),
            Node::Const(Const::Num(1.0)),
        )],
    );
    check_parser(
        "a - b",
        &[create_bin_op(
            BinOp::Num(NumOp::Sub),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a-1",
        &[create_bin_op(
            BinOp::Num(NumOp::Sub),
            Node::Local(String::from("a")),
            Node::Const(Const::Num(1.0)),
        )],
    );
    check_parser(
        "a / b",
        &[create_bin_op(
            BinOp::Num(NumOp::Div),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a/2",
        &[create_bin_op(
            BinOp::Num(NumOp::Div),
            Node::Local(String::from("a")),
            Node::Const(Const::Num(2.0)),
        )],
    );
    check_parser(
        "a * b",
        &[create_bin_op(
            BinOp::Num(NumOp::Mul),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a*2",
        &[create_bin_op(
            BinOp::Num(NumOp::Mul),
            Node::Local(String::from("a")),
            Node::Const(Const::Num(2.0)),
        )],
    );
    check_parser(
        "a ** b",
        &[create_bin_op(
            BinOp::Num(NumOp::Exp),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a**2",
        &[create_bin_op(
            BinOp::Num(NumOp::Exp),
            Node::Local(String::from("a")),
            Node::Const(Const::Num(2.0)),
        )],
    );
    check_parser(
        "a % b",
        &[create_bin_op(
            BinOp::Num(NumOp::Mod),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a%2",
        &[create_bin_op(
            BinOp::Num(NumOp::Mod),
            Node::Local(String::from("a")),
            Node::Const(Const::Num(2.0)),
        )],
    );

    // Check complex numeric operations
    check_parser(
        "a + d*(b-3)+1",
        &[create_bin_op(
            BinOp::Num(NumOp::Add),
            Node::Local(String::from("a")),
            create_bin_op(
                BinOp::Num(NumOp::Add),
                // FIXME: shouldn't the last addition be on the right?
                Node::Const(Const::Num(1.0)),
                create_bin_op(
                    BinOp::Num(NumOp::Mul),
                    Node::Local(String::from("d")),
                    create_bin_op(
                        BinOp::Num(NumOp::Sub),
                        Node::Local(String::from("b")),
                        Node::Const(Const::Num(3.0)),
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
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a&b",
        &[create_bin_op(
            BinOp::Bit(BitOp::And),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );

    check_parser(
        "a | b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Or),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a|b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Or),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );

    check_parser(
        "a ^ b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Xor),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a^b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Xor),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );

    check_parser(
        "a << b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shl),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a<<b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shl),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );

    check_parser(
        "a >> b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shr),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a>>b",
        &[create_bin_op(
            BinOp::Bit(BitOp::Shr),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );

    // Check assign ops
    check_parser(
        "a += b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Add),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a -= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Sub),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a *= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Mul),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a **= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Exp),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a /= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Div),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a %= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Mod),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a &= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::And),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a |= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Or),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a ^= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Xor),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a <<= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Shl),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a >>= b",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Shr),
            Node::Local(String::from("a")),
            Node::Local(String::from("b")),
        )],
    );
    check_parser(
        "a %= 10 / 2",
        &[create_bin_op(
            BinOp::Assign(AssignOp::Mod),
            Node::Local(String::from("a")),
            create_bin_op(
                BinOp::Num(NumOp::Div),
                Node::Const(Const::Num(10.0)),
                Node::Const(Const::Num(2.0)),
            ),
        )],
    );
}

#[test]
fn check_function_declarations() {
    check_parser(
        "function foo(a) { return a; }",
        &[Node::FunctionDecl(
            Some(String::from("foo")),
            vec![FormalParameter::new(
                String::from("a"),
                None,
                false,
            )],
            Box::new(Node::StatementList(vec![Node::Return(Some(Box::new(
                Node::Local(String::from("a")),
            )))])),
        )],
    );

    check_parser(
        "function foo(a) { return; }",
        &[Node::FunctionDecl(
            Some(String::from("foo")),
            vec![FormalParameter::new(
                String::from("a"),
                None,
                false,
            )],
            Box::new(Node::StatementList(vec![Node::Return(None)])),
        )],
    );

    check_parser(
        "function foo(a) { return }",
        &[Node::FunctionDecl(
            Some(String::from("foo")),
            vec![FormalParameter::new(
                String::from("a"),
                None,
                false,
            )],
            Box::new(Node::StatementList(vec![Node::Return(None)])),
        )],
    );

    check_parser(
        "function foo(a, ...b) {}",
        &[Node::FunctionDecl(
            Some(String::from("foo")),
            vec![
                FormalParameter::new(
                    String::from("a"),
                    None,
                    false,
                ),
                FormalParameter::new(
                    String::from("b"),
                    None,
                    true,
                ),
            ],
            Box::new(Node::StatementList(Vec::new()))
        )],
    );

    check_parser(
        "(...a) => {}",
        &[Node::ArrowFunctionDecl(
            vec![FormalParameter::new(
                String::from("a"),
                None,
                true,
            )],
            Box::new(Node::StatementList(Vec::new()))
        )],
    );

    check_parser(
        "(a, b, ...c) => {}",
        &[Node::ArrowFunctionDecl(
            vec![
                FormalParameter::new(
                    String::from("a"),
                    None,
                    false,
                ),
                FormalParameter::new(
                    String::from("b"),
                    None,
                    false,
                ),
                FormalParameter::new(
                    String::from("c"),
                    None,
                    true,
                ),
            ],
            Box::new(Node::StatementList(Vec::new())),
        )],
    );

    check_parser(
        "(a, b) => { return a + b; }",
        &[Node::ArrowFunctionDecl(
            vec![
                FormalParameter::new(
                    String::from("a"),
                    None,
                    false,
                ),
                FormalParameter::new(
                    String::from("b"),
                    None,
                    false,
                ),
            ],
            Box::new(Node::StatementList(vec![Node::Return(Some(Box::new(
                create_bin_op(
                    BinOp::Num(NumOp::Add),
                    Node::Local(String::from("a")),
                    Node::Local(String::from("b")),
                ),
            )))])),
        )],
    );

    check_parser(
        "(a, b) => { return; }",
        &[Node::ArrowFunctionDecl(
            vec![
                FormalParameter::new(
                    String::from("a"),
                    None,
                    false,
                ),
                FormalParameter::new(
                    String::from("b"),
                    None,
                    false,
                ),
            ],
            Box::new(Node::StatementList(vec![Node::Return(None)])),
        )],
    );

    check_parser(
        "(a, b) => { return }",
        &[Node::ArrowFunctionDecl(
            vec![
                FormalParameter::new(
                    String::from("a"),
                    None,
                    false,
                ),
                FormalParameter::new(
                    String::from("b"),
                    None,
                    false,
                ),
            ],
            Box::new(Node::StatementList(vec![Node::Return(None)])),
        )],
    );
}
