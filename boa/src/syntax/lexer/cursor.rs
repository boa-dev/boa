//! Module implementing the lexer cursor. This is used for managing the input byte stream.

use crate::{profiler::BoaProfiler, syntax::ast::Position};
use std::io::{self, Bytes, Error, ErrorKind, Read};

/// Cursor over the source code.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    iter: InnerIter<R>,
    peeked: Option<Option<char>>,
    pos: Position,
}

impl<R> Cursor<R> {
    /// Gets the current position of the cursor in the source code.
    #[inline]
    pub(super) fn pos(&self) -> Position {
        self.pos
    }
    /// Advances the position to the next column.
    #[inline]
    pub(super) fn next_column(&mut self) {
        let current_line = self.pos.line_number();
        let next_column = self.pos.column_number() + 1;
        self.pos = Position::new(current_line, next_column);
    }

    /// Advances the position to the next line.
    #[inline]
    fn next_line(&mut self) {
        let next_line = self.pos.line_number() + 1;
        self.pos = Position::new(next_line, 1);
    }

    /// Performs a carriage return to modify the position in the source.
    #[inline]
    fn carriage_return(&mut self) {
        let current_line = self.pos.line_number();
        self.pos = Position::new(current_line, 1);
    }
}

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new Lexer cursor.
    #[inline]
    pub(super) fn new(inner: R) -> Self {
        Self {
            iter: InnerIter::new(inner.bytes()),
            peeked: None,
            pos: Position::new(1, 1),
        }
    }

    /// Peeks the next character.
    #[inline]
    pub(super) fn peek(&mut self) -> Result<Option<char>, Error> {
        let _timer = BoaProfiler::global().start_event("cursor::peek()", "Lexing");

        let iter = &mut self.iter;
        if let Some(v) = self.peeked {
            Ok(v)
        } else {
            let val = iter.next_char()?;
            self.peeked = Some(val);
            Ok(val)
        }
    }

    /// Compares the character passed in to the next character, if they match true is returned and the buffer is incremented
    #[inline]
    pub(super) fn next_is(&mut self, peek: char) -> io::Result<bool> {
        let _timer = BoaProfiler::global().start_event("cursor::next_is()", "Lexing");

        Ok(match self.peek()? {
            Some(next) if next == peek => {
                let _ = self.peeked.take();
                true
            }
            _ => false,
        })
    }

    /// Applies the predicate to the next character and returns the result.
    /// Returns false if there is no next character.
    ///
    /// The buffer is not incremented.
    #[inline]
    pub(super) fn next_is_pred<F>(&mut self, pred: &F) -> io::Result<bool>
    where
        F: Fn(char) -> bool,
    {
        let _timer = BoaProfiler::global().start_event("cursor::next_is_pred()", "Lexing");

        Ok(if let Some(peek) = self.peek()? {
            pred(peek)
        } else {
            false
        })
    }

    /// Fills the buffer with all characters until the stop character is found.
    ///
    /// Note: It will not add the stop character to the buffer.
    pub(super) fn take_until(&mut self, stop: char, buf: &mut String) -> io::Result<()> {
        let _timer = BoaProfiler::global().start_event("cursor::take_until()", "Lexing");

        loop {
            if self.next_is(stop)? {
                return Ok(());
            } else if let Some(ch) = self.next_char()? {
                buf.push(ch);
            } else {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Unexpected end of file when looking for character {}", stop),
                ));
            }
        }
    }

    /// Fills the buffer with characters until the first character (x) for which the predicate (pred) is false
    /// (or the next character is none).
    ///
    /// Note that all characters up until x are added to the buffer including the character right before.
    pub(super) fn take_while_pred<F>(&mut self, buf: &mut String, pred: &F) -> io::Result<()>
    where
        F: Fn(char) -> bool,
    {
        let _timer = BoaProfiler::global().start_event("cursor::take_while_pred()", "Lexing");

        loop {
            if !self.next_is_pred(pred)? {
                return Ok(());
            } else if let Some(ch) = self.next_char()? {
                buf.push(ch);
            } else {
                // next_is_pred will return false if the next value is None so the None case should already be handled.
                unreachable!();
            }
        }
    }

    /// It will fill the buffer with checked ASCII bytes.
    ///
    /// This expects for the buffer to be fully filled. If it's not, it will fail with an
    /// `UnexpectedEof` I/O error.
    #[inline]
    pub(super) fn fill_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let _timer = BoaProfiler::global().start_event("cursor::fill_bytes()", "Lexing");

        self.iter.fill_bytes(buf)
    }

    /// Retrieves the next UTF-8 character.
    #[inline]
    pub(crate) fn next_char(&mut self) -> Result<Option<char>, Error> {
        let _timer = BoaProfiler::global().start_event("cursor::next_char()", "Lexing");

        let chr = match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next_char()?,
        };

        match chr {
            Some('\r') => self.carriage_return(),
            Some('\n') | Some('\u{2028}') | Some('\u{2029}') => self.next_line(),
            Some(_) => self.next_column(),
            None => {}
        }

        Ok(chr)
    }
}

/// Inner iterator for a cursor.
#[derive(Debug)]
struct InnerIter<R> {
    iter: Bytes<R>,
}

impl<R> InnerIter<R> {
    /// Creates a new inner iterator.
    #[inline]
    fn new(iter: Bytes<R>) -> Self {
        Self { iter }
    }
}

impl<R> InnerIter<R>
where
    R: Read,
{
    /// It will fill the buffer with checked ASCII bytes.
    ///
    /// This expects for the buffer to be fully filled. If it's not, it will fail with an
    /// `UnexpectedEof` I/O error.
    #[inline]
    fn fill_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        for byte in buf.iter_mut() {
            *byte = self.next_ascii()?.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected EOF when filling buffer",
                )
            })?;
        }
        Ok(())
    }

    /// Retrieves the next UTF-8 checked character.
    fn next_char(&mut self) -> io::Result<Option<char>> {
        let first_byte = match self.iter.next().transpose()? {
            Some(b) => b,
            None => return Ok(None),
        };

        let chr: char = if first_byte < 0x80 {
            // 0b0xxx_xxxx
            first_byte.into()
        } else {
            let mut buf = [first_byte, 0u8, 0u8, 0u8];
            let num_bytes = if first_byte < 0xE0 {
                // 0b110x_xxxx
                2
            } else if first_byte < 0xF0 {
                // 0b1110_xxxx
                3
            } else {
                // 0b1111_0xxx
                4
            };

            for b in buf.iter_mut().take(num_bytes).skip(1) {
                let next = match self.iter.next() {
                    Some(Ok(b)) => b,
                    Some(Err(e)) => return Err(e),
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "stream did not contain valid UTF-8",
                        ))
                    }
                };

                *b = next;
            }

            if let Ok(s) = std::str::from_utf8(&buf) {
                if let Some(chr) = s.chars().next() {
                    chr
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "stream did not contain valid UTF-8",
                    ));
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "stream did not contain valid UTF-8",
                ));
            }
        };

        Ok(Some(chr))
    }

    /// Retrieves the next ASCII checked character.
    #[inline]
    fn next_ascii(&mut self) -> io::Result<Option<u8>> {
        let next_byte = self.iter.next().transpose()?;

        match next_byte {
            Some(next) if next <= 0x7F => Ok(Some(next)),
            None => Ok(None),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-ASCII byte found",
            )),
        }
    }
}
