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

    /// Peeks the next token without moving the cursor.
    ///
    /// This has the same semantics / behaviour as peek_skip(0).
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    // #[deprecated = "Replaced with peek_skip(0)"]
    pub(super) fn peek(
        &mut self,
        skip_line_terminators: bool,
    ) -> Result<Option<Token>, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::peek()", "Parsing");
        self.peek_skip(0, skip_line_terminators)
        // if self.buf_size == 0 {
        //     // No value has been peeked ahead already so need to go get the next value.

        //     let next = self.lexer.next(skip_line_terminators)?;
        //     self.peeked[self.back_index] = next;
        //     self.buf_size += 1;
        // }

        // let val = self.peeked[self.back_index].clone();

        // if skip_line_terminators {
        //     if let Some(token) = val {
        //         if token.kind() == &TokenKind::LineTerminator {
        //             self.peeked[self.back_index].take();
        //             self.back_index = (self.back_index + 1) % PEEK_BUF_SIZE;
        //             self.peek(skip_line_terminators)
        //         } else {
        //             Ok(Some(token))
        //         }
        //     } else {
        //         Ok(None)
        //     }
        // } else {
        //     Ok(val)
        // }
    }

    /// Peeks the nth token after the next token.
    /// i.e. if there are tokens A, B, C, D, E and peek_skip(0) returns A then peek_skip(1) will return B.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    /// i.e. If there are tokens A, B, \n, C and peek_skip(0, false) is 'A' then the following will hold:
    ///         peek_skip(0, true) == 'A'
    ///         peek_skip(1, true) == 'B'
    ///         peek_skip(1, false) == 'B'
    ///         peek_skip(2, false) == \n
    ///         peek_skip(2, true) == 'C'
    ///         peek_skip(3, true) == None (End of stream)
    ///  Note:
    ///     peek_skip(3, false) == 'C' iff peek_skip(3, true) hasn't been called previously, this is because
    ///     with skip_line_terminators == true the '\n' would be discarded. This leads to the following statements
    ///     evaluating to true (in isolation from each other or any other previous cursor calls):
    ///         peek_skip(3, false) == peek_skip(3, false) == '\n'
    ///         peek_skip(3, true) == peek_skip(3, true) == None
    ///         peek_skip(3, true) == peek_skip(3, false) == None
    ///         (peek_skip(3, false) == 'C') != (peek_skip(3, true) == None)
    ///
    /// Demonstration:
    ///
    /// ```rust
    ///
    /// ```
    ///
    pub(super) fn peek_skip(
        &mut self,
        skip_n: usize,
        skip_line_terminators: bool,
    ) -> Result<Option<Token>, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::peek_skip()", "Parsing");
        if skip_n > MAX_PEEK_SKIP {
            unimplemented!("peek_skip(n) where n > 3");
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
        let val = self.peeked[(self.back_index + skip_n) % PEEK_BUF_SIZE].clone();
        // println!("peek_skip val: {:?}", val);
        Ok(val)

        // if skip_line_terminators {
        //     if let Some(token) = val {
        //         if token.kind() == &TokenKind::LineTerminator {
        //             // unimplemented!("Skip line terminators");
        //             self.peeked[self.back_index].take();
        //             self.back_index = (self.back_index + 1) % PEEK_BUF_SIZE;
        //             self.peek(skip_line_terminators)
        //         } else {
        //             Ok(Some(token))
        //         }
        //     } else {
        //         Ok(None)
        //     }
        // } else {
        // }

        // println!("peek_skip val: {:?}", val);
        // Ok(val)

        // if self.buf_size == 0 {
        //     // No value has been peeked ahead already so need to go get the next value.

        //     self.peeked[self.front_index] = self.lexer.next(skip_line_terminators)?;
        //     self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;

        //     let index = self.front_index;

        //     self.peeked[self.front_index] = self.lexer.next(skip_line_terminators)?;
        //     self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;

        //     Ok(self.peeked[index].clone())
        // } else if ((self.back_index + 1) % PEEK_BUF_SIZE) == self.front_index {
        //     // Indicates only a single value has been peeked ahead already
        //     let index = self.front_index;

        //     self.peeked[self.front_index] = self.lexer.next(skip_line_terminators)?;
        //     self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;

        //     Ok(self.peeked[index].clone())
        // } else {
        //     Ok(self.peeked[(self.back_index + 1) % PEEK_BUF_SIZE].clone())
        // }
    }

    /// Takes the given token and pushes it back onto the parser token queue.
    ///
    /// Note: it pushes it at the the front so the token will be returned on next .peek().
    #[inline]
    pub(super) fn push_back(&mut self, token: Token) {
        // if ((self.front_index + 1) % PEEK_BUF_SIZE) == self.back_index {
        //     // Indicates that the buffer already contains a pushed back value and there is therefore
        //     // no space for another.
        //     unimplemented!("Push back more than once");
        // }

        if self.buf_size >= (PEEK_BUF_SIZE - 1) {
            unimplemented!("Push back more than once");
        }

        if self.back_index == 0 {
            self.back_index = PEEK_BUF_SIZE - 1;
        } else {
            self.back_index -= 1;
        };

        self.buf_size += 1;
        self.peeked[self.back_index] = Some(token);

        // if self.front_index == self.back_index {
        //     // No value peeked already.
        //     self.peeked[self.front_index] = Some(token);
        //     self.front_index = (self.front_index + 1) % PEEK_BUF_SIZE;
        // } else {
        //     if self.back_index == 0 {
        //         self.back_index = PEEK_BUF_SIZE - 1;
        //     } else {
        //         self.back_index -= 1;
        //     }

        //     self.peeked[self.back_index] = Some(token);
        // }
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
            .peek(skip_line_terminators)?
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
        match self.peek(false)? {
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

    /// It will make sure that the next token is not a line terminator.
    ///
    /// It expects that the token stream does not end here.
    ///
    /// If skip is true then the token after the peek() token is checked instead.
    pub(super) fn peek_expect_no_lineterminator(&mut self, skip: bool) -> Result<(), ParseError> {
        let token = if skip {
            self.peek_skip(1, false)?
        } else {
            self.peek(false)?
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
        Ok(if let Some(token) = self.peek(skip_line_terminators)? {
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
