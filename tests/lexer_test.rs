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
