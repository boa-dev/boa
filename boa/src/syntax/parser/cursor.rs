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
    pub(crate) fn set_goal(&mut self, elm: InputElement) {
        self.lexer.set_goal(elm)
    }

    /// Lexes the next tokens as a regex assuming that the starting '/' has already been consumed.
    pub(super) fn lex_regex(&mut self, start: Position) -> Result<Token, ParseError> {
        self.set_goal(InputElement::RegExp);
        self.lexer
            .lex_slash_token(start)
            .map_err(|e| ParseError::lex(e))
    }

    /// Moves the cursor to the next token and returns the token.
    pub(super) fn next(&mut self) -> Option<Result<Token, ParseError>> {
        if let Some(t) = self.peeked.pop_front() {
            return t.map(|v| Ok(v));
        }

        // No value has been peeked ahead already so need to go get the next value.
        if let Some(t) = self.lexer.next() {
            Some(t.map_err(|e| ParseError::lex(e)))
        } else {
            None
        }
    }

    /// Peeks the next token without moving the cursor.
    pub(super) fn peek(&mut self) -> Option<Result<Token, ParseError>> {
        match self.peeked.pop_front() {
            Some(None) => {
                self.peeked.push_front(None); // Push the value back onto the peeked stack.
                return None;
            }
            Some(Some(token)) => {
                self.peeked.push_front(Some(token.clone())); // Push the value back onto the peeked stack.
                return Some(Ok(token));
            }
            None => {} // No value has been peeked ahead already so need to go get the next value.
        }

        match self.next() {
            Some(Ok(token)) => {
                self.peeked.push_back(Some(token.clone()));
                Some(Ok(token))
            }
            Some(Err(e)) => Some(Err(e)),
            None => {
                self.peeked.push_back(None);
                None
            }
        }
    }

    pub(super) fn peek_more(&mut self, skip: usize) -> Option<Result<Token, ParseError>> {
        if skip != 1 {
            // I don't believe we ever need to skip more than a single token?
            unimplemented!("Attempting to peek ahead more than a single token");
        }

        // Add elements to the peeked buffer upto the amount required to skip the given amount ahead.
        while self.peeked.len() < skip + 1 {
            match self.lexer.next() {
                Some(Ok(token)) => self.peeked.push_back(Some(token.clone())),
                Some(Err(e)) => return Some(Err(ParseError::lex(e))),
                None => self.peeked.push_back(None),
            }
        }

        let temp = self.peeked.pop_front().unwrap();
        let ret = self.peeked.pop_front().unwrap();

        self.peeked.push_front(ret.clone());
        self.peeked.push_front(temp);

        ret.map(|token| Ok(token))
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
        let next_token = self.peek().ok_or(ParseError::AbruptEnd)??;
        let kind = kind.into();

        if next_token.kind() == &kind {
            self.next();
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
        match self.peek() {
            Some(Ok(tk)) => match tk.kind() {
                TokenKind::Punctuator(Punctuator::Semicolon) => Ok((true, Some(tk))),
                TokenKind::LineTerminator | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    Ok((true, Some(tk)))
                }
                _ => Ok((false, Some(tk))),
            },
            Some(Err(e)) => Err(e),
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
                    self.next(); // Consume the token.
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
    pub(super) fn peek_expect_no_lineterminator(&mut self, skip: usize) -> Result<(), ParseError> {
        let token = if skip == 0 {
            self.peek()
        } else {
            self.peek_more(skip)
        };

        match token {
            Some(Ok(t)) => {
                if t.kind() == &TokenKind::LineTerminator {
                    Err(ParseError::unexpected(t, None))
                } else {
                    Ok(())
                }
            }
            Some(Err(e)) => Err(e),
            None => Err(ParseError::AbruptEnd),
        }
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`.
    pub(super) fn next_if<K>(&mut self, kind: K) -> Option<Result<Token, ParseError>>
    where
        K: Into<TokenKind>,
    {
        match self.peek() {
            Some(Ok(token)) => {
                if token.kind() == &kind.into() {
                    self.next()
                } else {
                    None
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    /// Advance the cursor to skip 0, 1 or more line terminators.
    pub(super) fn skip_line_terminators(&mut self) {
        while self.next_if(TokenKind::LineTerminator).is_some() {}
    }
}
