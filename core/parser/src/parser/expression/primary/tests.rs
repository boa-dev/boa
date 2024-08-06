use crate::parser::tests::{check_invalid_script, check_script_parser};
use boa_ast::{
    expression::{
        literal::{ArrayLiteral, Literal},
        operator::{
            assign::{AssignOp, AssignTarget},
            Assign,
        },
        Identifier, Parenthesized,
    },
    pattern::{ArrayPattern, ArrayPatternElement, ObjectPattern, ObjectPatternElement, Pattern},
    property::PropertyName,
    Expression, Statement,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;

#[test]
fn check_string() {
    // Check empty string
    check_script_parser(
        "\"\"",
        vec![Statement::Expression(Expression::from(Literal::from(Sym::EMPTY_STRING))).into()],
        &mut Interner::default(),
    );

    // Check non-empty string
    let interner = &mut Interner::default();
    check_script_parser(
        "\"hello\"",
        vec![Statement::Expression(Expression::from(Literal::from(
            interner.get_or_intern_static("hello", utf16!("hello")),
        )))
        .into()],
        interner,
    );
}

#[test]
fn check_destructuring_assignment_object_assignment_operator() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_script_parser(
        "({ a: a = 0 } = 0);",
        vec![
            Statement::Expression(Expression::Parenthesized(Parenthesized::new(
                Expression::Assign(Assign::new(
                    AssignOp::Assign,
                    AssignTarget::Pattern(Pattern::Object(ObjectPattern::from(vec![
                        ObjectPatternElement::SingleName {
                            name: PropertyName::from(a),
                            ident: Identifier::from(a),
                            default_init: Some(Literal::from(0).into()),
                        },
                    ]))),
                    Literal::from(0).into(),
                )),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn check_destructuring_assignment_object_invalid_assignment_operators() {
    check_invalid_script("({ a: a &&= 0 } = 0);");
    check_invalid_script("({ a: a ||= 0 } = 0);");
    check_invalid_script("({ a: a ??= 0 } = 0);");
    check_invalid_script("({ a: a *= 0 } = 0);");
    check_invalid_script("({ a: a /= 0 } = 0);");
    check_invalid_script("({ a: a %= 0 } = 0);");
    check_invalid_script("({ a: a += 0 } = 0);");
    check_invalid_script("({ a: a -= 0 } = 0);");
    check_invalid_script("({ a: a <<= 0 } = 0);");
    check_invalid_script("({ a: a >>= 0 } = 0);");
    check_invalid_script("({ a: a >>>= 0 } = 0);");
    check_invalid_script("({ a: a &= 0 } = 0);");
    check_invalid_script("({ a: a ^= 0 } = 0);");
    check_invalid_script("({ a: a |= 0 } = 0);");
    check_invalid_script("({ a: a **= 0 } = 0);");
}

#[test]
fn check_destructuring_assignment_array_assignment_operator() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_script_parser(
        "([ a = 0 ] = []);",
        vec![
            Statement::Expression(Expression::Parenthesized(Parenthesized::new(
                Expression::Assign(Assign::new(
                    AssignOp::Assign,
                    AssignTarget::Pattern(Pattern::Array(ArrayPattern::from(vec![
                        ArrayPatternElement::SingleName {
                            ident: Identifier::from(a),
                            default_init: Some(Literal::from(0).into()),
                        },
                    ]))),
                    ArrayLiteral::from([]).into(),
                )),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn check_destructuring_assignment_array_invalid_assignment_operators() {
    check_invalid_script("([ a &&= 0 ] = []);");
    check_invalid_script("([ a ||= 0 ] = []);");
    check_invalid_script("([ a ??= 0 ] = []);");
    check_invalid_script("([ a *= 0 ] = []);");
    check_invalid_script("([ a /= 0 ] = []);");
    check_invalid_script("([ a %= 0 ] = []);");
    check_invalid_script("([ a += 0 ] = []);");
    check_invalid_script("([ a -= 0 ] = []);");
    check_invalid_script("([ a <<= 0 ] = []);");
    check_invalid_script("([ a >>= 0 ] = []);");
    check_invalid_script("([ a >>>= 0 ] = []);");
    check_invalid_script("([ a &= 0 ] = []);");
    check_invalid_script("([ a ^= 0 ] = []);");
    check_invalid_script("([ a |= 0 ] = []);");
    check_invalid_script("([ a **= 0 ] = []);");
}
