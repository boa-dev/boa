use crate::syntax::ast::Position;
use std::io::{self, Bytes, ErrorKind, Read};

/// Cursor over the source code.
#[derive(Debug)]
pub(super) struct Cursor<R> {
    iter: InnerIter<R>,
    peeked: Option<Option<io::Result<char>>>,
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
    pub(super) fn next_line(&mut self) {
        let next_line = self.pos.line_number() + 1;
        self.pos = Position::new(next_line, 1);
    }

    /// Performs a carriage return to modify the position in the source.
    #[inline]
    pub(super) fn carriage_return(&mut self) {
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
    pub(super) fn peek(&mut self) -> Option<&io::Result<char>> {
        let iter = &mut self.iter;
        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    /// Compares the character passed in to the next character, if they match true is returned and the buffer is incremented
    #[inline]
    pub(super) fn next_is(&mut self, peek: char) -> io::Result<bool> {
        Ok(match self.peek() {
            None => false,
            Some(&Ok(next)) if next == peek => {
                let _ = self.peeked.take();
                true
            }
            _ => false,
            // Some(&Err(_)) => return self.peeked.take().unwrap().unwrap().map(|_| false),
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
        Ok(match self.peek() {
            None => false,
            Some(Ok(peek)) => pred(*peek),
            Some(Err(e)) => todo!(),
        })
    }

    /// Fills the buffer with all characters until the stop character is found.
    ///
    /// Note: It will not add the stop character to the buffer.
    pub(super) fn take_until(&mut self, stop: char, buf: &mut String) -> io::Result<()> {
        loop {
            if self.next_is(stop)? {
                return Ok(());
            } else {
                match self.next() {
                    None => {
                        return Err(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            format!("Unexpected end of file when looking for character {}", stop),
                        ));
                    }
                    Some(Err(e)) => {
                        return Err(e);
                    }
                    Some(Ok(ch)) => {
                        buf.push(ch);
                    }
                }
            }
        }
    }

    /// Fills the buffer with characters until the first character (x) for which the predicate (pred) is false
    /// (or the next character is none).
    ///
    /// Note that all characters up until x are added to the buffer including the character right before.
    pub(super) fn take_until_pred<F>(&mut self, buf: &mut String, pred: &F) -> io::Result<()>
    where
        F: Fn(char) -> bool,
    {
        loop {
            if !self.next_is_pred(pred)? {
                return Ok(());
            } else {
                match self.next() {
                    None => {
                        unimplemented!();
                        // unreachable!();
                    }
                    Some(Err(e)) => {
                        return Err(e);
                    }
                    Some(Ok(ch)) => {
                        buf.push(ch);
                    }
                }
            }
        }
    }

    /// It will fill the buffer with checked ASCII bytes.
    pub(super) fn fill_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        unimplemented!()
    }
}

impl<R> Iterator for Cursor<R>
where
    R: Read,
{
    type Item = io::Result<char>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let chr = match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next(),
        };

        match chr {
            Some(Ok('\r')) => self.carriage_return(),
            Some(Ok('\n')) | Some(Ok('\u{2028}')) | Some(Ok('\u{2029}')) => self.next_line(),
            Some(Ok(_)) => self.next_column(),
            _ => {}
        }

        chr
    }
}

/// Inner iterator for a cursor.
#[derive(Debug)]
struct InnerIter<R> {
    iter: Bytes<R>,
}

impl<R> InnerIter<R> {
    /// Creates a new inner iterator.
    fn new(iter: Bytes<R>) -> Self {
        Self { iter }
    }
}

impl<R> Iterator for InnerIter<R>
where
    R: Read,
{
    type Item = io::Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        let first_byte = match self.iter.next()? {
            Ok(b) => b,
            Err(e) => return Some(Err(e)),
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
                    Some(Err(e)) => return Some(Err(e)),
                    None => {
                        return Some(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "stream did not contain valid UTF-8",
                        )))
                    }
                };

                *b = next;
            }

            match std::str::from_utf8(&buf) {
                Ok(s) => match s.chars().next() {
                    Some(chr) => chr,
                    None => {
                        return Some(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "stream did not contain valid UTF-8",
                        )));
                    }
                },
                Err(_) => {
                    return Some(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "stream did not contain valid UTF-8",
                    )));
                }
            }
        };

        Some(Ok(chr))
    }
}
