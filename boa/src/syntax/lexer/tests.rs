//! Tests for the lexer.
#![allow(clippy::indexing_slicing)]

use super::*;
use crate::syntax::ast::Keyword;

fn span(start: (u32, u32), end: (u32, u32)) -> Span {
    Span::new(Position::new(start.0, start.1), Position::new(end.0, end.1))
}

#[test]
fn check_single_line_comment() {
    let s1 = "var \n//This is a comment\ntrue";
    let mut lexer = Lexer::new(s1);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::Keyword(Keyword::Var));
    assert_eq!(lexer.tokens[1].kind, TokenKind::LineTerminator);
    assert_eq!(lexer.tokens[2].kind, TokenKind::BooleanLiteral(true));
}

#[test]
fn check_multi_line_comment() {
    let s = "var /* await \n break \n*/ x";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::Keyword(Keyword::Var));
    assert_eq!(lexer.tokens[1].kind, TokenKind::identifier("x"));
}

#[test]
fn check_string() {
    let s = "'aaa' \"bbb\"";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::string_literal("aaa"));

    assert_eq!(lexer.tokens[1].kind, TokenKind::string_literal("bbb"));
}

#[test]
fn check_punctuators() {
    // https://tc39.es/ecma262/#sec-punctuators
    let s = "{ ( ) [ ] . ... ; , < > <= >= == != === !== \
             + - * % -- << >> >>> & | ^ ! ~ && || ? : \
             = += -= *= &= **= ++ ** <<= >>= >>>= &= |= ^= =>";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(
        lexer.tokens[0].kind,
        TokenKind::Punctuator(Punctuator::OpenBlock)
    );
    assert_eq!(
        lexer.tokens[1].kind,
        TokenKind::Punctuator(Punctuator::OpenParen)
    );
    assert_eq!(
        lexer.tokens[2].kind,
        TokenKind::Punctuator(Punctuator::CloseParen)
    );
    assert_eq!(
        lexer.tokens[3].kind,
        TokenKind::Punctuator(Punctuator::OpenBracket)
    );
    assert_eq!(
        lexer.tokens[4].kind,
        TokenKind::Punctuator(Punctuator::CloseBracket)
    );
    assert_eq!(lexer.tokens[5].kind, TokenKind::Punctuator(Punctuator::Dot));
    assert_eq!(
        lexer.tokens[6].kind,
        TokenKind::Punctuator(Punctuator::Spread)
    );
    assert_eq!(
        lexer.tokens[7].kind,
        TokenKind::Punctuator(Punctuator::Semicolon)
    );
    assert_eq!(
        lexer.tokens[8].kind,
        TokenKind::Punctuator(Punctuator::Comma)
    );
    assert_eq!(
        lexer.tokens[9].kind,
        TokenKind::Punctuator(Punctuator::LessThan)
    );
    assert_eq!(
        lexer.tokens[10].kind,
        TokenKind::Punctuator(Punctuator::GreaterThan)
    );
    assert_eq!(
        lexer.tokens[11].kind,
        TokenKind::Punctuator(Punctuator::LessThanOrEq)
    );
    assert_eq!(
        lexer.tokens[12].kind,
        TokenKind::Punctuator(Punctuator::GreaterThanOrEq)
    );
    assert_eq!(lexer.tokens[13].kind, TokenKind::Punctuator(Punctuator::Eq));
    assert_eq!(
        lexer.tokens[14].kind,
        TokenKind::Punctuator(Punctuator::NotEq)
    );
    assert_eq!(
        lexer.tokens[15].kind,
        TokenKind::Punctuator(Punctuator::StrictEq)
    );
    assert_eq!(
        lexer.tokens[16].kind,
        TokenKind::Punctuator(Punctuator::StrictNotEq)
    );
    assert_eq!(
        lexer.tokens[17].kind,
        TokenKind::Punctuator(Punctuator::Add)
    );
    assert_eq!(
        lexer.tokens[18].kind,
        TokenKind::Punctuator(Punctuator::Sub)
    );
    assert_eq!(
        lexer.tokens[19].kind,
        TokenKind::Punctuator(Punctuator::Mul)
    );
    assert_eq!(
        lexer.tokens[20].kind,
        TokenKind::Punctuator(Punctuator::Mod)
    );
    assert_eq!(
        lexer.tokens[21].kind,
        TokenKind::Punctuator(Punctuator::Dec)
    );
    assert_eq!(
        lexer.tokens[22].kind,
        TokenKind::Punctuator(Punctuator::LeftSh)
    );
    assert_eq!(
        lexer.tokens[23].kind,
        TokenKind::Punctuator(Punctuator::RightSh)
    );
    assert_eq!(
        lexer.tokens[24].kind,
        TokenKind::Punctuator(Punctuator::URightSh)
    );
    assert_eq!(
        lexer.tokens[25].kind,
        TokenKind::Punctuator(Punctuator::And)
    );
    assert_eq!(lexer.tokens[26].kind, TokenKind::Punctuator(Punctuator::Or));
    assert_eq!(
        lexer.tokens[27].kind,
        TokenKind::Punctuator(Punctuator::Xor)
    );
    assert_eq!(
        lexer.tokens[28].kind,
        TokenKind::Punctuator(Punctuator::Not)
    );
    assert_eq!(
        lexer.tokens[29].kind,
        TokenKind::Punctuator(Punctuator::Neg)
    );
    assert_eq!(
        lexer.tokens[30].kind,
        TokenKind::Punctuator(Punctuator::BoolAnd)
    );
    assert_eq!(
        lexer.tokens[31].kind,
        TokenKind::Punctuator(Punctuator::BoolOr)
    );
    assert_eq!(
        lexer.tokens[32].kind,
        TokenKind::Punctuator(Punctuator::Question)
    );
    assert_eq!(
        lexer.tokens[33].kind,
        TokenKind::Punctuator(Punctuator::Colon)
    );
    assert_eq!(
        lexer.tokens[34].kind,
        TokenKind::Punctuator(Punctuator::Assign)
    );
    assert_eq!(
        lexer.tokens[35].kind,
        TokenKind::Punctuator(Punctuator::AssignAdd)
    );
    assert_eq!(
        lexer.tokens[36].kind,
        TokenKind::Punctuator(Punctuator::AssignSub)
    );
    assert_eq!(
        lexer.tokens[37].kind,
        TokenKind::Punctuator(Punctuator::AssignMul)
    );
    assert_eq!(
        lexer.tokens[38].kind,
        TokenKind::Punctuator(Punctuator::AssignAnd)
    );
    assert_eq!(
        lexer.tokens[39].kind,
        TokenKind::Punctuator(Punctuator::AssignPow)
    );
    assert_eq!(
        lexer.tokens[40].kind,
        TokenKind::Punctuator(Punctuator::Inc)
    );
    assert_eq!(
        lexer.tokens[41].kind,
        TokenKind::Punctuator(Punctuator::Exp)
    );
    assert_eq!(
        lexer.tokens[42].kind,
        TokenKind::Punctuator(Punctuator::AssignLeftSh)
    );
    assert_eq!(
        lexer.tokens[43].kind,
        TokenKind::Punctuator(Punctuator::AssignRightSh)
    );
    assert_eq!(
        lexer.tokens[44].kind,
        TokenKind::Punctuator(Punctuator::AssignURightSh)
    );
    assert_eq!(
        lexer.tokens[45].kind,
        TokenKind::Punctuator(Punctuator::AssignAnd)
    );
    assert_eq!(
        lexer.tokens[46].kind,
        TokenKind::Punctuator(Punctuator::AssignOr)
    );
    assert_eq!(
        lexer.tokens[47].kind,
        TokenKind::Punctuator(Punctuator::AssignXor)
    );
    assert_eq!(
        lexer.tokens[48].kind,
        TokenKind::Punctuator(Punctuator::Arrow)
    );
}

