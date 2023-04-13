//! Boa's lexing for ECMAScript comments.

use crate::lexer::{Cursor, Error, Token, TokenKind, Tokenizer};
use boa_ast::{Position, Span};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

use super::is_whitespace;

/// Lexes a single line comment.
///
/// Assumes that the initial '//' is already consumed.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-comments
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar
pub(super) struct SingleLineComment;

impl<R> Tokenizer<R> for SingleLineComment {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        _interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("SingleLineComment", "Lexing");

        // Skip either to the end of the line or to the end of the input
        while let Some(ch) = cursor.peek_char()? {
            let tried_ch = char::try_from(ch);
            match tried_ch {
                Ok(c) if c == '\r' || c == '\n' || c == '\u{2028}' || c == '\u{2029}' => break,
                _ => {}
            };
            cursor.next_char().expect("Comment character vanished");
        }
        Ok(Token::new(
            TokenKind::Comment,
            Span::new(start_pos, cursor.pos()),
        ))
    }
}

/// Lexes a delimited (`/* */`) comment.
///
/// Assumes that the initial '/*' is already consumed.
///
/// If `multiline` is `true`, this is equivalent to the [`MultiLineComment`][mlc] token
/// from the spec. Otherwise, it is equivalent to the [`SingleLineDelimitedComment`][sldc] token,
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [mlc]: https://tc39.es/ecma262/#prod-MultiLineComment
/// [sldc]: https://tc39.es/ecma262/#prod-annexB-SingleLineDelimitedComment
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar
pub(super) struct DelimitedComment;

impl<R> Tokenizer<R> for DelimitedComment {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("MultiLineComment", "Lexing");

        let mut new_line = false;
        while let Some(ch) = cursor.next_char()? {
            let tried_ch = char::try_from(ch);
            match tried_ch {
                Ok(c) if c == '*' && cursor.next_is(b'/')? => {
                    return Ok(Token::new(
                        if new_line {
                            HTMLCloseComment.lex(cursor, start_pos, interner)?;
                            TokenKind::LineTerminator
                        } else {
                            TokenKind::Comment
                        },
                        Span::new(start_pos, cursor.pos()),
                    ))
                }
                Ok(c) if c == '\r' || c == '\n' || c == '\u{2028}' || c == '\u{2029}' => {
                    new_line = true;
                }
                _ => {}
            };
        }

        Err(Error::syntax(
            "unterminated multiline comment",
            cursor.pos(),
        ))
    }
}

/// Lexes a first line Hashbang comment
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar

pub(super) struct HashbangComment;

impl<R> Tokenizer<R> for HashbangComment {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        _interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("Hashbang", "Lexing");

        while let Some(ch) = cursor.next_char()? {
            let tried_ch = char::try_from(ch);
            match tried_ch {
                Ok(c) if c == '\r' || c == '\n' || c == '\u{2028}' || c == '\u{2029}' => break,
                _ => {}
            };
        }

        Ok(Token::new(
            TokenKind::Comment,
            Span::new(start_pos, cursor.pos()),
        ))
    }
}

/// Lexes a single line HTML close comment
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-html-like-comments
pub(super) struct HTMLCloseComment;

impl<R> Tokenizer<R> for HTMLCloseComment {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("HTML close", "Lexing");

        while cursor.peek_char()?.map_or(false, is_whitespace) {
            let _next = cursor.next_char();
        }

        while cursor.peek_n(2)? == [b'/', b'*'] {
            let _next = cursor.next_byte();
            let _next = cursor.next_byte();

            let start = cursor.pos();
            let comment = DelimitedComment.lex(cursor, start, interner)?;
            if comment.kind() == &TokenKind::LineTerminator {
                return Ok(comment)
            }

            while cursor.peek_char()?.map_or(false, is_whitespace) {
                let _next = cursor.next_char();
            }
        }

        if cursor.peek_n(3)? != [b'-', b'-', b'>'] {
            return Ok(Token::new(
                TokenKind::Comment,
                Span::new(start_pos, cursor.pos()),
            ));
        }

        let _next = cursor.next_byte();
        let _next = cursor.next_byte();
        let _next = cursor.next_byte();

        let start = cursor.pos();
        SingleLineComment.lex(cursor, start, interner)?;

        Ok(Token::new(
            TokenKind::Comment,
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
