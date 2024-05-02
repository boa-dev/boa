//! Boa's lexer cursor that manages the input byte stream.

use crate::source::{ReadChar, UTF8Input};
use boa_ast::Position;
use boa_profiler::Profiler;
use std::io::{self, Error, ErrorKind};

/// Cursor over the source code.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    iter: R,
    pos: Position,
    module: bool,
    strict: bool,
    peeked: [Option<u32>; 4],
}

impl<R> Cursor<R> {
    /// Gets the current position of the cursor in the source code.
    pub(super) const fn pos(&self) -> Position {
        self.pos
    }

    /// Advances the position to the next column.
    fn next_column(&mut self) {
        let current_line = self.pos.line_number();
        let next_column = self.pos.column_number() + 1;
        self.pos = Position::new(current_line, next_column);
    }

    /// Advances the position to the next line.
    fn next_line(&mut self) {
        let next_line = self.pos.line_number() + 1;
        self.pos = Position::new(next_line, 1);
    }

    /// Returns if strict mode is currently active.
    pub(super) const fn strict(&self) -> bool {
        self.strict
    }

    /// Sets the current strict mode.
    pub(super) fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Returns if the module mode is currently active.
    pub(super) const fn module(&self) -> bool {
        self.module
    }

    /// Sets the current goal symbol to module.
    pub(super) fn set_module(&mut self, module: bool) {
        self.module = module;
        self.strict = module;
    }
}

impl<R: ReadChar> Cursor<R> {
    /// Creates a new Lexer cursor.
    pub(super) fn new(inner: R) -> Self {
        Self {
            iter: inner,
            pos: Position::new(1, 1),
            strict: false,
            module: false,
            peeked: [None; 4],
        }
    }

    /// Peeks the next n bytes, the maximum number of peeked bytes is 4 (n <= 4).
    pub(super) fn peek_n(&mut self, n: u8) -> Result<&[Option<u32>; 4], Error> {
        let _timer = Profiler::global().start_event("cursor::peek_n()", "Lexing");

        let peeked = self.peeked.iter().filter(|c| c.is_some()).count();
        let needs_peek = n as usize - peeked;

        for i in 0..needs_peek {
            let next = self.iter.next_char()?;
            self.peeked[i + peeked] = next;
        }

        Ok(&self.peeked)
    }

    /// Peeks the next UTF-8 character in u32 code point.
    pub(super) fn peek_char(&mut self) -> Result<Option<u32>, Error> {
        let _timer = Profiler::global().start_event("cursor::peek_char()", "Lexing");

        if let Some(c) = self.peeked[0] {
            return Ok(Some(c));
        }

        let next = self.iter.next_char()?;
        self.peeked[0] = next;
        Ok(next)
    }

    pub(super) fn next_if(&mut self, c: u32) -> io::Result<bool> {
        let _timer = Profiler::global().start_event("cursor::next_if()", "Lexing");

        if self.peek_char()? == Some(c) {
            self.next_char()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Applies the predicate to the next character and returns the result.
    /// Returns false if the next character is not a valid ascii or there is no next character.
    /// Otherwise returns the result from the predicate on the ascii in char
    ///
    /// The buffer is not incremented.
    pub(super) fn next_is_ascii_pred<F>(&mut self, pred: &F) -> io::Result<bool>
    where
        F: Fn(char) -> bool,
    {
        let _timer = Profiler::global().start_event("cursor::next_is_pred()", "Lexing");

        Ok(match self.peek_char()? {
            Some(byte) if (0..=0x7F).contains(&byte) =>
            {
                #[allow(clippy::cast_possible_truncation)]
                pred(char::from(byte as u8))
            }
            Some(_) | None => false,
        })
    }

    /// Fills the buffer with all bytes until the stop byte is found.
    /// Returns error when reaching the end of the buffer.
    ///
    /// Note that all bytes up until the stop byte are added to the buffer, including the byte right before.
    pub(super) fn take_until(&mut self, stop: u32, buf: &mut Vec<u32>) -> io::Result<()> {
        let _timer = Profiler::global().start_event("cursor::take_until()", "Lexing");

        loop {
            if self.next_if(stop)? {
                return Ok(());
            } else if let Some(c) = self.next_char()? {
                buf.push(c);
            } else {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Unexpected end of file when looking for character {stop}"),
                ));
            }
        }
    }

    /// Fills the buffer with characters until the first ascii character for which the predicate (pred) is false.
    /// It also stops when the next character is not an ascii or there is no next character.
    ///
    /// Note that all characters up until the stop character are added to the buffer, including the character right before.
    pub(super) fn take_while_ascii_pred<F>(&mut self, buf: &mut Vec<u8>, pred: &F) -> io::Result<()>
    where
        F: Fn(char) -> bool,
    {
        let _timer = Profiler::global().start_event("cursor::take_while_ascii_pred()", "Lexing");

        loop {
            if !self.next_is_ascii_pred(pred)? {
                return Ok(());
            } else if let Some(byte) = self.next_char()? {
                #[allow(clippy::cast_possible_truncation)]
                buf.push(byte as u8);
            } else {
                // next_is_pred will return false if the next value is None so the None case should already be handled.
                unreachable!();
            }
        }
    }

    /// Retrieves the next UTF-8 character.
    pub(crate) fn next_char(&mut self) -> Result<Option<u32>, Error> {
        let _timer = Profiler::global().start_event("cursor::next_char()", "Lexing");

        let ch = if let Some(c) = self.peeked[0] {
            self.peeked[0] = None;
            self.peeked.rotate_left(1);
            Some(c)
        } else {
            self.iter.next_char()?
        };

        match ch {
            Some(0xD) => {
                // Try to take a newline if it's next, for windows "\r\n" newlines
                // Otherwise, treat as a Mac OS9 bare '\r' newline
                if self.peek_char()? == Some(0xA) {
                    self.peeked[0] = None;
                    self.peeked.rotate_left(1);
                }
                self.next_line();
            }
            // '\n' | '\u{2028}' | '\u{2029}'
            Some(0xA | 0x2028 | 0x2029) => self.next_line(),
            Some(_) => self.next_column(),
            _ => {}
        }

        Ok(ch)
    }
}

impl<'a> From<&'a [u8]> for Cursor<UTF8Input<&'a [u8]>> {
    fn from(input: &'a [u8]) -> Self {
        Self::new(UTF8Input::new(input))
    }
}
