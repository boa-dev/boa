//! Tests for the lexer.
#![allow(clippy::indexing_slicing)]

use crate::lexer::{
    template::TemplateString, token::Numeric, Cursor, Error, Interner, Lexer, Position, Punctuator,
    Read, Span, TokenKind,
};

use boa_ast::Keyword;
use boa_interner::Sym;
use boa_macros::utf16;

use std::str;

fn span(start: (u32, u32), end: (u32, u32)) -> Span {
    Span::new(Position::new(start.0, start.1), Position::new(end.0, end.1))
}

fn expect_tokens<R>(lexer: &mut Lexer<R>, expected: &[TokenKind], interner: &mut Interner)
where
    R: Read,
{
    for expect in expected.iter() {
        assert_eq!(&lexer.next(interner).unwrap().unwrap().kind(), &expect);
    }

    assert!(
        lexer.next(interner).unwrap().is_none(),
        "Unexpected extra token lexed at end of input"
    );
}

#[test]
fn check_single_line_comment() {
    let s1 = "var \n//This is a comment\ntrue";
    let mut lexer = Lexer::new(s1.as_bytes());
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::Keyword((Keyword::Var, false)),
        TokenKind::LineTerminator,
        TokenKind::LineTerminator,
        TokenKind::BooleanLiteral(true),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_single_line_comment_with_crlf_ending() {
    let s1 = "var \r\n//This is a comment\r\ntrue";
    let mut lexer = Lexer::new(s1.as_bytes());
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::Keyword((Keyword::Var, false)),
        TokenKind::LineTerminator,
        TokenKind::LineTerminator,
        TokenKind::BooleanLiteral(true),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_multi_line_comment() {
    let s = "var /* await \n break \n*/ x";
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let sym = interner.get_or_intern_static("x", utf16!("x"));
    let expected = [
        TokenKind::Keyword((Keyword::Var, false)),
        TokenKind::LineTerminator,
        TokenKind::identifier(sym),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_identifier() {
    let s = "x x1 _x $x __ $$ Ѐ ЀЀ x\u{200C}\u{200D} \\u0078 \\u0078\\u0078 \\u{0078}x\\u{0078}";
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::identifier(interner.get_or_intern_static("x", utf16!("x"))),
        TokenKind::identifier(interner.get_or_intern_static("x1", utf16!("x1"))),
        TokenKind::identifier(interner.get_or_intern_static("_x", utf16!("_x"))),
        TokenKind::identifier(interner.get_or_intern_static("$x", utf16!("$x"))),
        TokenKind::identifier(interner.get_or_intern_static("__", utf16!("__"))),
        TokenKind::identifier(interner.get_or_intern_static("$$", utf16!("$$"))),
        TokenKind::identifier(interner.get_or_intern_static("Ѐ", utf16!("Ѐ"))),
        TokenKind::identifier(interner.get_or_intern_static("ЀЀ", utf16!("ЀЀ"))),
        TokenKind::identifier(
            interner.get_or_intern_static("x\u{200C}\u{200D}", utf16!("x\u{200C}\u{200D}")),
        ),
        TokenKind::identifier(interner.get_or_intern_static("x", utf16!("x"))),
        TokenKind::identifier(interner.get_or_intern_static("xx", utf16!("xx"))),
        TokenKind::identifier(interner.get_or_intern_static("xxx", utf16!("xxx"))),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_invalid_identifier_start() {
    let invalid_identifier_starts = ["\u{200C}", "\u{200D}", "😀"];

    for s in &invalid_identifier_starts {
        let mut lexer = Lexer::new(s.as_bytes());
        let interner = &mut Interner::default();
        lexer
            .next(interner)
            .expect_err("Invalid identifier start not rejected as expected");
    }
}

#[test]
fn check_invalid_identifier_part() {
    let invalid_identifier_parts = [" ", "\n", ".", "*", "😀", "\u{007F}"];
    let interner = &mut Interner::default();

    let sym = interner.get_or_intern_static("x", utf16!("x"));
    for part in &invalid_identifier_parts {
        let s = String::from("x") + part;
        let mut lexer = Lexer::new(s.as_bytes());
        let interner = &mut Interner::default();
        assert_eq!(
            lexer.next(interner).unwrap().unwrap().kind(),
            &TokenKind::identifier(sym)
        );
    }
}

#[test]
fn check_string() {
    let s = "'aaa' \"bbb\"";
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let a_sym = interner.get_or_intern_static("aaa", utf16!("aaa"));
    let b_sym = interner.get_or_intern_static("bbb", utf16!("bbb"));
    let expected = [
        TokenKind::string_literal(a_sym),
        TokenKind::string_literal(b_sym),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_template_literal_simple() {
    let s = "`I'm a template literal`";
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let sym =
        interner.get_or_intern_static("I'm a template literal", utf16!("I'm a template literal"));

    assert_eq!(
        lexer.next(interner).unwrap().unwrap().kind(),
        &TokenKind::template_no_substitution(TemplateString::new(sym, Position::new(1, 1)))
    );
}

#[test]
fn check_template_literal_unterminated() {
    let s = "`I'm a template";
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    lexer
        .next(interner)
        .expect_err("Lexer did not handle unterminated literal with error");
}

#[test]
fn check_punctuators() {
    // https://tc39.es/ecma262/#sec-punctuators
    let s = "{ ( ) [ ] . ... ; , < > <= >= == != === !== \
             + - * % -- << >> >>> & | ^ ! ~ && || ? : \
             = += -= *= &= **= ++ ** <<= >>= >>>= &= |= ^= => ?? ??= &&= ||= ?.";
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::Punctuator(Punctuator::OpenBlock),
        TokenKind::Punctuator(Punctuator::OpenParen),
        TokenKind::Punctuator(Punctuator::CloseParen),
        TokenKind::Punctuator(Punctuator::OpenBracket),
        TokenKind::Punctuator(Punctuator::CloseBracket),
        TokenKind::Punctuator(Punctuator::Dot),
        TokenKind::Punctuator(Punctuator::Spread),
        TokenKind::Punctuator(Punctuator::Semicolon),
        TokenKind::Punctuator(Punctuator::Comma),
        TokenKind::Punctuator(Punctuator::LessThan),
        TokenKind::Punctuator(Punctuator::GreaterThan),
        TokenKind::Punctuator(Punctuator::LessThanOrEq),
        TokenKind::Punctuator(Punctuator::GreaterThanOrEq),
        TokenKind::Punctuator(Punctuator::Eq),
        TokenKind::Punctuator(Punctuator::NotEq),
        TokenKind::Punctuator(Punctuator::StrictEq),
        TokenKind::Punctuator(Punctuator::StrictNotEq),
        TokenKind::Punctuator(Punctuator::Add),
        TokenKind::Punctuator(Punctuator::Sub),
        TokenKind::Punctuator(Punctuator::Mul),
        TokenKind::Punctuator(Punctuator::Mod),
        TokenKind::Punctuator(Punctuator::Dec),
        TokenKind::Punctuator(Punctuator::LeftSh),
        TokenKind::Punctuator(Punctuator::RightSh),
        TokenKind::Punctuator(Punctuator::URightSh),
        TokenKind::Punctuator(Punctuator::And),
        TokenKind::Punctuator(Punctuator::Or),
        TokenKind::Punctuator(Punctuator::Xor),
        TokenKind::Punctuator(Punctuator::Not),
        TokenKind::Punctuator(Punctuator::Neg),
        TokenKind::Punctuator(Punctuator::BoolAnd),
        TokenKind::Punctuator(Punctuator::BoolOr),
        TokenKind::Punctuator(Punctuator::Question),
        TokenKind::Punctuator(Punctuator::Colon),
        TokenKind::Punctuator(Punctuator::Assign),
        TokenKind::Punctuator(Punctuator::AssignAdd),
        TokenKind::Punctuator(Punctuator::AssignSub),
        TokenKind::Punctuator(Punctuator::AssignMul),
        TokenKind::Punctuator(Punctuator::AssignAnd),
        TokenKind::Punctuator(Punctuator::AssignPow),
        TokenKind::Punctuator(Punctuator::Inc),
        TokenKind::Punctuator(Punctuator::Exp),
        TokenKind::Punctuator(Punctuator::AssignLeftSh),
        TokenKind::Punctuator(Punctuator::AssignRightSh),
        TokenKind::Punctuator(Punctuator::AssignURightSh),
        TokenKind::Punctuator(Punctuator::AssignAnd),
        TokenKind::Punctuator(Punctuator::AssignOr),
        TokenKind::Punctuator(Punctuator::AssignXor),
        TokenKind::Punctuator(Punctuator::Arrow),
        TokenKind::Punctuator(Punctuator::Coalesce),
        TokenKind::Punctuator(Punctuator::AssignCoalesce),
        TokenKind::Punctuator(Punctuator::AssignBoolAnd),
        TokenKind::Punctuator(Punctuator::AssignBoolOr),
        TokenKind::Punctuator(Punctuator::Optional),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_keywords() {
    // https://tc39.es/ecma262/#sec-keywords
    let s = "await break case catch class const continue debugger default delete \
             do else export extends finally for function if import in instanceof \
             new return super switch this throw try typeof var void while with yield";

    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::Keyword((Keyword::Await, false)),
        TokenKind::Keyword((Keyword::Break, false)),
        TokenKind::Keyword((Keyword::Case, false)),
        TokenKind::Keyword((Keyword::Catch, false)),
        TokenKind::Keyword((Keyword::Class, false)),
        TokenKind::Keyword((Keyword::Const, false)),
        TokenKind::Keyword((Keyword::Continue, false)),
        TokenKind::Keyword((Keyword::Debugger, false)),
        TokenKind::Keyword((Keyword::Default, false)),
        TokenKind::Keyword((Keyword::Delete, false)),
        TokenKind::Keyword((Keyword::Do, false)),
        TokenKind::Keyword((Keyword::Else, false)),
        TokenKind::Keyword((Keyword::Export, false)),
        TokenKind::Keyword((Keyword::Extends, false)),
        TokenKind::Keyword((Keyword::Finally, false)),
        TokenKind::Keyword((Keyword::For, false)),
        TokenKind::Keyword((Keyword::Function, false)),
        TokenKind::Keyword((Keyword::If, false)),
        TokenKind::Keyword((Keyword::Import, false)),
        TokenKind::Keyword((Keyword::In, false)),
        TokenKind::Keyword((Keyword::InstanceOf, false)),
        TokenKind::Keyword((Keyword::New, false)),
        TokenKind::Keyword((Keyword::Return, false)),
        TokenKind::Keyword((Keyword::Super, false)),
        TokenKind::Keyword((Keyword::Switch, false)),
        TokenKind::Keyword((Keyword::This, false)),
        TokenKind::Keyword((Keyword::Throw, false)),
        TokenKind::Keyword((Keyword::Try, false)),
        TokenKind::Keyword((Keyword::TypeOf, false)),
        TokenKind::Keyword((Keyword::Var, false)),
        TokenKind::Keyword((Keyword::Void, false)),
        TokenKind::Keyword((Keyword::While, false)),
        TokenKind::Keyword((Keyword::With, false)),
        TokenKind::Keyword((Keyword::Yield, false)),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_variable_definition_tokens() {
    let s = "let a = 'hello';";
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let a_sym = interner.get_or_intern_static("a", utf16!("a"));
    let hello_sym = interner.get_or_intern_static("hello", utf16!("hello"));
    let expected = [
        TokenKind::Keyword((Keyword::Let, false)),
        TokenKind::identifier(a_sym),
        TokenKind::Punctuator(Punctuator::Assign),
        TokenKind::string_literal(hello_sym),
        TokenKind::Punctuator(Punctuator::Semicolon),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn check_positions() {
    let s = r#"console.log("hello world"); // Test"#;
    // --------123456789
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    // The first column is 1 (not zero indexed)
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 1), (1, 8))
    );

    // Dot Token starts on column 8
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 8), (1, 9))
    );

    // Log Token starts on column 9
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 9), (1, 12))
    );

    // Open parenthesis token starts on column 12
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 12), (1, 13))
    );

    // String token starts on column 13
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 13), (1, 26))
    );

    // Close parenthesis token starts on column 26.
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 26), (1, 27))
    );

    // Semi Colon token starts on column 35
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 27), (1, 28))
    );
}