#[test]
fn check_keywords() {
    // https://tc39.es/ecma262/#sec-keywords
    let s = "await break case catch class const continue debugger default delete \
             do else export extends finally for function if import in instanceof \
             new return super switch this throw try typeof var void while with yield";

    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::Keyword(Keyword::Await));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Keyword(Keyword::Break));
    assert_eq!(lexer.tokens[2].kind, TokenKind::Keyword(Keyword::Case));
    assert_eq!(lexer.tokens[3].kind, TokenKind::Keyword(Keyword::Catch));
    assert_eq!(lexer.tokens[4].kind, TokenKind::Keyword(Keyword::Class));
    assert_eq!(lexer.tokens[5].kind, TokenKind::Keyword(Keyword::Const));
    assert_eq!(lexer.tokens[6].kind, TokenKind::Keyword(Keyword::Continue));
    assert_eq!(lexer.tokens[7].kind, TokenKind::Keyword(Keyword::Debugger));
    assert_eq!(lexer.tokens[8].kind, TokenKind::Keyword(Keyword::Default));
    assert_eq!(lexer.tokens[9].kind, TokenKind::Keyword(Keyword::Delete));
    assert_eq!(lexer.tokens[10].kind, TokenKind::Keyword(Keyword::Do));
    assert_eq!(lexer.tokens[11].kind, TokenKind::Keyword(Keyword::Else));
    assert_eq!(lexer.tokens[12].kind, TokenKind::Keyword(Keyword::Export));
    assert_eq!(lexer.tokens[13].kind, TokenKind::Keyword(Keyword::Extends));
    assert_eq!(lexer.tokens[14].kind, TokenKind::Keyword(Keyword::Finally));
    assert_eq!(lexer.tokens[15].kind, TokenKind::Keyword(Keyword::For));
    assert_eq!(lexer.tokens[16].kind, TokenKind::Keyword(Keyword::Function));
    assert_eq!(lexer.tokens[17].kind, TokenKind::Keyword(Keyword::If));
    assert_eq!(lexer.tokens[18].kind, TokenKind::Keyword(Keyword::Import));
    assert_eq!(lexer.tokens[19].kind, TokenKind::Keyword(Keyword::In));
    assert_eq!(
        lexer.tokens[20].kind,
        TokenKind::Keyword(Keyword::InstanceOf)
    );
    assert_eq!(lexer.tokens[21].kind, TokenKind::Keyword(Keyword::New));
    assert_eq!(lexer.tokens[22].kind, TokenKind::Keyword(Keyword::Return));
    assert_eq!(lexer.tokens[23].kind, TokenKind::Keyword(Keyword::Super));
    assert_eq!(lexer.tokens[24].kind, TokenKind::Keyword(Keyword::Switch));
    assert_eq!(lexer.tokens[25].kind, TokenKind::Keyword(Keyword::This));
    assert_eq!(lexer.tokens[26].kind, TokenKind::Keyword(Keyword::Throw));
    assert_eq!(lexer.tokens[27].kind, TokenKind::Keyword(Keyword::Try));
    assert_eq!(lexer.tokens[28].kind, TokenKind::Keyword(Keyword::TypeOf));
    assert_eq!(lexer.tokens[29].kind, TokenKind::Keyword(Keyword::Var));
    assert_eq!(lexer.tokens[30].kind, TokenKind::Keyword(Keyword::Void));
    assert_eq!(lexer.tokens[31].kind, TokenKind::Keyword(Keyword::While));
    assert_eq!(lexer.tokens[32].kind, TokenKind::Keyword(Keyword::With));
    assert_eq!(lexer.tokens[33].kind, TokenKind::Keyword(Keyword::Yield));
}

