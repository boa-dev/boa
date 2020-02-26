//! Tests for the lexer.
#![allow(clippy::indexing_slicing)]

use super::*;
use crate::syntax::ast::keyword::Keyword;

#[test]
fn check_single_line_comment() {
    let s1 = "var \n//=\nx";
    let mut lexer = Lexer::new(s1);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::Keyword(Keyword::Var));
    assert_eq!(lexer.tokens[1].data, TokenData::Comment("//=".to_owned()));
    assert_eq!(lexer.tokens[2].data, TokenData::Identifier("x".to_string()));
}

#[test]
fn check_multi_line_comment() {
    let s = "var /* await \n break \n*/ x";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::Keyword(Keyword::Var));
    assert_eq!(
        lexer.tokens[1].data,
        TokenData::Comment("/* await \n break \n*/".to_owned())
    );
    assert_eq!(lexer.tokens[2].data, TokenData::Identifier("x".to_string()));
}

#[test]
fn check_string() {
    let s = "'aaa' \"bbb\"";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(
        lexer.tokens[0].data,
        TokenData::StringLiteral("aaa".to_string())
    );

    assert_eq!(
        lexer.tokens[1].data,
        TokenData::StringLiteral("bbb".to_string())
    );
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
        lexer.tokens[0].data,
        TokenData::Punctuator(Punctuator::OpenBlock)
    );
    assert_eq!(
        lexer.tokens[1].data,
        TokenData::Punctuator(Punctuator::OpenParen)
    );
    assert_eq!(
        lexer.tokens[2].data,
        TokenData::Punctuator(Punctuator::CloseParen)
    );
    assert_eq!(
        lexer.tokens[3].data,
        TokenData::Punctuator(Punctuator::OpenBracket)
    );
    assert_eq!(
        lexer.tokens[4].data,
        TokenData::Punctuator(Punctuator::CloseBracket)
    );
    assert_eq!(lexer.tokens[5].data, TokenData::Punctuator(Punctuator::Dot));
    assert_eq!(
        lexer.tokens[6].data,
        TokenData::Punctuator(Punctuator::Spread)
    );
    assert_eq!(
        lexer.tokens[7].data,
        TokenData::Punctuator(Punctuator::Semicolon)
    );
    assert_eq!(
        lexer.tokens[8].data,
        TokenData::Punctuator(Punctuator::Comma)
    );
    assert_eq!(
        lexer.tokens[9].data,
        TokenData::Punctuator(Punctuator::LessThan)
    );
    assert_eq!(
        lexer.tokens[10].data,
        TokenData::Punctuator(Punctuator::GreaterThan)
    );
    assert_eq!(
        lexer.tokens[11].data,
        TokenData::Punctuator(Punctuator::LessThanOrEq)
    );
    assert_eq!(
        lexer.tokens[12].data,
        TokenData::Punctuator(Punctuator::GreaterThanOrEq)
    );
    assert_eq!(lexer.tokens[13].data, TokenData::Punctuator(Punctuator::Eq));
    assert_eq!(
        lexer.tokens[14].data,
        TokenData::Punctuator(Punctuator::NotEq)
    );
    assert_eq!(
        lexer.tokens[15].data,
        TokenData::Punctuator(Punctuator::StrictEq)
    );
    assert_eq!(
        lexer.tokens[16].data,
        TokenData::Punctuator(Punctuator::StrictNotEq)
    );
    assert_eq!(
        lexer.tokens[17].data,
        TokenData::Punctuator(Punctuator::Add)
    );
    assert_eq!(
        lexer.tokens[18].data,
        TokenData::Punctuator(Punctuator::Sub)
    );
    assert_eq!(
        lexer.tokens[19].data,
        TokenData::Punctuator(Punctuator::Mul)
    );
    assert_eq!(
        lexer.tokens[20].data,
        TokenData::Punctuator(Punctuator::Mod)
    );
    assert_eq!(
        lexer.tokens[21].data,
        TokenData::Punctuator(Punctuator::Dec)
    );
    assert_eq!(
        lexer.tokens[22].data,
        TokenData::Punctuator(Punctuator::LeftSh)
    );
    assert_eq!(
        lexer.tokens[23].data,
        TokenData::Punctuator(Punctuator::RightSh)
    );
    assert_eq!(
        lexer.tokens[24].data,
        TokenData::Punctuator(Punctuator::URightSh)
    );
    assert_eq!(
        lexer.tokens[25].data,
        TokenData::Punctuator(Punctuator::And)
    );
    assert_eq!(lexer.tokens[26].data, TokenData::Punctuator(Punctuator::Or));
    assert_eq!(
        lexer.tokens[27].data,
        TokenData::Punctuator(Punctuator::Xor)
    );
    assert_eq!(
        lexer.tokens[28].data,
        TokenData::Punctuator(Punctuator::Not)
    );
    assert_eq!(
        lexer.tokens[29].data,
        TokenData::Punctuator(Punctuator::Neg)
    );
    assert_eq!(
        lexer.tokens[30].data,
        TokenData::Punctuator(Punctuator::BoolAnd)
    );
    assert_eq!(
        lexer.tokens[31].data,
        TokenData::Punctuator(Punctuator::BoolOr)
    );
    assert_eq!(
        lexer.tokens[32].data,
        TokenData::Punctuator(Punctuator::Question)
    );
    assert_eq!(
        lexer.tokens[33].data,
        TokenData::Punctuator(Punctuator::Colon)
    );
    assert_eq!(
        lexer.tokens[34].data,
        TokenData::Punctuator(Punctuator::Assign)
    );
    assert_eq!(
        lexer.tokens[35].data,
        TokenData::Punctuator(Punctuator::AssignAdd)
    );
    assert_eq!(
        lexer.tokens[36].data,
        TokenData::Punctuator(Punctuator::AssignSub)
    );
    assert_eq!(
        lexer.tokens[37].data,
        TokenData::Punctuator(Punctuator::AssignMul)
    );
    assert_eq!(
        lexer.tokens[38].data,
        TokenData::Punctuator(Punctuator::AssignAnd)
    );
    assert_eq!(
        lexer.tokens[39].data,
        TokenData::Punctuator(Punctuator::AssignPow)
    );
    assert_eq!(
        lexer.tokens[40].data,
        TokenData::Punctuator(Punctuator::Inc)
    );
    assert_eq!(
        lexer.tokens[41].data,
        TokenData::Punctuator(Punctuator::Pow)
    );
    assert_eq!(
        lexer.tokens[42].data,
        TokenData::Punctuator(Punctuator::AssignLeftSh)
    );
    assert_eq!(
        lexer.tokens[43].data,
        TokenData::Punctuator(Punctuator::AssignRightSh)
    );
    assert_eq!(
        lexer.tokens[44].data,
        TokenData::Punctuator(Punctuator::AssignURightSh)
    );
    assert_eq!(
        lexer.tokens[45].data,
        TokenData::Punctuator(Punctuator::AssignAnd)
    );
    assert_eq!(
        lexer.tokens[46].data,
        TokenData::Punctuator(Punctuator::AssignOr)
    );
    assert_eq!(
        lexer.tokens[47].data,
        TokenData::Punctuator(Punctuator::AssignXor)
    );
    assert_eq!(
        lexer.tokens[48].data,
        TokenData::Punctuator(Punctuator::Arrow)
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
    assert_eq!(lexer.tokens[0].data, TokenData::Keyword(Keyword::Await));
    assert_eq!(lexer.tokens[1].data, TokenData::Keyword(Keyword::Break));
    assert_eq!(lexer.tokens[2].data, TokenData::Keyword(Keyword::Case));
    assert_eq!(lexer.tokens[3].data, TokenData::Keyword(Keyword::Catch));
    assert_eq!(lexer.tokens[4].data, TokenData::Keyword(Keyword::Class));
    assert_eq!(lexer.tokens[5].data, TokenData::Keyword(Keyword::Const));
    assert_eq!(lexer.tokens[6].data, TokenData::Keyword(Keyword::Continue));
    assert_eq!(lexer.tokens[7].data, TokenData::Keyword(Keyword::Debugger));
    assert_eq!(lexer.tokens[8].data, TokenData::Keyword(Keyword::Default));
    assert_eq!(lexer.tokens[9].data, TokenData::Keyword(Keyword::Delete));
    assert_eq!(lexer.tokens[10].data, TokenData::Keyword(Keyword::Do));
    assert_eq!(lexer.tokens[11].data, TokenData::Keyword(Keyword::Else));
    assert_eq!(lexer.tokens[12].data, TokenData::Keyword(Keyword::Export));
    assert_eq!(lexer.tokens[13].data, TokenData::Keyword(Keyword::Extends));
    assert_eq!(lexer.tokens[14].data, TokenData::Keyword(Keyword::Finally));
    assert_eq!(lexer.tokens[15].data, TokenData::Keyword(Keyword::For));
    assert_eq!(lexer.tokens[16].data, TokenData::Keyword(Keyword::Function));
    assert_eq!(lexer.tokens[17].data, TokenData::Keyword(Keyword::If));
    assert_eq!(lexer.tokens[18].data, TokenData::Keyword(Keyword::Import));
    assert_eq!(lexer.tokens[19].data, TokenData::Keyword(Keyword::In));
    assert_eq!(
        lexer.tokens[20].data,
        TokenData::Keyword(Keyword::InstanceOf)
    );
    assert_eq!(lexer.tokens[21].data, TokenData::Keyword(Keyword::New));
    assert_eq!(lexer.tokens[22].data, TokenData::Keyword(Keyword::Return));
    assert_eq!(lexer.tokens[23].data, TokenData::Keyword(Keyword::Super));
    assert_eq!(lexer.tokens[24].data, TokenData::Keyword(Keyword::Switch));
    assert_eq!(lexer.tokens[25].data, TokenData::Keyword(Keyword::This));
    assert_eq!(lexer.tokens[26].data, TokenData::Keyword(Keyword::Throw));
    assert_eq!(lexer.tokens[27].data, TokenData::Keyword(Keyword::Try));
    assert_eq!(lexer.tokens[28].data, TokenData::Keyword(Keyword::TypeOf));
    assert_eq!(lexer.tokens[29].data, TokenData::Keyword(Keyword::Var));
    assert_eq!(lexer.tokens[30].data, TokenData::Keyword(Keyword::Void));
    assert_eq!(lexer.tokens[31].data, TokenData::Keyword(Keyword::While));
    assert_eq!(lexer.tokens[32].data, TokenData::Keyword(Keyword::With));
    assert_eq!(lexer.tokens[33].data, TokenData::Keyword(Keyword::Yield));
}

