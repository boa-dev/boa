use crate::{
    profiler::BoaProfiler,
    syntax::{
        lexer::{InputElement, Lexer, Position, Token, TokenKind},
        parser::error::ParseError,
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

#[derive(Debug)]
pub(super) struct BufferedLexer<R> {
    lexer: Lexer<R>,
    peeked: [Option<Token>; PEEK_BUF_SIZE],
    buf_size: usize,
    back_index: usize,
}

impl<R> From<Lexer<R>> for BufferedLexer<R>
where
    R: Read,
{
    #[inline(always)]
    fn from(lexer: Lexer<R>) -> Self {
        Self {
            lexer,
            peeked: [
                None::<Token>;
                PEEK_BUF_SIZE
            ],
            buf_size: 0,
            back_index: 0,
        }
    }
}

impl<R> From<R> for BufferedLexer<R>
where
    R: Read,
{
    #[inline(always)]
    fn from(reader: R) -> Self {
        Lexer::new(reader).into()
    }
}

impl<R> BufferedLexer<R>
where
    R: Read,
{
    /// Sets the goal symbol for the lexer.
    #[inline(always)]
    pub(super) fn set_goal(&mut self, elm: InputElement) {
        let _timer = BoaProfiler::global().start_event("cursor::set_goal()", "Parsing");
        self.lexer.set_goal(elm)
    }

    /// Lexes the next tokens as a regex assuming that the starting '/' has already been consumed.
    #[inline(always)]
    pub(super) fn lex_regex(&mut self, start: Position) -> Result<Token, ParseError> {
        let _timer = BoaProfiler::global().start_event("cursor::lex_regex()", "Parsing");
        self.set_goal(InputElement::RegExp);
        self.lexer.lex_slash_token(start).map_err(|e| e.into())
    }

    /// Moves the cursor to the next token and returns the token.
    ///
    /// If skip_line_terminators is true then line terminators will be discarded.
    ///
    /// This follows iterator semantics in that a peek(0, false) followed by a next(false) will return the same value.
    /// Note that because a peek(n, false) may return a line terminator a subsequent next(true) may not return the same value.
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

    /// Takes the given token and pushes it back onto the parser token queue.
    ///
    /// Note: it pushes it at the the front so the token will be returned on next .peek().
    ///
    /// A push_back call must never (directly or indirectly) follow another push_back call without a next between them.
    #[inline]
    pub(super) fn push_back(&mut self, token: Token) {
        let _timer = BoaProfiler::global().start_event("cursor::push_back()", "Parsing");
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
    }
}
