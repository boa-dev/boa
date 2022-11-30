use crate::{
    lexer::{InputElement, Lexer, Token, TokenKind},
    parser::ParseResult,
    Error,
};
use boa_ast::Position;
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

#[cfg(test)]
mod tests;

/// The maximum number of tokens which can be peeked ahead.
const MAX_PEEK_SKIP: usize = 3;

/// The fixed size of the buffer used for storing values that are peeked ahead.
///
/// The size is calculated for a worst case scenario, where we want to peek `MAX_PEEK_SKIP` tokens
/// skipping line terminators, and the stream ends just after:
/// ```text
/// [\n, B, \n, C, \n, D, \n, E, \n, F, None]
///   0  0   1  1   2  2   3  3   4  4  5
/// ```
const PEEK_BUF_SIZE: usize = (MAX_PEEK_SKIP + 1) * 2 + 1;

#[derive(Debug)]
pub(super) struct BufferedLexer<R> {
    lexer: Lexer<R>,
    peeked: [Option<Token>; PEEK_BUF_SIZE],
    read_index: usize,
    write_index: usize,
}

impl<R> From<Lexer<R>> for BufferedLexer<R>
where
    R: Read,
{
    #[inline]
    fn from(lexer: Lexer<R>) -> Self {
        Self {
            lexer,
            peeked: [
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
                None::<Token>,
            ],
            read_index: 0,
            write_index: 0,
        }
    }
}

impl<R> From<R> for BufferedLexer<R>
where
    R: Read,
{
    #[inline]
    fn from(reader: R) -> Self {
        Lexer::new(reader).into()
    }
}