#[test]
fn check_variable_definition_tokens() {
    let s = "let a = 'hello';";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::Keyword(Keyword::Let));
    assert_eq!(lexer.tokens[1].data, TokenData::Identifier("a".to_string()));
    assert_eq!(
        lexer.tokens[2].data,
        TokenData::Punctuator(Punctuator::Assign)
    );
    assert_eq!(
        lexer.tokens[3].data,
        TokenData::StringLiteral("hello".to_string())
    );
}

#[test]
fn check_positions() {
    let s = "console.log(\"hello world\"); // Test";
    // ------123456789
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    // The first column is 1 (not zero indexed)
    assert_eq!(lexer.tokens[0].pos.column_number, 1);
    assert_eq!(lexer.tokens[0].pos.line_number, 1);
    // Dot Token starts on column 8
    assert_eq!(lexer.tokens[1].pos.column_number, 8);
    assert_eq!(lexer.tokens[1].pos.line_number, 1);
    // Log Token starts on column 9
    assert_eq!(lexer.tokens[2].pos.column_number, 9);
    assert_eq!(lexer.tokens[2].pos.line_number, 1);
    // Open parenthesis token starts on column 12
    assert_eq!(lexer.tokens[3].pos.column_number, 12);
    assert_eq!(lexer.tokens[3].pos.line_number, 1);
    // String token starts on column 13
    assert_eq!(lexer.tokens[4].pos.column_number, 13);
    assert_eq!(lexer.tokens[4].pos.line_number, 1);
    // Close parenthesis token starts on column 26
    assert_eq!(lexer.tokens[5].pos.column_number, 26);
    assert_eq!(lexer.tokens[5].pos.line_number, 1);
    // Semi Colon token starts on column 27
    assert_eq!(lexer.tokens[6].pos.column_number, 27);
    assert_eq!(lexer.tokens[6].pos.line_number, 1);
    // Comment start on column 29
    // Semi Colon token starts on column 27
    assert_eq!(lexer.tokens[7].pos.column_number, 29);
    assert_eq!(lexer.tokens[7].pos.line_number, 1);
}