#[test]
fn check_positions_codepoint() {
    let s = r#"console.log("hello world\u{2764}"); // Test"#;
    // --------123456789
    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    // The first column is 1 (not zero indexed)
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 1), (1, 8))
    );

    // Dot Token starts on column 8
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 8), (1, 9))
    );

    // Log Token starts on column 9
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 9), (1, 12))
    );

    // Open parenthesis token starts on column 12
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 12), (1, 13))
    );

    // String token starts on column 13
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 13), (1, 34))
    );

    // Close parenthesis token starts on column 34
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 34), (1, 35))
    );

    // Semi Colon token starts on column 35
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 35), (1, 36))
    );
}

#[test]
fn check_line_numbers() {
    let s = "x\ny\n";

    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 1), (1, 2))
    );
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((1, 2), (2, 1))
    );
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((2, 1), (2, 2))
    );
    assert_eq!(
        lexer.next(interner).unwrap().unwrap().span(),
        span((2, 2), (3, 1))
    );
}

// Increment/Decrement
#[test]
fn check_decrement_advances_lexer_2_places() {
    // Here we want an example of decrementing an integer
    let mut lexer = Lexer::new(&b"let a = b--;"[..]);
    let interner = &mut Interner::default();

    for _ in 0..4 {
        lexer.next(interner).unwrap();
    }

    assert_eq!(
        lexer.next(interner).unwrap().unwrap().kind(),
        &TokenKind::Punctuator(Punctuator::Dec)
    );
    // Decrementing means adding 2 characters '--', the lexer should consume it as a single token
    // and move the curser forward by 2, meaning the next token should be a semicolon

    assert_eq!(
        lexer.next(interner).unwrap().unwrap().kind(),
        &TokenKind::Punctuator(Punctuator::Semicolon)
    );
}