#[test]
fn check_variable_definition_tokens() {
    let s = "let a = 'hello';";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::Keyword(Keyword::Let));
    assert_eq!(lexer.tokens[1].kind, TokenKind::identifier("a"));
    assert_eq!(
        lexer.tokens[2].kind,
        TokenKind::Punctuator(Punctuator::Assign)
    );
    assert_eq!(lexer.tokens[3].kind, TokenKind::string_literal("hello"));
}

#[test]
fn check_positions() {
    let s = r#"console.log("hello world\u{2764}"); // Test"#;
    // --------123456789
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    // The first column is 1 (not zero indexed)
    assert_eq!(lexer.tokens[0].span(), span((1, 1), (1, 7)));

    // Dot Token starts on column 8
    assert_eq!(lexer.tokens[1].span(), span((1, 8), (1, 8)));

    // Log Token starts on column 9
    assert_eq!(lexer.tokens[2].span(), span((1, 9), (1, 11)));

    // Open parenthesis token starts on column 12
    assert_eq!(lexer.tokens[3].span(), span((1, 12), (1, 12)));

    // String token starts on column 13
    assert_eq!(lexer.tokens[4].span(), span((1, 13), (1, 33)));

    // Close parenthesis token starts on column 34
    assert_eq!(lexer.tokens[5].span(), span((1, 34), (1, 34)));

    // Semi Colon token starts on column 35
    assert_eq!(lexer.tokens[6].span(), span((1, 35), (1, 35)));
}