#[test]
fn check_line_numbers() {
    let s = "// Copyright (C) 2017 Ecma International.  All rights reserved.\n\
                 // This code is governed by the BSD license found in the LICENSE file.\n\
                 /*---\n\
                 description: |\n    \
                     Collection of assertion functions used throughout test262\n\
                 defines: [assert]\n\
                 ---*/\n\n\n\
                 function assert(mustBeTrue, message) {";

    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    // The first column is 1 (not zero indexed), first line is also 1
    assert_eq!(lexer.tokens[0].pos.column_number, 1);
    assert_eq!(lexer.tokens[0].pos.line_number, 1);
    // Second comment starts on line 2
    assert_eq!(lexer.tokens[1].pos.column_number, 1);
    assert_eq!(lexer.tokens[1].pos.line_number, 2);
    // Multiline comment starts on line 3
    assert_eq!(lexer.tokens[2].pos.column_number, 1);
    assert_eq!(lexer.tokens[2].pos.line_number, 3);
    // Function Token is on line 10
    assert_eq!(lexer.tokens[3].pos.column_number, 1);
    assert_eq!(lexer.tokens[3].pos.line_number, 10);
}

// Increment/Decrement
#[test]
fn check_decrement_advances_lexer_2_places() {
    // Here we want an example of decrementing an integer
    let s = "let a = b--;";
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[4].data, TokenData::Punctuator(Punctuator::Dec));
    // Decrementing means adding 2 characters '--', the lexer should consume it as a single token
    // and move the curser forward by 2, meaning the next token should be a semicolon
    assert_eq!(
        lexer.tokens[5].data,
        TokenData::Punctuator(Punctuator::Semicolon)
    );
}

