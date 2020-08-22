use super::{Cursor, Error, Span, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::Position,
        lexer::{Token, TokenKind},
    },
};
use std::io::Read;

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
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions
#[derive(Debug, Clone, Copy)]
pub(super) struct RegexLiteral;

impl<R> Tokenizer<R> for RegexLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("RegexLiteral", "Lexing");

        let mut body = String::new();

        // Lex RegularExpressionBody.
        loop {
            match cursor.next_char()? {
                None => {
                    // Abrupt end.
                    return Err(Error::syntax("abrupt end on regular expression"));
                }
                Some(c) => {
                    match c {
                        '/' => break, // RegularExpressionBody finished.
                        '\n' | '\r' | '\u{2028}' | '\u{2029}' => {
                            // Not allowed in Regex literal.
                            return Err(Error::syntax(
                                "new lines are not allowed in regular expressions",
                            ));
                        }
                        '\\' => {
                            // Escape sequence
                            body.push('\\');
                            if let Some(sc) = cursor.next_char()? {
                                match sc {
                                    '\n' | '\r' | '\u{2028}' | '\u{2029}' => {
                                        // Not allowed in Regex literal.
                                        return Err(Error::syntax(
                                            "new lines are not allowed in regular expressions",
                                        ));
                                    }
                                    ch => body.push(ch),
                                }
                            } else {
                                // Abrupt end of regex.
                                return Err(Error::syntax("abrupt end on regular expression"));
                            }
                        }
                        _ => body.push(c),
                    }
                }
            }
        }

        // body was parsed, now look for flags
        let mut flags = String::new();
        cursor.take_while_pred(&mut flags, &char::is_alphabetic)?;

        Ok(Token::new(
            TokenKind::regular_expression_literal(body, flags.parse()?),
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
