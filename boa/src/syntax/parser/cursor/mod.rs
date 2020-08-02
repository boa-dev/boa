//! Cursor implementation for the parser.

use super::ParseError;
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::Punctuator,
        lexer::{InputElement, Lexer, Position, Token, TokenKind},
    },
};
use std::{cmp::min, io::Read};

#[cfg(test)]
mod tests;

/// The fixed size of the buffer used for storing values that are peeked ahead.
/// Sized 5 to allow for peeking ahead upto 4 values and pushing back a single value.
const PEEK_BUF_SIZE: usize = 5;

/// The maximum number of tokens which can be peeked ahead.
const MAX_PEEK_SKIP: usize = 3;

/// Token cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    lexer: Lexer<R>,
    peeked: [Option<Token>; PEEK_BUF_SIZE],
    buf_size: usize,
    back_index: usize,
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new cursor.
    #[inline]
    pub(super) fn new(reader: R) -> Self {
        Self {
            lexer: Lexer::new(reader),
            peeked: [
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
            ],
            buf_size: 0,
            back_index: 0,
        }
    }

    /// Sets the goal symbol for the lexer.
    #[inline]
    pub(super) fn set_goal(&mut self, elm: InputElement) {
        let _timer = BoaProfiler::global().start_event("cursor::set_goal()", "Parsing");
        self.lexer.set_goal(elm)
    }

    /// Lexes the next tokens as a regex assuming that the starting '/' has already been consumed.
    #[inline]
    pub(super) fn lex_regex(&mut self, start: Position) -> Result<Token, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::lex_regex()", "Parsing");
        self.set_goal(InputElement::RegExp);
        self.lexer.lex_slash_token(start).map_err(|e| e.into())
    }

    /// Moves the cursor to the next token and returns the token.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    #[inline]
    pub(super) fn next(
        &mut self,
        skip_line_terminators: bool,
    ) -> Result<Option<Token>, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::next()", "Parsing");

        if self.buf_size == 0 {
            // No value has been peeked ahead already so need to go get the next value.
            Ok(self.lexer.next(skip_line_terminators)?)
        } else {
            // A value has already been peeked ahead so use that.
            let val = self.peeked[self.back_index].take();
            self.back_index = (self.back_index + 1) % PEEK_BUF_SIZE;
            self.buf_size -= 1;

            if skip_line_terminators {
                if let Some(t) = val {
                    if *t.kind() == TokenKind::LineTerminator {
                        self.next(skip_line_terminators)
                    } else {
                        Ok(Some(t))
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(val)
            }
        }
    }

    /// Peeks the nth token after the next token.
    /// n must be in the range [0, 3]
    /// i.e. if there are tokens A, B, C, D, E and peek(0, false) returns A then:
    ///     peek(1, false) == peek(1, true) == B.
    ///     peek(2, false) will return C.
    ///     peek(3, false) will return D.
    /// where A, B, C, D, E are tokens but not line terminators.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    /// i.e. If there are tokens A, B, \n, C and peek(0, false) is 'A' then the following will hold:
    ///         peek(0, true) == 'A'
    ///         peek(1, true) == 'B'
    ///         peek(1, false) == 'B'
    ///         peek(2, false) == \n
    ///         peek(2, true) == 'C'
    ///         peek(3, true) == None (End of stream)
    ///  Note:
    ///     peek(3, false) == 'C' iff peek(3, true) hasn't been called previously, this is because
    ///     with skip_line_terminators == true the '\n' would be discarded. This leads to the following statements
    ///     evaluating to true (in isolation from each other or any other previous cursor calls):
    ///         peek(3, false) == peek(3, false) == '\n'
    ///         peek(3, true) == peek(3, true) == None
    ///         peek(3, true) == peek(3, false) == None
    ///         (peek(3, false) == 'C') != (peek(3, true) == None)
    ///
    /// This behaviour is demonstrated directly in the cursor::tests::skip_peeked_terminators test.
    ///
    pub(super) fn peek(
        &mut self,
        skip_n: usize,
        skip_line_terminators: bool,
    ) -> Result<Option<Token>, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::peek()", "Parsing");
        if skip_n > MAX_PEEK_SKIP {
            unimplemented!("peek(n) where n > {}", MAX_PEEK_SKIP);
        }

        if skip_line_terminators {
            // We must go through the peeked buffer to remove any line terminators.
            // This only needs to be done upto the point at which we are peeking - it is
            // important that we don't go further than this as we would risk removing line terminators
            // which are later needed.
            for i in 0..min(skip_n + 1, self.buf_size) {
                let index = (self.back_index + i) % PEEK_BUF_SIZE;
                if let Some(t) = self.peeked[index].clone() {
                    if t.kind() == &TokenKind::LineTerminator {
                        self.peeked[index].take(); // Remove the line terminator

                        let mut dst_index = index; // Dst index for the swap.

                        // Move all subsequent values up (asif the line terminator never existed).
                        for j in (i + 1)..self.buf_size {
                            let src_index = (self.back_index + j) % PEEK_BUF_SIZE; // Src index for the swap.
                            self.peeked[dst_index] = self.peeked[src_index].take();
                            dst_index = src_index;
                        }

                        self.buf_size -= 1;
                    }
                }
            }
        }

        while self.buf_size <= skip_n {
            // Need to keep peeking more values.
            self.peeked[(self.back_index + self.buf_size) % PEEK_BUF_SIZE] =
                self.lexer.next(skip_line_terminators)?;

            self.buf_size += 1;
        }

        // Have now peeked ahead the right number of spaces so can fetch the value directly.
        Ok(self.peeked[(self.back_index + skip_n) % PEEK_BUF_SIZE].clone())
    }

    /// Returns an error if the next token is not of kind `kind`.
    ///
    /// Note: it will consume the next token only if the next token is the expected type.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
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
