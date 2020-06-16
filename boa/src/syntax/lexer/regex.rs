use super::{Cursor, Error, Tokenizer, Span};
use crate::syntax::ast::Position;
use crate::syntax::lexer::Token;
use std::io::{self, ErrorKind, Read};
use crate::syntax::lexer::TokenKind;

/// Regex literal lexing.
///
/// Lexes Division, Assigndiv or Regex literal.
///
/// Expects: Initial '/' to already be consumed by cursor.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://www.ecma-international.org/ecma-262/#sec-literals-regular-expression-literals
/// [mdn]:
#[derive(Debug, Clone, Copy)]
pub(super) struct RegexLiteral;

impl<R> Tokenizer<R> for RegexLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let mut body = String::new();

        // Lex RegularExpressionBody.
        loop {
            match cursor.next() {
                None => {
                    // Abrupt end.
                    return Err(Error::syntax("Abrupt end, regex not terminated"));
                }
                Some(Err(e)) => {
                    return Err(Error::from(e));
                }
                Some(Ok(c)) => {
                    match c {
                        '/' => break, // RegularExpressionBody finished.
                        '\n' | '\r' | '\u{2028}' | '\u{2029}' => {
                            // Not allowed in Regex literal.
                            return Err(Error::syntax("Encountered new line during regex"));
                        }
                        '\\' => {
                            // Escape sequence
                            body.push('\\');
                            match cursor.next() {
                                None => {
                                    // Abrupt end of regex.
                                    return Err(Error::syntax("Abrupt end, regex not terminated"));
                                }
                                Some(Err(_)) => {
                                    return Err(Error::from(io::Error::new(
                                        ErrorKind::Interrupted,
                                        "Failed to peek next character",
                                    )))
                                }
                                Some(Ok(sc)) => {
                                    match sc {
                                        '\n' | '\r' | '\u{2028}' | '\u{2029}' => {
                                            // Not allowed in Regex literal.
                                            return Err(Error::syntax(
                                                "Encountered new line during regex",
                                            ));
                                        }
                                        ch => body.push(ch),
                                    }
                                }
                            }
                        }
                        _ => body.push(c),
                    }
                }
            }
        }

        // body was parsed, now look for flags
        let mut flags = String::new();
        cursor.take_until_pred(&mut flags, &char::is_alphabetic);

        Ok(Token::new(
            TokenKind::regular_expression_literal(body, flags.parse()?),
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
