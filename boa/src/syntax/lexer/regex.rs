use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::Position;
use crate::syntax::lexer::Token;
use std::io::{self, ErrorKind, Read};

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
    fn lex(&mut self, cursor: &mut Cursor<R>, _start_pos: Position) -> Result<Token, Error>
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

        unimplemented!(
            "Regex handling, requires ability to peek more than a single character ahead"
        );
        // if regex {
        //     // body was parsed, now look for flags
        //     let flags = self.take_char_while(char::is_alphabetic)?;
        //     self.move_columns(body.len() as u32 + 1 + flags.len() as u32);
        //     self.push_token(TokenKind::regular_expression_literal(
        //         body, flags.parse()?,
        //     ), start_pos);
        // } else {
        //     // failed to parse regex, restore original buffer position and
        //     // parse either div or assigndiv
        //     self.buffer = original_buffer;
        //     self.position = original_pos;
        //     if self.next_is('=') {
        //         self.push_token(TokenKind::Punctuator(
        //             Punctuator::AssignDiv,
        //         ), start_pos);
        //     } else {
        //         self.push_token(TokenKind::Punctuator(Punctuator::Div), start_pos);
        //     }
        // }
    }
}