#[test]
fn numbers() {
    let mut lexer =
        Lexer::new("1 2 0x34 056 7.89 42. 5e3 5e+3 5e-3 0b10 0O123 0999 1.0e1 1.0e-1 1.0E1 1E1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
    assert_eq!(lexer.tokens[1].data, TokenData::NumericLiteral(2.0));
    assert_eq!(lexer.tokens[2].data, TokenData::NumericLiteral(52.0));
    assert_eq!(lexer.tokens[3].data, TokenData::NumericLiteral(46.0));
    assert_eq!(lexer.tokens[4].data, TokenData::NumericLiteral(7.89));
    assert_eq!(lexer.tokens[5].data, TokenData::NumericLiteral(42.0));
    assert_eq!(lexer.tokens[6].data, TokenData::NumericLiteral(5000.0));
    assert_eq!(lexer.tokens[7].data, TokenData::NumericLiteral(5000.0));
    assert_eq!(lexer.tokens[8].data, TokenData::NumericLiteral(0.005));
    assert_eq!(lexer.tokens[9].data, TokenData::NumericLiteral(2.0));
    assert_eq!(lexer.tokens[10].data, TokenData::NumericLiteral(83.0));
    assert_eq!(lexer.tokens[11].data, TokenData::NumericLiteral(999.0));
    assert_eq!(lexer.tokens[12].data, TokenData::NumericLiteral(10.0));
    assert_eq!(lexer.tokens[13].data, TokenData::NumericLiteral(0.1));
    assert_eq!(lexer.tokens[14].data, TokenData::NumericLiteral(10.0));
    assert_eq!(lexer.tokens[14].data, TokenData::NumericLiteral(10.0));
}

#[test]
fn test_single_number_without_semicolon() {
    let mut lexer = Lexer::new("1");
    lexer.lex().expect("failed to lex");
}

#[test]
fn test_number_followed_by_dot() {
    let mut lexer = Lexer::new("1..");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
    assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Dot));
}

#[test]
fn test_regex_literal() {
    let mut lexer = Lexer::new("/(?:)/");
    lexer.lex().expect("failed to lex");
    assert_eq!(
        lexer.tokens[0].data,
        TokenData::RegularExpressionLiteral("(?:)".to_string(), "".to_string())
    );
}

#[test]
fn test_regex_literal_flags() {
    let mut lexer = Lexer::new(r"/\/[^\/]*\/*/gmi");
    lexer.lex().expect("failed to lex");
    assert_eq!(
        lexer.tokens[0].data,
        TokenData::RegularExpressionLiteral("\\/[^\\/]*\\/*".to_string(), "gmi".to_string())
    );
}

#[test]
fn test_addition_no_spaces() {
    let mut lexer = Lexer::new("1+1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
    assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].data, TokenData::NumericLiteral(1.0));
}

#[test]
fn test_addition_no_spaces_left_side() {
    let mut lexer = Lexer::new("1+ 1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
    assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].data, TokenData::NumericLiteral(1.0));
}

#[test]
fn test_addition_no_spaces_right_side() {
    let mut lexer = Lexer::new("1 +1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
    assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].data, TokenData::NumericLiteral(1.0));
}

#[test]
fn test_addition_no_spaces_e_number_left_side() {
    let mut lexer = Lexer::new("1e2+ 1");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(100.0));
    assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].data, TokenData::NumericLiteral(1.0));
}

#[test]
fn test_addition_no_spaces_e_number_right_side() {
    let mut lexer = Lexer::new("1 +1e3");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1.0));
    assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Add));
    assert_eq!(lexer.tokens[2].data, TokenData::NumericLiteral(1000.0));
}

#[test]
fn test_addition_no_spaces_e_number() {
    let mut lexer = Lexer::new("1e3+1e11");
    lexer.lex().expect("failed to lex");
    assert_eq!(lexer.tokens[0].data, TokenData::NumericLiteral(1000.0));
    assert_eq!(lexer.tokens[1].data, TokenData::Punctuator(Punctuator::Add));
    assert_eq!(
        lexer.tokens[2].data,
        TokenData::NumericLiteral(100_000_000_000.0)
    );
}