#[test]
#[ignore]
fn two_divisions_in_expression() {
    let s = "    return a !== 0 || 1 / a === 1 / b;";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    // dbg!(&lexer.tokens);

    assert_eq!(lexer.tokens[11].span(), span((1, 37), (1, 37)));
}

#[test]
fn check_line_numbers() {
    let s = "x\ny\n";

    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");

    assert_eq!(lexer.tokens[0].span(), span((1, 1), (1, 1)));
    assert_eq!(lexer.tokens[1].span(), span((1, 2), (2, 1)));
    assert_eq!(lexer.tokens[2].span(), span((2, 1), (2, 1)));
    assert_eq!(lexer.tokens[3].span(), span((2, 2), (3, 1)));
}

// Increment/Decrement
#[test]
fn check_decrement_advances_lexer_2_places() {
    // Here we want an example of decrementing an integer
    let s = "let a = b--;";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[4].kind, TokenKind::Punctuator(Punctuator::Dec));
    // Decrementing means adding 2 characters '--', the lexer should consume it as a single token
    // and move the curser forward by 2, meaning the next token should be a semicolon
    assert_eq!(
        lexer.tokens[5].kind,
        TokenKind::Punctuator(Punctuator::Semicolon)
    );
}

#[test]
fn check_nan() {
    let mut lexer = Lexer::new("let a = NaN;");
    lexer.lex().expect("failed to lex");

    match lexer.tokens[3].kind {
        TokenKind::NumericLiteral(NumericLiteral::Rational(a)) => {
            assert!(a.is_nan());
        }
        ref other => panic!("Incorrect token kind found for NaN: {}", other),
    }
}

#[test]
fn numbers() {
    let mut lexer = Lexer::new(
        "1 2 0x34 056 7.89 42. 5e3 5e+3 5e-3 0b10 0O123 0999 1.0e1 1.0e-1 1.0E1 1E1 0.0 0.12",
    );

    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(1));
    assert_eq!(lexer.tokens[1].kind, TokenKind::numeric_literal(2));
    assert_eq!(lexer.tokens[2].kind, TokenKind::numeric_literal(52));
    assert_eq!(lexer.tokens[3].kind, TokenKind::numeric_literal(46));
    assert_eq!(lexer.tokens[4].kind, TokenKind::numeric_literal(7.89));
    assert_eq!(lexer.tokens[5].kind, TokenKind::numeric_literal(42.0));
    assert_eq!(lexer.tokens[6].kind, TokenKind::numeric_literal(5000.0));
    assert_eq!(lexer.tokens[7].kind, TokenKind::numeric_literal(5000.0));
    assert_eq!(lexer.tokens[8].kind, TokenKind::numeric_literal(0.005));
    assert_eq!(lexer.tokens[9].kind, TokenKind::numeric_literal(2));
    assert_eq!(lexer.tokens[10].kind, TokenKind::numeric_literal(83));
    assert_eq!(lexer.tokens[11].kind, TokenKind::numeric_literal(999));
    assert_eq!(lexer.tokens[12].kind, TokenKind::numeric_literal(10.0));
    assert_eq!(lexer.tokens[13].kind, TokenKind::numeric_literal(0.1));
    assert_eq!(lexer.tokens[14].kind, TokenKind::numeric_literal(10.0));
    assert_eq!(lexer.tokens[15].kind, TokenKind::numeric_literal(10.0));
    assert_eq!(lexer.tokens[16].kind, TokenKind::numeric_literal(0.0));
    assert_eq!(lexer.tokens[17].kind, TokenKind::numeric_literal(0.12));
}