impl<R> BufferedLexer<R>
where
    R: Read,
{
    /// Sets the goal symbol for the lexer.
    #[inline]
    pub(super) fn set_goal(&mut self, elm: InputElement) {
        let _timer = Profiler::global().start_event("cursor::set_goal()", "Parsing");
        self.lexer.set_goal(elm);
    }

    /// Lexes the next tokens as a regex assuming that the starting '/' has already been consumed.
    #[inline]
    pub(super) fn lex_regex(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> ParseResult<Token> {
        let _timer = Profiler::global().start_event("cursor::lex_regex()", "Parsing");
        self.set_goal(InputElement::RegExp);
        self.lexer
            .lex_slash_token(start, interner)
            .map_err(Into::into)
    }

    /// Lexes the next tokens as template middle or template tail assuming that the starting
    /// '}' has already been consumed.
    pub(super) fn lex_template(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> ParseResult<Token> {
        self.lexer
            .lex_template(start, interner)
            .map_err(Error::from)
    }

    #[inline]
    pub(super) const fn strict_mode(&self) -> bool {
        self.lexer.strict_mode()
    }

    #[inline]
    pub(super) fn set_strict_mode(&mut self, strict_mode: bool) {
        self.lexer.set_strict_mode(strict_mode);
    }

    /// Fills the peeking buffer with the next token.
    ///
    /// It will not fill two line terminators one after the other.
    fn fill(&mut self, interner: &mut Interner) -> ParseResult<()> {
        debug_assert!(
            self.write_index < PEEK_BUF_SIZE,
            "write index went out of bounds"
        );

        let previous_index = self.write_index.checked_sub(1).unwrap_or(PEEK_BUF_SIZE - 1);

        if let Some(ref token) = self.peeked[previous_index] {
            if token.kind() == &TokenKind::LineTerminator {
                // We don't want to have multiple contiguous line terminators in the buffer, since
                // they have no meaning.
                let next = loop {
                    let next = self.lexer.next(interner)?;
                    if let Some(ref token) = next {
                        if token.kind() != &TokenKind::LineTerminator {
                            break next;
                        }
                    } else {
                        break None;
                    }
                };

                self.peeked[self.write_index] = next;
            } else {
                self.peeked[self.write_index] = self.lexer.next(interner)?;
            }
        } else {
            self.peeked[self.write_index] = self.lexer.next(interner)?;
        }
        self.write_index = (self.write_index + 1) % PEEK_BUF_SIZE;

        debug_assert_ne!(
            self.read_index, self.write_index,
            "we reached the read index with the write index"
        );
        debug_assert!(
            self.read_index < PEEK_BUF_SIZE,
            "read index went out of bounds"
        );

        Ok(())
    }

    /// Moves the cursor to the next token and returns the token.
    ///
    /// If `skip_line_terminators` is true then line terminators will be discarded.
    ///
    /// This follows iterator semantics in that a `peek(0, false)` followed by a `next(false)` will
    /// return the same value. Note that because a `peek(n, false)` may return a line terminator a
    /// subsequent `next(true)` may not return the same value.
    pub(super) fn next(
        &mut self,
        skip_line_terminators: bool,
        interner: &mut Interner,
    ) -> ParseResult<Option<Token>> {
        if self.read_index == self.write_index {
            self.fill(interner)?;
        }

        if let Some(ref token) = self.peeked[self.read_index] {
            if skip_line_terminators && token.kind() == &TokenKind::LineTerminator {
                // We only store 1 contiguous line terminator, so if the one at `self.read_index`
                // was a line terminator, we know that the next won't be one.
                self.read_index = (self.read_index + 1) % PEEK_BUF_SIZE;
                if self.read_index == self.write_index {
                    self.fill(interner)?;
                }
            }
            let tok = self.peeked[self.read_index].take();
            self.read_index = (self.read_index + 1) % PEEK_BUF_SIZE;

            Ok(tok)
        } else {
            // We do not update the read index, since we should always return `None` from now on.
            Ok(None)
        }
    }

    /// Peeks the `n`th token after the next token.
    ///
    /// **Note:** `n` must be in the range `[0, 3]`.
    /// i.e. if there are tokens `A`, `B`, `C`, `D`, `E` and `peek(0, false)` returns `A` then:
    ///  - `peek(1, false) == peek(1, true) == B`.
    ///  - `peek(2, false)` will return `C`.
    /// where `A`, `B`, `C`, `D` and `E` are tokens but not line terminators.
    ///
    /// If `skip_line_terminators` is `true` then line terminators will be discarded.
    /// i.e. If there are tokens `A`, `\n`, `B` and `peek(0, false)` is `A` then the following
    /// will hold:
    ///  - `peek(0, true) == A`
    ///  - `peek(0, false) == A`
    ///  - `peek(1, true) == B`
    ///  - `peek(1, false) == \n`
    ///  - `peek(2, true) == None` (End of stream)
    ///  - `peek(2, false) == B`
    pub(super) fn peek(
        &mut self,
        skip_n: usize,
        skip_line_terminators: bool,
        interner: &mut Interner,
    ) -> ParseResult<Option<&Token>> {
        assert!(
            skip_n <= MAX_PEEK_SKIP,
            "you cannot skip more than {} elements",
            MAX_PEEK_SKIP
        );

        let mut read_index = self.read_index;
        let mut count = 0;
        let res_token = loop {
            if read_index == self.write_index {
                self.fill(interner)?;
            }

            if let Some(ref token) = self.peeked[read_index] {
                if skip_line_terminators && token.kind() == &TokenKind::LineTerminator {
                    read_index = (read_index + 1) % PEEK_BUF_SIZE;
                    // We only store 1 contiguous line terminator, so if the one at `self.read_index`
                    // was a line terminator, we know that the next won't be one.
                    if read_index == self.write_index {
                        self.fill(interner)?;
                    }
                }
                if count == skip_n {
                    break self.peeked[read_index].as_ref();
                }
            } else {
                break None;
            }
            read_index = (read_index + 1) % PEEK_BUF_SIZE;
            count += 1;
        };

        Ok(res_token)
    }
}