#[test]
fn single_int() {
    let mut lexer = Lexer::new(&b"52"[..]);
    let interner = &mut Interner::default();

    let expected = [TokenKind::numeric_literal(52)];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn numbers() {
    let mut lexer = Lexer::new(
        "1 2 0x34 056 7.89 42. 5e3 5e+3 5e-3 0b10 0O123 0999 1.0e1 1.0e-1 1.0E1 1E1 0.0 0.12 -32"
            .as_bytes(),
    );
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(1),
        TokenKind::numeric_literal(2),
        TokenKind::numeric_literal(52),
        TokenKind::numeric_literal(46),
        TokenKind::numeric_literal(7.89),
        TokenKind::numeric_literal(42),
        TokenKind::numeric_literal(5000),
        TokenKind::numeric_literal(5000),
        TokenKind::numeric_literal(0.005),
        TokenKind::numeric_literal(2),
        TokenKind::numeric_literal(83),
        TokenKind::numeric_literal(999),
        TokenKind::numeric_literal(10),
        TokenKind::numeric_literal(0.1),
        TokenKind::numeric_literal(10),
        TokenKind::numeric_literal(10),
        TokenKind::numeric_literal(0),
        TokenKind::numeric_literal(0.12),
        TokenKind::Punctuator(Punctuator::Sub),
        TokenKind::numeric_literal(32),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn numbers_with_separators() {
    let mut lexer = Lexer::new(
        "1_0 2_0 0x3_4 056 7.8_9 4_2. 5_0e2 5_0e+2 5_0e-4 0b1_0 1_0.0_0e2 1.0E-0_1 -3_2".as_bytes(),
    );
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(10),
        TokenKind::numeric_literal(20),
        TokenKind::numeric_literal(52),
        TokenKind::numeric_literal(46),
        TokenKind::numeric_literal(7.89),
        TokenKind::numeric_literal(42),
        TokenKind::numeric_literal(5000),
        TokenKind::numeric_literal(5000),
        TokenKind::numeric_literal(0.005),
        TokenKind::numeric_literal(2),
        TokenKind::numeric_literal(1000),
        TokenKind::numeric_literal(0.1),
        TokenKind::Punctuator(Punctuator::Sub),
        TokenKind::numeric_literal(32),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn numbers_with_bad_separators() {
    let numbers = [
        "0b_10", "0x_10", "10_", "1._10", "1e+_10", "1E_10", "10__00",
    ];

    for n in &numbers {
        let mut lexer = Lexer::new(n.as_bytes());
        let interner = &mut Interner::default();
        assert!(lexer.next(interner).is_err());
    }
}

#[test]
fn big_exp_numbers() {
    let mut lexer = Lexer::new(&b"1.0e25 1.0e36 9.0e50"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(10_000_000_000_000_000_000_000_000.0),
        TokenKind::numeric_literal(1_000_000_000_000_000_000_000_000_000_000_000_000.0),
        TokenKind::numeric_literal(
            900_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000.0,
        ),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn big_literal_numbers() {
    let mut lexer = Lexer::new(&b"10000000000000000000000000"[..]);
    let interner = &mut Interner::default();

    let expected = [TokenKind::numeric_literal(
        10_000_000_000_000_000_000_000_000.0,
    )];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn implicit_octal_edge_case() {
    let mut lexer = Lexer::new(&b"044.5 094.5"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(36),
        TokenKind::numeric_literal(0.5),
        TokenKind::numeric_literal(94.5),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn hexadecimal_edge_case() {
    let mut lexer = Lexer::new(&b"0xffff.ff 0xffffff"[..]);
    let interner = &mut Interner::default();

    let sym = interner.get_or_intern_static("ff", utf16!("ff"));
    let expected = [
        TokenKind::numeric_literal(0xffff),
        TokenKind::Punctuator(Punctuator::Dot),
        TokenKind::identifier(sym),
        TokenKind::numeric_literal(0x00ff_ffff),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn single_number_without_semicolon() {
    let mut lexer = Lexer::new(&b"1"[..]);
    let interner = &mut Interner::default();

    if let Some(x) = lexer.next(interner).unwrap() {
        assert_eq!(x.kind(), &TokenKind::numeric_literal(Numeric::Integer(1)));
    } else {
        panic!("Failed to lex 1 without semicolon");
    }
}

#[test]
fn number_followed_by_dot() {
    let mut lexer = Lexer::new(&b"1.."[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(1),
        TokenKind::Punctuator(Punctuator::Dot),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn regex_literal() {
    let mut lexer = Lexer::new(&b"/(?:)/"[..]);
    let interner = &mut Interner::default();

    let expected = [TokenKind::regular_expression_literal(
        interner.get_or_intern_static("(?:)", utf16!("(?:)")),
        Sym::EMPTY_STRING,
    )];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn regex_equals_following_assignment() {
    let mut lexer = Lexer::new(&b"const myRegex = /=/;"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::Keyword((Keyword::Const, false)),
        TokenKind::identifier(interner.get_or_intern_static("myRegex", utf16!("myRegex"))),
        TokenKind::Punctuator(Punctuator::Assign),
        TokenKind::regular_expression_literal(
            interner.get_or_intern_static("=", utf16!("=")),
            Sym::EMPTY_STRING,
        ),
        TokenKind::Punctuator(Punctuator::Semicolon),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn regex_literal_flags() {
    let mut lexer = Lexer::new(&br"/\/[^\/]*\/*/gmi"[..]);
    let interner = &mut Interner::default();

    let expected = [TokenKind::regular_expression_literal(
        interner.get_or_intern_static("\\/[^\\/]*\\/*", utf16!("\\/[^\\/]*\\/*")),
        interner.get_or_intern_static("gim", utf16!("gim")),
    )];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn regex_literal_flags_err() {
    let mut lexer = Lexer::new(&br"/\/[^\/]*\/*/gmip"[..]);
    let interner = &mut Interner::default();

    lexer
        .next(interner)
        .expect_err("Lexer did not handle regex literal with error");

    let mut lexer = Lexer::new(&br"/\/[^\/]*\/*/gmii"[..]);
    let interner = &mut Interner::default();

    lexer
        .next(interner)
        .expect_err("Lexer did not handle regex literal with error");
}

#[test]
fn addition_no_spaces() {
    let mut lexer = Lexer::new(&b"1+1"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(1),
        TokenKind::Punctuator(Punctuator::Add),
        TokenKind::numeric_literal(1),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn addition_no_spaces_left_side() {
    let mut lexer = Lexer::new(&b"1+ 1"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(1),
        TokenKind::Punctuator(Punctuator::Add),
        TokenKind::numeric_literal(1),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn addition_no_spaces_right_side() {
    let mut lexer = Lexer::new(&b"1 +1"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(1),
        TokenKind::Punctuator(Punctuator::Add),
        TokenKind::numeric_literal(1),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn addition_no_spaces_e_number_left_side() {
    let mut lexer = Lexer::new(&b"1e2+ 1"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(100),
        TokenKind::Punctuator(Punctuator::Add),
        TokenKind::numeric_literal(1),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn addition_no_spaces_e_number_right_side() {
    let mut lexer = Lexer::new(&b"1 +1e3"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(1),
        TokenKind::Punctuator(Punctuator::Add),
        TokenKind::numeric_literal(1000),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn addition_no_spaces_e_number() {
    let mut lexer = Lexer::new(&b"1e3+1e11"[..]);
    let interner = &mut Interner::default();

    let expected = [
        TokenKind::numeric_literal(1000),
        TokenKind::Punctuator(Punctuator::Add),
        TokenKind::numeric_literal(100_000_000_000.0),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn take_while_ascii_pred_simple() {
    let mut cur = Cursor::new(&b"abcdefghijk"[..]);

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_ascii_pred(&mut buf, &|c| c == 'a' || c == 'b' || c == 'c')
        .unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "abc");
}

#[test]
fn take_while_ascii_pred_immediate_stop() {
    let mut cur = Cursor::new(&b"abcdefghijk"[..]);

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_ascii_pred(&mut buf, &|_| false).unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "");
}

#[test]
fn take_while_ascii_pred_entire_str() {
    let mut cur = Cursor::new(&b"abcdefghijk"[..]);

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_ascii_pred(&mut buf, &|_| true).unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "abcdefghijk");
}

#[test]
fn take_while_ascii_pred_non_ascii_stop() {
    let mut cur = Cursor::new("abcde😀fghijk".as_bytes());

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_ascii_pred(&mut buf, &|_| true).unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "abcde");
}

#[test]
fn take_while_char_pred_simple() {
    let mut cur = Cursor::new(&b"abcdefghijk"[..]);

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_char_pred(&mut buf, &|c| {
        c == 'a' as u32 || c == 'b' as u32 || c == 'c' as u32
    })
    .unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "abc");
}

#[test]
fn take_while_char_pred_immediate_stop() {
    let mut cur = Cursor::new(&b"abcdefghijk"[..]);

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_char_pred(&mut buf, &|_| false).unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "");
}

#[test]
fn take_while_char_pred_entire_str() {
    let mut cur = Cursor::new(&b"abcdefghijk"[..]);

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_char_pred(&mut buf, &|_| true).unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "abcdefghijk");
}

#[test]
fn take_while_char_pred_utf8_char() {
    let mut cur = Cursor::new("abc😀defghijk".as_bytes());

    let mut buf: Vec<u8> = Vec::new();

    cur.take_while_char_pred(&mut buf, &|c| {
        if let Ok(c) = char::try_from(c) {
            c == 'a' || c == 'b' || c == 'c' || c == '😀'
        } else {
            false
        }
    })
    .unwrap();

    assert_eq!(str::from_utf8(buf.as_slice()).unwrap(), "abc😀");
}

#[test]
fn illegal_following_numeric_literal() {
    // Checks as per https://tc39.es/ecma262/#sec-literals-numeric-literals that a NumericLiteral cannot
    // be immediately followed by an IdentifierStart or DecimalDigit.

    // Decimal Digit
    let mut lexer = Lexer::new(&b"11.6n3"[..]);
    let interner = &mut Interner::default();

    let err = lexer
        .next(interner)
        .expect_err("DecimalDigit following NumericLiteral not rejected as expected");
    if let Error::Syntax(_, pos) = err {
        assert_eq!(pos, Position::new(1, 5));
    } else {
        panic!("invalid error type");
    }

    // Identifier Start
    let mut lexer = Lexer::new(&b"17.4$"[..]);
    let interner = &mut Interner::default();

    if let Error::Syntax(_, pos) = lexer
        .next(interner)
        .expect_err("IdentifierStart '$' following NumericLiteral not rejected as expected")
    {
        assert_eq!(pos, Position::new(1, 5));
    } else {
        panic!("invalid error type");
    }

    let mut lexer = Lexer::new(&b"17.4_"[..]);
    let interner = &mut Interner::default();

    if let Error::Syntax(_, pos) = lexer
        .next(interner)
        .expect_err("IdentifierStart '_' following NumericLiteral not rejected as expected")
    {
        assert_eq!(pos, Position::new(1, 5));
    } else {
        panic!("invalid error type");
    }
}

#[test]
fn string_codepoint_with_no_braces() {
    let mut lexer = Lexer::new(&br#""test\uD38Dtest""#[..]);
    let interner = &mut Interner::default();

    assert!(lexer.next(interner).is_ok());
}

#[test]
#[ignore]
fn illegal_code_point_following_numeric_literal() {
    // Checks as per https://tc39.es/ecma262/#sec-literals-numeric-literals that a NumericLiteral cannot
    // be immediately followed by an IdentifierStart where the IdentifierStart
    let mut lexer = Lexer::new(&br#"17.4\u{2764}"#[..]);
    let interner = &mut Interner::default();

    assert!(
        lexer.next(interner).is_err(),
        "{}",
        r#"IdentifierStart \u{2764} following NumericLiteral not rejected as expected"#
    );
}

#[test]
fn string_unicode() {
    let s = r#"'中文';"#;

    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let sym = interner.get_or_intern_static("中文", utf16!("中文"));
    let expected = [
        TokenKind::StringLiteral(sym),
        TokenKind::Punctuator(Punctuator::Semicolon),
    ];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn string_unicode_escape_with_braces() {
    let mut lexer = Lexer::new(&br#"'{\u{20ac}\u{a0}\u{a0}}'"#[..]);
    let interner = &mut Interner::default();

    let sym =
        interner.get_or_intern_static("{\u{20ac}\u{a0}\u{a0}}", utf16!("{\u{20ac}\u{a0}\u{a0}}"));
    let expected = [TokenKind::StringLiteral(sym)];

    expect_tokens(&mut lexer, &expected, interner);

    lexer = Lexer::new(&br#"\u{{a0}"#[..]);

    if let Error::Syntax(_, pos) = lexer
        .next(interner)
        .expect_err("Malformed Unicode character sequence expected")
    {
        assert_eq!(pos, Position::new(1, 1));
    } else {
        panic!("invalid error type");
    }

    lexer = Lexer::new(&br#"\u{{a0}}"#[..]);

    if let Error::Syntax(_, pos) = lexer
        .next(interner)
        .expect_err("Malformed Unicode character sequence expected")
    {
        assert_eq!(pos, Position::new(1, 1));
    } else {
        panic!("invalid error type");
    }
}

#[test]
fn string_unicode_escape_with_braces_2() {
    let s = r#"'\u{20ac}\u{a0}\u{a0}'"#;

    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let sym = interner.get_or_intern_static("\u{20ac}\u{a0}\u{a0}", utf16!("\u{20ac}\u{a0}\u{a0}"));
    let expected = [TokenKind::StringLiteral(sym)];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn string_with_single_escape() {
    let s = r#"'\Б'"#;

    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let sym = interner.get_or_intern_static("Б", utf16!("Б"));
    let expected = [TokenKind::StringLiteral(sym)];

    expect_tokens(&mut lexer, &expected, interner);
}

#[test]
fn string_legacy_octal_escape() {
    let test_cases = [
        (r#"'\3'"#, "\u{3}"),
        (r#"'\03'"#, "\u{3}"),
        (r#"'\003'"#, "\u{3}"),
        (r#"'\0003'"#, "\u{0}3"),
        (r#"'\43'"#, "#"),
        (r#"'\043'"#, "#"),
        (r#"'\101'"#, "A"),
    ];

    for (s, expected) in &test_cases {
        let mut lexer = Lexer::new(s.as_bytes());
        let interner = &mut Interner::default();

        let sym = interner.get_or_intern(expected.encode_utf16().collect::<Vec<_>>().as_slice());
        let expected_tokens = [TokenKind::StringLiteral(sym)];

        expect_tokens(&mut lexer, &expected_tokens, interner);
    }

    for (s, _) in &test_cases {
        let mut lexer = Lexer::new(s.as_bytes());
        let interner = &mut Interner::default();
        lexer.set_strict_mode(true);

        if let Error::Syntax(_, pos) = lexer
            .next(interner)
            .expect_err("Octal-escape in strict mode not rejected as expected")
        {
            assert_eq!(pos, Position::new(1, 2));
        } else {
            panic!("invalid error type");
        }
    }
}

#[test]
fn string_zero_escape() {
    let test_cases = [(r#"'\0'"#, "\u{0}"), (r#"'\0A'"#, "\u{0}A")];

    for (s, expected) in &test_cases {
        let mut lexer = Lexer::new(s.as_bytes());
        let interner = &mut Interner::default();

        let sym = interner.get_or_intern(expected.encode_utf16().collect::<Vec<_>>().as_slice());
        let expected_tokens = [TokenKind::StringLiteral(sym)];

        expect_tokens(&mut lexer, &expected_tokens, interner);
    }
}

#[test]
fn string_non_octal_decimal_escape() {
    let test_cases = [(r#"'\8'"#, "8"), (r#"'\9'"#, "9")];

    for (s, expected) in &test_cases {
        let mut lexer = Lexer::new(s.as_bytes());
        let interner = &mut Interner::default();

        let sym = interner.get_or_intern(expected.encode_utf16().collect::<Vec<_>>().as_slice());
        let expected_tokens = [TokenKind::StringLiteral(sym)];

        expect_tokens(&mut lexer, &expected_tokens, interner);
    }

    for (s, _) in &test_cases {
        let mut lexer = Lexer::new(s.as_bytes());
        let interner = &mut Interner::default();
        lexer.set_strict_mode(true);

        if let Error::Syntax(_, pos) = lexer
            .next(interner)
            .expect_err("Non-octal-decimal-escape in strict mode not rejected as expected")
        {
            assert_eq!(pos, Position::new(1, 2));
        } else {
            panic!("invalid error type");
        }
    }
}

#[test]
fn string_line_continuation() {
    let s = "'hello \\\nworld'";

    let mut lexer = Lexer::new(s.as_bytes());
    let interner = &mut Interner::default();

    let sym = interner.get_or_intern_static("hello world", utf16!("hello world"));
    let expected_tokens = [TokenKind::StringLiteral(sym)];

    expect_tokens(&mut lexer, &expected_tokens, interner);
}

mod carriage_return {
    use super::*;

    fn expect_tokens_with_lines(lines: usize, src: &str) {
        let mut lexer = Lexer::new(src.as_bytes());
        let interner = &mut Interner::default();

        let mut expected = Vec::with_capacity(lines + 2);
        expected.push(TokenKind::Punctuator(Punctuator::Sub));
        for _ in 0..lines {
            expected.push(TokenKind::LineTerminator);
        }
        expected.push(TokenKind::NumericLiteral(Numeric::Integer(3)));

        expect_tokens(&mut lexer, &expected, interner);
    }

    #[test]
    fn regular_line() {
        expect_tokens_with_lines(1, "-\n3");
        expect_tokens_with_lines(2, "-\n\n3");
        expect_tokens_with_lines(3, "-\n\n\n3");
    }

    #[test]
    fn carriage_return() {
        expect_tokens_with_lines(1, "-\r3");
        expect_tokens_with_lines(2, "-\r\r3");
        expect_tokens_with_lines(3, "-\r\r\r3");
    }

    #[test]
    fn windows_line() {
        expect_tokens_with_lines(1, "-\r\n3");
        expect_tokens_with_lines(2, "-\r\n\r\n3");
        expect_tokens_with_lines(3, "-\r\n\r\n\r\n3");
    }

    #[test]
    fn mixed_line() {
        expect_tokens_with_lines(2, "-\r\n\n3");
        expect_tokens_with_lines(2, "-\n\r3");
        expect_tokens_with_lines(3, "-\r\n\n\r3");
    }
}