#[test]
fn implicit_octal_edge_case() {
    let mut lexer = Lexer::new("044.5 094.5");

    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(36));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Dot));
    assert_eq!(lexer.tokens[2].kind, TokenKind::numeric_literal(5));

    assert_eq!(lexer.tokens[3].kind, TokenKind::numeric_literal(94.5));
}

#[test]
fn hexadecimal_edge_case() {
    let mut lexer = Lexer::new("0xffff.ff 0xffffff");

    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(0xffff));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Dot));
    assert_eq!(lexer.tokens[2].kind, TokenKind::identifier("ff"));

    assert_eq!(
        lexer.tokens[3].kind,
        TokenKind::numeric_literal(0x00ff_ffff)
    );
}

#[test]
fn single_number_without_semicolon() {
    let mut lexer = Lexer::new("1");
    lexer.lex().expect("failed to lex");
}

#[test]
fn number_followed_by_dot() {
    let mut lexer = Lexer::new("1..");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(1.0));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Dot));
}

#[test]
fn regex_literal() {
    let mut lexer = Lexer::new("/(?:)/");
    lexer.lex().expect("failed to lex");
    assert_eq!(
        lexer.tokens[0].kind,
        TokenKind::regular_expression_literal("(?:)", "".parse().unwrap())
    );
}

#[test]
fn regex_literal_flags() {
    let mut lexer = Lexer::new(r"/\/[^\/]*\/*/gmi");
    lexer.lex().expect("failed to lex");
    assert_eq!(
        lexer.tokens[0].kind,
        TokenKind::regular_expression_literal("\\/[^\\/]*\\/*", "gmi".parse().unwrap())
    );
}

#[test]
fn addition_no_spaces() {
    let mut lexer = Lexer::new("1+1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(1));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].kind, TokenKind::numeric_literal(1));
}

#[test]
fn addition_no_spaces_left_side() {
    let mut lexer = Lexer::new("1+ 1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(1));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].kind, TokenKind::numeric_literal(1));
}

#[test]
fn addition_no_spaces_right_side() {
    let mut lexer = Lexer::new("1 +1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(1));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].kind, TokenKind::numeric_literal(1));
}

#[test]
fn addition_no_spaces_e_number_left_side() {
    let mut lexer = Lexer::new("1e2+ 1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(100.0));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].kind, TokenKind::numeric_literal(1));
}

#[test]
fn addition_no_spaces_e_number_right_side() {
    let mut lexer = Lexer::new("1 +1e3");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(1));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].kind, TokenKind::numeric_literal(1000.0));
}

#[test]
fn addition_no_spaces_e_number() {
    let mut lexer = Lexer::new("1e3+1e11");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].kind, TokenKind::numeric_literal(1000.0));
    assert_eq!(lexer.tokens[1].kind, TokenKind::Punctuator(Punctuator::Add));
    assert_eq!(
        lexer.tokens[2].kind,
        TokenKind::numeric_literal(100_000_000_000.0)
    );
}
