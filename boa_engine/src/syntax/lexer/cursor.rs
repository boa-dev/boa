//! Module implementing the lexer cursor. This is used for managing the input byte stream.
use crate::syntax::ast::Position;
use boa_profiler::Profiler;
use std::io::{self, Bytes, Error, ErrorKind, Read};

/// Cursor over the source code.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    iter: InnerIter<R>,
    pos: Position,
    strict_mode: bool,
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

    #[inline]
    pub(super) fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    #[inline]
    pub(super) fn set_strict_mode(&mut self, strict_mode: bool) {
        self.strict_mode = strict_mode;
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
            pos: Position::new(1, 1),
            strict_mode: false,
        }
    }

    /// Creates a new Lexer cursor with an initial position.
    #[inline]
    pub(super) fn with_position(inner: R, pos: Position) -> Self {
        Self {
            iter: InnerIter::new(inner.bytes()),
            pos,
            strict_mode: false,
        }
    }

    /// Peeks the next byte.
    #[inline]
    pub(super) fn peek(&mut self) -> Result<Option<u8>, Error> {
        let _timer = Profiler::global().start_event("cursor::peek()", "Lexing");

        self.iter.peek_byte()
    }

    /// Peeks the next n bytes, the maximum number of peeked bytes is 4 (n <= 4).
    #[inline]
    pub(super) fn peek_n(&mut self, n: u8) -> Result<u32, Error> {
        let _timer = Profiler::global().start_event("cursor::peek_n()", "Lexing");

        self.iter.peek_n_bytes(n)
    }

    /// Peeks the next UTF-8 character in u32 code point.
    #[inline]
    pub(super) fn peek_char(&mut self) -> Result<Option<u32>, Error> {
        let _timer = Profiler::global().start_event("cursor::peek_char()", "Lexing");

        self.iter.peek_char()
    }

    /// Compares the byte passed in to the next byte, if they match true is returned and the buffer is incremented
    #[inline]
    pub(super) fn next_is(&mut self, byte: u8) -> io::Result<bool> {
        let _timer = Profiler::global().start_event("cursor::next_is()", "Lexing");

        Ok(match self.peek()? {
            Some(next) if next == byte => {
                let _ = self.next_byte()?;
                true
            }
            _ => false,
        })
    }

    /// Applies the predicate to the next character and returns the result.
    /// Returns false if the next character is not a valid ascii or there is no next character.
    /// Otherwise returns the result from the predicate on the ascii in char
    ///
    /// The buffer is not incremented.
    #[inline]
    pub(super) fn next_is_ascii_pred<F>(&mut self, pred: &F) -> io::Result<bool>
    where
        F: Fn(char) -> bool,
    {
        let _timer = Profiler::global().start_event("cursor::next_is_ascii_pred()", "Lexing");

        Ok(match self.peek()? {
            Some(byte) => match byte {
                0..=0x7F => pred(char::from(byte)),
                _ => false,
            },
            None => false,
        })
    }

    /// Applies the predicate to the next UTF-8 character and returns the result.
    /// Returns false if there is no next character, otherwise returns the result from the
    /// predicate on the ascii char
    ///
    /// The buffer is not incremented.
    #[allow(dead_code)]
    #[inline]
    pub(super) fn next_is_char_pred<F>(&mut self, pred: &F) -> io::Result<bool>
    where
        F: Fn(u32) -> bool,
    {
        let _timer = Profiler::global().start_event("cursor::next_is_char_pred()", "Lexing");

        Ok(if let Some(peek) = self.peek_char()? {
            pred(peek)
        } else {
            false
        })
    }

    /// Fills the buffer with all bytes until the stop byte is found.
    /// Returns error when reaching the end of the buffer.
    ///
    /// Note that all bytes up until the stop byte are added to the buffer, including the byte right before.
    pub(super) fn take_until(&mut self, stop: u8, buf: &mut Vec<u8>) -> io::Result<()> {
        let _timer = Profiler::global().start_event("cursor::take_until()", "Lexing");

        loop {
            if self.next_is(stop)? {
                return Ok(());
            } else if let Some(byte) = self.next_byte()? {
                buf.push(byte);
            } else {
                return Err(io::Error::new(
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
            } else if let Some(byte) = self.next_byte()? {
                buf.push(byte);
            } else {
                // next_is_pred will return false if the next value is None so the None case should already be handled.
                unreachable!();
            }
        }
    }

    /// Fills the buffer with characters until the first character for which the predicate (pred) is false.
    /// It also stops when there is no next character.
    ///
    /// Note that all characters up until the stop character are added to the buffer, including the character right before.
    #[allow(dead_code)]
    pub(super) fn take_while_char_pred<F>(&mut self, buf: &mut Vec<u8>, pred: &F) -> io::Result<()>
    where
        F: Fn(u32) -> bool,
    {
        let _timer = Profiler::global().start_event("cursor::take_while_char_pred()", "Lexing");

        loop {
            if !self.next_is_char_pred(pred)? {
                return Ok(());
            } else if let Some(ch) = self.peek_char()? {
                for _ in 0..utf8_len(ch) {
                    buf.push(
                        self.next_byte()?
                            .expect("already checked that the next character exists"),
                    );
                }
            } else {
                // next_is_pred will return false if the next value is None so the None case should already be handled.
                unreachable!();
            }
        }
    }

    /// It will fill the buffer with bytes.
    ///
    /// This expects for the buffer to be fully filled. If it's not, it will fail with an
    /// `UnexpectedEof` I/O error.
    #[inline]
    pub(super) fn fill_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let _timer = Profiler::global().start_event("cursor::fill_bytes()", "Lexing");

        self.iter.fill_bytes(buf)
    }

    /// Retrieves the next byte.
    #[inline]
    pub(crate) fn next_byte(&mut self) -> Result<Option<u8>, Error> {
        let _timer = Profiler::global().start_event("cursor::next_byte()", "Lexing");

        let byte = self.iter.next_byte()?;

        match byte {
            Some(b'\r') => {
                // Try to take a newline if it's next, for windows "\r\n" newlines
                // Otherwise, treat as a Mac OS9 bare '\r' newline
                if self.peek()? == Some(b'\n') {
                    let _next = self.iter.next_byte();
                }
                self.next_line();
            }
            Some(b'\n') => self.next_line(),
            Some(0xE2) => {
                // Try to match '\u{2028}' (e2 80 a8) and '\u{2029}' (e2 80 a9)
                let next_bytes = self.peek_n(2)?;
                if next_bytes == 0xA8_80 || next_bytes == 0xA9_80 {
                    self.next_line();
                } else {
                    // 0xE2 is a utf8 first byte
                    self.next_column();
                }
            }
            Some(b) if utf8_is_first_byte(b) => self.next_column(),
            _ => {}
        }

        Ok(byte)
    }

    /// Retrieves the next UTF-8 character.
    #[inline]
    pub(crate) fn next_char(&mut self) -> Result<Option<u32>, Error> {
        let _timer = Profiler::global().start_event("cursor::next_char()", "Lexing");

        let ch = self.iter.next_char()?;

        match ch {
            Some(0xD) => {
                // Try to take a newline if it's next, for windows "\r\n" newlines
                // Otherwise, treat as a Mac OS9 bare '\r' newline
                if self.peek()? == Some(0xA) {
                    let _next = self.iter.next_byte();
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

/// Inner iterator for a cursor.
#[derive(Debug)]
#[allow(clippy::option_option)]
struct InnerIter<R> {
    iter: Bytes<R>,
    num_peeked_bytes: u8,
    peeked_bytes: u32,
    peeked_char: Option<Option<u32>>,
}

impl<R> InnerIter<R> {
    /// Creates a new inner iterator.
    #[inline]
    fn new(iter: Bytes<R>) -> Self {
        Self {
            iter,
            num_peeked_bytes: 0,
            peeked_bytes: 0,
            peeked_char: None,
        }
    }
}

impl<R> InnerIter<R>
where
    R: Read,
{
    /// It will fill the buffer with checked ascii bytes.
    ///
    /// This expects for the buffer to be fully filled. If it's not, it will fail with an
    /// `UnexpectedEof` I/O error.
    #[inline]
    fn fill_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        for byte in buf.iter_mut() {
            *byte = self.next_byte()?.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected EOF when filling buffer",
                )
            })?;
        }
        Ok(())
    }

    /// Increments the iter by n bytes.
    #[inline]
    fn increment(&mut self, n: u32) -> Result<(), Error> {
        for _ in 0..n {
            if None == self.next_byte()? {
                break;
            }
        }
        Ok(())
    }

    /// Peeks the next byte.
    #[inline]
    pub(super) fn peek_byte(&mut self) -> Result<Option<u8>, Error> {
        if self.num_peeked_bytes > 0 {
            let byte = self.peeked_bytes as u8;
            Ok(Some(byte))
        } else {
            match self.iter.next().transpose()? {
                Some(byte) => {
                    self.num_peeked_bytes = 1;
                    self.peeked_bytes = u32::from(byte);
                    Ok(Some(byte))
                }
                None => Ok(None),
            }
        }
    }

    /// Peeks the next n bytes, the maximum number of peeked bytes is 4 (n <= 4).
    #[inline]
    pub(super) fn peek_n_bytes(&mut self, n: u8) -> Result<u32, Error> {
        while self.num_peeked_bytes < n && self.num_peeked_bytes < 4 {
            match self.iter.next().transpose()? {
                Some(byte) => {
                    self.peeked_bytes |= u32::from(byte) << (self.num_peeked_bytes * 8);
                    self.num_peeked_bytes += 1;
                }
                None => break,
            };
        }

        match n {
            0 => Ok(0),
            1 => Ok(self.peeked_bytes & 0xFF),
            2 => Ok(self.peeked_bytes & 0xFFFF),
            3 => Ok(self.peeked_bytes & 0xFFFFFF),
            _ => Ok(self.peeked_bytes),
        }
    }

    /// Peeks the next unchecked character in u32 code point.
    #[inline]
    pub(super) fn peek_char(&mut self) -> Result<Option<u32>, Error> {
        if let Some(ch) = self.peeked_char {
            Ok(ch)
        } else {
            // Decode UTF-8
            let x = match self.peek_byte()? {
                Some(b) if b < 128 => {
                    self.peeked_char = Some(Some(u32::from(b)));
                    return Ok(Some(u32::from(b)));
                }
                Some(b) => b,
                None => {
                    self.peeked_char = None;
                    return Ok(None);
                }
            };

            // Multibyte case follows
            // Decode from a byte combination out of: [[[x y] z] w]
            // NOTE: Performance is sensitive to the exact formulation here
            let init = utf8_first_byte(x, 2);
            let y = (self.peek_n_bytes(2)? >> 8) as u8;
            let mut ch = utf8_acc_cont_byte(init, y);
            if x >= 0xE0 {
                // [[x y z] w] case
                // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
                let z = (self.peek_n_bytes(3)? >> 16) as u8;
                let y_z = utf8_acc_cont_byte(u32::from(y & CONT_MASK), z);
                ch = init << 12 | y_z;
                if x >= 0xF0 {
                    // [x y z w] case
                    // use only the lower 3 bits of `init`
                    let w = (self.peek_n_bytes(4)? >> 24) as u8;
                    ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
                }
            };

            self.peeked_char = Some(Some(ch));
            Ok(Some(ch))
        }
    }

    /// Retrieves the next byte
    #[inline]
    fn next_byte(&mut self) -> io::Result<Option<u8>> {
        self.peeked_char = None;
        if self.num_peeked_bytes > 0 {
            let byte = (self.peeked_bytes & 0xFF) as u8;
            self.num_peeked_bytes -= 1;
            self.peeked_bytes >>= 8;
            Ok(Some(byte))
        } else {
            self.iter.next().transpose()
        }
    }

    /// Retrieves the next unchecked char in u32 code point.
    #[inline]
    fn next_char(&mut self) -> io::Result<Option<u32>> {
        if let Some(ch) = self.peeked_char.take() {
            if let Some(c) = ch {
                self.increment(utf8_len(c))?;
            }
            return Ok(ch);
        }

        // Decode UTF-8
        let x = match self.next_byte()? {
            Some(b) if b < 128 => return Ok(Some(u32::from(b))),
            Some(b) => b,
            None => return Ok(None),
        };

        // Multibyte case follows
        // Decode from a byte combination out of: [[[x y] z] w]
        // NOTE: Performance is sensitive to the exact formulation here
        let init = utf8_first_byte(x, 2);
        let y = unwrap_or_0(self.next_byte()?);
        let mut ch = utf8_acc_cont_byte(init, y);
        if x >= 0xE0 {
            // [[x y z] w] case
            // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
            let z = unwrap_or_0(self.next_byte()?);
            let y_z = utf8_acc_cont_byte(u32::from(y & CONT_MASK), z);
            ch = init << 12 | y_z;
            if x >= 0xF0 {
                // [x y z w] case
                // use only the lower 3 bits of `init`
                let w = unwrap_or_0(self.next_byte()?);
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        };

        Ok(Some(ch))
    }
}

/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;

/// Returns the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
#[inline]
fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    u32::from(byte & (0x7F >> width))
}

/// Returns the value of `ch` updated with continuation byte `byte`.
#[inline]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | u32::from(byte & CONT_MASK)
}

/// Checks whether the byte is a UTF-8 first byte (i.e., ascii byte or starts with the
/// bits `11`).
#[inline]
fn utf8_is_first_byte(byte: u8) -> bool {
    byte <= 0x7F || (byte >> 6) == 0x11
}

#[inline]
fn unwrap_or_0(opt: Option<u8>) -> u8 {
    opt.unwrap_or(0)
}

#[inline]
fn utf8_len(ch: u32) -> u32 {
    if ch <= 0x7F {
        1
    } else if ch <= 0x7FF {
        2
    } else if ch <= 0xFFFF {
        3
    } else {
        4
    }
}
