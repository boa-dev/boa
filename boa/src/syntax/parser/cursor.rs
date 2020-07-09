//! Cursor implementation for the parser.

use super::ParseError;
use crate::syntax::{
    ast::Punctuator,
    lexer::{InputElement, Lexer, Position, Token, TokenKind},
};

use std::collections::VecDeque;
use std::io::Read;

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    lexer: Lexer<R>,
    peeked: VecDeque<Option<Token>>,
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new cursor.
    pub(super) fn new(reader: R) -> Self {
        Self {
            lexer: Lexer::new(reader),
            peeked: VecDeque::new(),
        }
    }

    /// Sets the goal symbol for the lexer.
    #[inline]
    pub(crate) fn set_goal(&mut self, elm: InputElement) {
        self.lexer.set_goal(elm)
    }

    /// Lexes the next tokens as a regex assuming that the starting '/' has already been consumed.
    pub(super) fn lex_regex(&mut self, start: Position) -> Result<Token, ParseError> {
        self.set_goal(InputElement::RegExp);
        self.lexer.lex_slash_token(start).map_err(|e| e.into())
    }

    /// Moves the cursor to the next token and returns the token.
    pub(super) fn next(&mut self) -> Result<Option<Token>, ParseError> {
        if let Some(t) = self.peeked.pop_front() {
            return Ok(t);
        }

        // No value has been peeked ahead already so need to go get the next value.
        Ok(self.lexer.next()?)
    }

    /// Peeks the next token without moving the cursor.
    pub(super) fn peek(&mut self) -> Result<Option<Token>, ParseError> {
        if let Some(v) = self.peeked.front() {
            return Ok(v.clone());
        }

        // No value has been peeked ahead already so need to go get the next value.
        let val = self.next()?;
        self.peeked.push_back(val.clone());
        Ok(val)
    }

    /// Peeks the token after the next token.
    /// i.e. if there are tokens A, B, C and peek() returns A then peek_skip(1) will return B.
    pub(super) fn peek_skip(&mut self) -> Result<Option<Token>, ParseError> {
        // Add elements to the peeked buffer upto the amount required to skip the given amount ahead.
        while self.peeked.len() < 2 {
            match self.lexer.next()? {
                Some(token) => self.peeked.push_back(Some(token.clone())),
                None => self.peeked.push_back(None),
            }
        }

        let temp = self
            .peeked
            .pop_front()
            .expect("Front peeked value has vanished");
        let ret = self
            .peeked
            .pop_front()
            .expect("Back peeked value has vanished");

        self.peeked.push_front(ret.clone());
        self.peeked.push_front(temp);

        Ok(ret)
    }

    /// Takes the given token and pushes it back onto the parser token queue (at the front so the token will be returned on next .peek()).
    pub(super) fn push_back(&mut self, token: Token) {
        self.peeked.push_front(Some(token));
    }

    /// Returns an error if the next token is not of kind `kind`.
    ///
    /// Note: it will consume the next token only if the next token is the expected type.
    pub(super) fn expect<K>(&mut self, kind: K, context: &'static str) -> Result<Token, ParseError>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.peek()?.ok_or(ParseError::AbruptEnd)?;
        let kind = kind.into();

        if next_token.kind() == &kind {
            self.next()?.expect("Token vanished");
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
    pub(super) fn peek_semicolon(&mut self) -> Result<(bool, Option<Token>), ParseError> {
        match self.peek()? {
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

    /// It will check if the next token is a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn expect_semicolon(
        &mut self,
        context: &'static str,
    ) -> Result<Option<Token>, ParseError> {
        match self.peek_semicolon()? {
            (true, Some(tk)) => match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) | TokenKind::LineTerminator => {
                    self.next()?.expect("Token vanished"); // Consume the token.
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

    /// It will make sure that the next token is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    ///
    /// If skip is true then the token after the peek() token is checked instead.
    pub(super) fn peek_expect_no_lineterminator(&mut self, skip: bool) -> Result<(), ParseError> {
        let token = if skip {
            self.peek_skip()?
        } else {
            self.peek()?
        };

        if let Some(t) = token {
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
    pub(super) fn next_if<K>(&mut self, kind: K) -> Result<Option<Token>, ParseError>
    where
        K: Into<TokenKind>,
    {
        Ok(if let Some(token) = self.peek()? {
            if token.kind() == &kind.into() {
                self.next()?
            } else {
                None
            }
        } else {
            None
        })
    }

    /// Advance the cursor to skip 0, 1 or more line terminators.
    #[inline]
    pub(super) fn skip_line_terminators(&mut self) -> Result<(), ParseError> {
        while self.next_if(TokenKind::LineTerminator)?.is_some() {}
        Ok(())
    }
}
