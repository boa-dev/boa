//! Cursor implementation for the parser.
mod buffered_lexer;

use super::ParseError;
use crate::syntax::{
    ast::Punctuator,
    lexer::{InputElement, Lexer, Position, Token, TokenKind},
};
use buffered_lexer::BufferedLexer;
use std::io::Read;

/// The result of a peek for a semicolon.
#[derive(Debug)]
pub(super) enum SemicolonResult<'s> {
    Found(Option<&'s Token>),
    NotFound(&'s Token),
}

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    buffered_lexer: BufferedLexer<R>,
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new cursor with the given reader.
    #[inline]
    pub(super) fn new(reader: R) -> Self {
        Self {
            buffered_lexer: Lexer::new(reader).into(),
        }
    }

    #[inline]
    pub(super) fn set_goal(&mut self, elm: InputElement) {
        self.buffered_lexer.set_goal(elm)
    }

    #[inline]
    pub(super) fn lex_regex(&mut self, start: Position) -> Result<Token, ParseError> {
        self.buffered_lexer.lex_regex(start)
    }

    #[inline]
    pub(super) fn next(&mut self) -> Result<Option<Token>, ParseError> {
        self.buffered_lexer.next(true)
    }

    #[inline]
    pub(super) fn peek(&mut self, skip_n: usize) -> Result<Option<&Token>, ParseError> {
        self.buffered_lexer.peek(skip_n, true)
    }

    /// Returns an error if the next token is not of kind `kind`.
    ///
    /// Note: it will consume the next token only if the next token is the expected type.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    #[inline]
    pub(super) fn expect<K>(&mut self, kind: K, context: &'static str) -> Result<Token, ParseError>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.next()?.ok_or(ParseError::AbruptEnd)?;
        let kind = kind.into();

        if next_token.kind() == &kind {
            Ok(next_token)
        } else {
            Err(ParseError::expected(vec![kind], next_token, context))
        }
    }

    /// It will peek for the next token, to see if it's a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    #[inline]
    pub(super) fn peek_semicolon(&mut self) -> Result<SemicolonResult<'_>, ParseError> {
        match self.buffered_lexer.peek(0, false)? {
            Some(tk) => match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon)
                | TokenKind::LineTerminator
                | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    Ok(SemicolonResult::Found(Some(tk)))
                }
                _ => Ok(SemicolonResult::NotFound(tk)),
            },
            None => Ok(SemicolonResult::Found(None)),
        }
    }

    /// Consumes the next token if it is a semicolon, or returns a `ParseError` if it's not.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    #[inline]
    pub(super) fn expect_semicolon(&mut self, context: &'static str) -> Result<(), ParseError> {
        match self.peek_semicolon()? {
            SemicolonResult::Found(Some(tk)) => match *tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) | TokenKind::LineTerminator => {
                    let _ = self.buffered_lexer.next(false)?;
                    Ok(())
                }
                _ => Ok(()),
            },
            SemicolonResult::Found(None) => Ok(()),
            SemicolonResult::NotFound(tk) => Err(ParseError::expected(
                vec![TokenKind::Punctuator(Punctuator::Semicolon)],
                tk.clone(),
                context,
            )),
        }
    }

    /// It will make sure that the peeked token (skipping n tokens) is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    ///
    /// This is just syntatic sugar for a .peek(skip_n, false) call followed by a check that the result is not a line terminator or None.
    #[inline]
    pub(super) fn peek_expect_no_lineterminator(
        &mut self,
        skip_n: usize,
    ) -> Result<&Token, ParseError> {
        if let Some(t) = self.buffered_lexer.peek(skip_n, false)? {
            if t.kind() == &TokenKind::LineTerminator {
                Err(ParseError::unexpected(t.clone(), None))
            } else {
                Ok(t)
            }
        } else {
            Err(ParseError::AbruptEnd)
        }
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`.
    ///
    /// No next token also returns None.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    #[inline]
    pub(super) fn next_if<K>(&mut self, kind: K) -> Result<Option<Token>, ParseError>
    where
        K: Into<TokenKind>,
    {
        Ok(if let Some(token) = self.peek(0)? {
            if token.kind() == &kind.into() {
                self.next()?
            } else {
                None
            }
        } else {
            None
        })
    }
}
