//! Boa's lexer cursor that manages the input byte stream.
use boa_ast::Position;
use boa_profiler::Profiler;
use std::io::{self, Bytes, Error, ErrorKind, Read};

/// Cursor over the source code.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    iter: InnerIter<R>,
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

impl<R> Cursor<R>
where
    R: Read,
{
    /// Creates a new Lexer cursor.
    pub(super) fn new(inner: R) -> Self {
        Self {
            iter: InnerIter::new(inner.bytes()),
            pos: Position::new(1, 1),
            strict: false,
            module: false,
            peeked: [None; 4],
        }
    }

    /// Creates a new Lexer cursor with an initial position.
    pub(super) fn with_position(inner: R, pos: Position) -> Self {
        Self {
            iter: InnerIter::new(inner.bytes()),
            pos,
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

/// Inner iterator for a cursor.
#[derive(Debug)]
struct InnerIter<R> {
    iter: Bytes<R>,
}

impl<R> InnerIter<R> {
    /// Creates a new inner iterator.
    const fn new(iter: Bytes<R>) -> Self {
        Self { iter }
    }
}

impl<R> InnerIter<R>
where
    R: Read,
{
    /// Retrieves the next byte
    fn next_byte(&mut self) -> io::Result<Option<u8>> {
        self.iter.next().transpose()
    }

    /// Retrieves the next unchecked char in u32 code point.
    fn next_char(&mut self) -> io::Result<Option<u32>> {
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
fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    u32::from(byte & (0x7F >> width))
}

/// Returns the value of `ch` updated with continuation byte `byte`.
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | u32::from(byte & CONT_MASK)
}

fn unwrap_or_0(opt: Option<u8>) -> u8 {
    opt.unwrap_or(0)
}
