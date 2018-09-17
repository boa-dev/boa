extern crate js;
use js::syntax::ast::keyword::Keyword;
use js::syntax::ast::punc::Punctuator;
use js::syntax::ast::token::TokenData;
use js::syntax::lexer::Lexer;

#[test]
/// Check basic variable definition tokens
fn check_variable_definition_tokens() {
    let s = &String::from("let a = 'hello';");
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("finished");
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
/// Check positions are correct
fn check_positions() {
    let s = &String::from("console.log(\"hello world\");");
    // -------------------123456789
    let mut lexer = Lexer::new(s);
    lexer.lex().expect("finished");
    // The first column is 1 (not zero indexed)
    assert_eq!(lexer.tokens[0].pos.column_number, 1);
    assert_eq!(lexer.tokens[0].pos.line_number, 1);
    // Dot Token starts on line 7
    assert_eq!(lexer.tokens[1].pos.column_number, 8);
    assert_eq!(lexer.tokens[1].pos.line_number, 1);
    // Log Token starts on line 7
    assert_eq!(lexer.tokens[2].pos.column_number, 9);
    assert_eq!(lexer.tokens[2].pos.line_number, 1);
    // Open parenthesis token starts on line 12
    assert_eq!(lexer.tokens[3].pos.column_number, 12);
    assert_eq!(lexer.tokens[3].pos.line_number, 1);
    // String token starts on line 13
    assert_eq!(lexer.tokens[4].pos.column_number, 13);
    assert_eq!(lexer.tokens[4].pos.line_number, 1);
    // Close parenthesis token starts on line 26
    assert_eq!(lexer.tokens[5].pos.column_number, 26);
    assert_eq!(lexer.tokens[5].pos.line_number, 1);
    // Semi Colon token starts on line 27
    assert_eq!(lexer.tokens[6].pos.column_number, 27);
    assert_eq!(lexer.tokens[6].pos.line_number, 1);
}
