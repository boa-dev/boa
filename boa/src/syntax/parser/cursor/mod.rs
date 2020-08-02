//! Cursor implementation for the parser.
mod buffered_lexer;

use super::ParseError;
use crate::syntax::{
    ast::Punctuator,
    lexer::{InputElement, Lexer, Position, Token, TokenKind},
};
use buffered_lexer::BufferedLexer;
use std::io::Read;

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
    /// Creates a new cursor.
    #[inline(always)]
    pub(super) fn new(reader: R) -> Self {
        Self {
            buffered_lexer: Lexer::new(reader).into(),
        }
    }

    #[inline(always)]
    pub(super) fn set_goal(&mut self, elm: InputElement) {
        self.buffered_lexer.set_goal(elm)
    }

    #[inline(always)]
    pub(super) fn lex_regex(&mut self, start: Position) -> Result<Token, ParseError> {
        self.buffered_lexer.lex_regex(start)
    }

    #[inline(always)]
    pub(super) fn next(
        &mut self,
        skip_line_terminators: bool,
    ) -> Result<Option<Token>, ParseError> {
        self.buffered_lexer.next(skip_line_terminators)
    }

    #[inline(always)]
    pub(super) fn peek(
        &mut self,
        skip_n: usize,
        skip_line_terminators: bool,
    ) -> Result<Option<Token>, ParseError> {
        self.buffered_lexer.peek(skip_n, skip_line_terminators)
    }

    #[inline(always)]
    pub(super) fn push_back(&mut self, token: Token) {
        self.buffered_lexer.push_back(token)
    }

    /// Returns an error if the next token is not of kind `kind`.
    ///
    /// Note: it will consume the next token only if the next token is the expected type.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    #[inline(always)]
    pub(super) fn expect<K>(
        &mut self,
        kind: K,
        context: &'static str,
        skip_line_terminators: bool,
    ) -> Result<Token, ParseError>
    where
        K: Into<TokenKind>,
    {
        let next_token = self
            .peek(0, skip_line_terminators)?
            .ok_or(ParseError::AbruptEnd)?;
        let kind = kind.into();

        if next_token.kind() == &kind {
            self.next(skip_line_terminators)?.expect("Token vanished");
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
    #[inline(always)]
    pub(super) fn peek_semicolon(&mut self) -> Result<(bool, Option<Token>), ParseError> {
        match self.peek(0, false)? {
            Some(tk) => match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) => Ok((true, Some(tk))),
                TokenKind::LineTerminator | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    Ok((true, Some(tk)))
                }
                _ => Ok((false, Some(tk))),
            },
            None => Ok((true, None)),
        }
    }

    /// Consumes the next token iff it is a semicolon otherwise returns an expected ParseError.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    #[inline(always)]
    pub(super) fn expect_semicolon(
        &mut self,
        context: &'static str,
    ) -> Result<Option<Token>, ParseError> {
        match self.peek_semicolon()? {
            (true, Some(tk)) => match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) | TokenKind::LineTerminator => {
                    self.next(false)?.expect("Token vanished"); // Consume the token.
                    Ok(Some(tk))
                }
                _ => Ok(Some(tk)),
            },
            (true, None) => Ok(None),
            (false, Some(tk)) => Err(ParseError::expected(
                vec![TokenKind::Punctuator(Punctuator::Semicolon)],
                tk,
                context,
            )),
            (false, None) => unreachable!(),
        }
    }

    /// It will make sure that the peeked token (skipping n tokens) is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    ///
    /// This is just syntatic sugar for a .peek(skip_n, false) call followed by a check that the result is not a line terminator or None.
    #[inline(always)]
    pub(super) fn peek_expect_no_lineterminator(
        &mut self,
        skip_n: usize,
    ) -> Result<(), ParseError> {
        if let Some(t) = self.peek(skip_n, false)? {
            if t.kind() == &TokenKind::LineTerminator {
                Err(ParseError::unexpected(t, None))
            } else {
                Ok(())
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
    #[inline(always)]
    pub(super) fn next_if<K>(
        &mut self,
        kind: K,
        skip_line_terminators: bool,
    ) -> Result<Option<Token>, ParseError>
    where
        K: Into<TokenKind>,
    {
        Ok(if let Some(token) = self.peek(0, skip_line_terminators)? {
            if token.kind() == &kind.into() {
                self.next(skip_line_terminators)?
            } else {
                None
            }
        } else {
            None
        })
    }
}
