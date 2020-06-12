use super::{Cursor, Error, RegexLiteral, Tokenizer};
use crate::syntax::ast::{Position, Span};
use crate::syntax::lexer::{Token, TokenKind};
use std::io::{ErrorKind, Read};

macro_rules! comment_match {
    () => {
        '/'
    };
}

/// Skips comments.
///
/// Assumes that the '/' char is already consumed.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]:
/// [mdn]:
pub(super) struct Comment;

impl Comment {
    /// Creates a new comment lexer.
    pub(super) fn new() -> Self {
        Self {}
    }
}

impl<R> Tokenizer<R> for Comment {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        match cursor.peek() {
            None => Err(Error::syntax("Expecting Token /,*,= or regex")),
            Some(Err(_)) => Err(Error::from(std::io::Error::new(
                ErrorKind::Interrupted,
                "Failed to peek next character",
            ))),
            Some(Ok(ch)) => {
                match ch {
                    '/' => {
                        // Skip either to the end of the line or to the end of the input
                        while let Some(ch) = cursor.next() {
                            match ch {
                                Err(e) => {
                                    return Err(Error::IO(e));
                                }
                                Ok('\n') => {
                                    break;
                                }
                                _ => {}
                            }
                        }
                        cursor.next_line();
                        Ok(Token::new(
                            TokenKind::Comment,
                            Span::new(start_pos, cursor.pos()),
                        ))
                    }
                    // block comment
                    '*' => {
                        loop {
                            if let Some(ch) = cursor.next() {
                                match ch {
                                    Err(e) => {
                                        return Err(Error::IO(e));
                                    }
                                    Ok('\n') => {
                                        cursor.next_line();
                                    }
                                    Ok('*') => {
                                        if cursor.next_is('/')? {
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                return Err(Error::syntax("unterminated multiline comment"));
                            }
                        }
                        Ok(Token::new(
                            TokenKind::Comment,
                            Span::new(start_pos, cursor.pos()),
                        ))
                    }
                    _ => RegexLiteral::new().lex(cursor, start_pos),
                }
            }
        }
    }
}
