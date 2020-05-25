//! Cursor implementation for the parser.

use super::ParseError;
use crate::syntax::ast::{
    token::{Token, TokenKind},
    Punctuator,
};

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug, Clone, Default)]
pub(super) struct Cursor<'a> {
    /// The tokens being input.
    tokens: &'a [Token],
    /// The current position within the tokens.
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor.
    pub(super) fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            ..Self::default()
        }
    }

    /// Retrieves the current position of the cursor in the token stream.
    pub(super) fn pos(&self) -> usize {
        self.pos
    }

    /// Moves the cursor to the given position.
    ///
    /// This is intended to be used *always* with `Cursor::pos()`.
    pub(super) fn seek(&mut self, pos: usize) {
        self.pos = pos
    }

    /// Moves the cursor to the next token and returns the token.
    pub(super) fn next(&mut self) -> Option<&'a Token> {
        loop {
            let token = self.tokens.get(self.pos);
            if let Some(tk) = token {
                self.pos += 1;

                if tk.kind != TokenKind::LineTerminator {
                    break Some(tk);
                }
            } else {
                break None;
            }
        }
    }

    /// Peeks the next token without moving the cursor.
    pub(super) fn peek(&self, skip: usize) -> Option<&'a Token> {
        let mut count = 0;
        let mut skipped = 0;
        loop {
            let token = self.tokens.get(self.pos + count);
            count += 1;

            if let Some(tk) = token {
                if tk.kind != TokenKind::LineTerminator {
                    if skipped == skip {
                        break Some(tk);
                    }

                    skipped += 1;
                }
            } else {
                break None;
            }
        }
    }

    /// Moves the cursor to the previous token and returns the token.
    pub(super) fn back(&mut self) {
        debug_assert!(
            self.pos > 0,
            "cannot go back in a cursor that is at the beginning of the list of tokens"
        );

        self.pos -= 1;
        while self
            .tokens
            .get(self.pos - 1)
            .expect("token disappeared")
            .kind
            == TokenKind::LineTerminator
            && self.pos > 0
        {
            self.pos -= 1;
        }
    }

    /// Peeks the previous token without moving the cursor.
    pub(super) fn peek_prev(&self) -> Option<&'a Token> {
        if self.pos == 0 {
            None
        } else {
            let mut back = 1;
            let mut tok = self.tokens.get(self.pos - back).expect("token disappeared");
            while self.pos >= back && tok.kind == TokenKind::LineTerminator {
                back += 1;
                tok = self.tokens.get(self.pos - back).expect("token disappeared");
            }

            if back == self.pos {
                None
            } else {
                Some(tok)
            }
        }
    }

    /// Returns an error if the next token is not of kind `kind`.
    ///
    /// Note: it will consume the next token.
    pub(super) fn expect<K>(&mut self, kind: K, context: &'static str) -> Result<(), ParseError>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.next().ok_or(ParseError::AbruptEnd)?;
        let kind = kind.into();

        if next_token.kind == kind {
            Ok(())
        } else {
            Err(ParseError::expected(
                vec![kind],
                next_token.clone(),
                context,
            ))
        }
    }

    /// It will peek for the next token, to see if it's a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn peek_semicolon(&self, do_while: bool) -> (bool, Option<&Token>) {
        match self.tokens.get(self.pos) {
            Some(tk) => match tk.kind {
                TokenKind::Punctuator(Punctuator::Semicolon) => (true, Some(tk)),
                TokenKind::LineTerminator | TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    (true, Some(tk))
                }
                _ => {
                    if do_while {
                        debug_assert!(
                            self.pos != 0,
                            "cannot be finishing a do-while if we are at the beginning"
                        );

                        let tok = self
                            .tokens
                            .get(self.pos - 1)
                            .expect("could not find previous token");
                        if tok.kind == TokenKind::Punctuator(Punctuator::CloseParen) {
                            return (true, Some(tk));
                        }
                    }

                    (false, Some(tk))
                }
            },
            None => (true, None),
        }
    }

    /// It will check if the next token is a semicolon.
    ///
    /// It will automatically insert a semicolon if needed, as specified in the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
    pub(super) fn expect_semicolon(
        &mut self,
        do_while: bool,
        context: &'static str,
    ) -> Result<(), ParseError> {
        match self.peek_semicolon(do_while) {
            (true, Some(tk)) => match tk.kind {
                TokenKind::Punctuator(Punctuator::Semicolon) | TokenKind::LineTerminator => {
                    self.pos += 1;
                    Ok(())
                }
                _ => Ok(()),
            },
            (true, None) => Ok(()),
            (false, Some(tk)) => Err(ParseError::expected(
                vec![TokenKind::Punctuator(Punctuator::Semicolon)],
                tk.clone(),
                context,
            )),
            (false, None) => unreachable!(),
        }
    }

    /// It will make sure that the next token is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    pub(super) fn peek_expect_no_lineterminator(&mut self, skip: usize) -> Result<(), ParseError> {
        let mut count = 0;
        let mut skipped = 0;
        loop {
            let token = self.tokens.get(self.pos + count);
            count += 1;
            if let Some(tk) = token {
                if skipped == skip && tk.kind == TokenKind::LineTerminator {
                    break Err(ParseError::unexpected(tk.clone(), None));
                } else if skipped == skip && tk.kind != TokenKind::LineTerminator {
                    break Ok(());
                } else if tk.kind != TokenKind::LineTerminator {
                    skipped += 1;
                }
            } else {
                break Err(ParseError::AbruptEnd);
            }
        }
    }

    /// Advance the cursor to the next token and retrieve it, only if it's of `kind` type.
    ///
    /// When the next token is a `kind` token, get the token, otherwise return `None`. This
    /// function skips line terminators.
    pub(super) fn next_if<K>(&mut self, kind: K) -> Option<&'a Token>
    where
        K: Into<TokenKind>,
    {
        let next_token = self.peek(0)?;

        if next_token.kind == kind.into() {
            self.next()
        } else {
            None
        }
    }
}
