//! Cursor implementation for the parser.

use crate::error::{Error, ParseResult};

/// Binary cursor.
///
/// This internal structure gives basic testable operations to the parser.
#[derive(Debug, Clone)]
pub(super) struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor over the given byte slice.
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Returns the number of bytes remaining.
    pub(super) fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    /// Returns `true` if all input has been consumed.
    pub(super) fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Reads a single byte, advancing the cursor.
    pub(super) fn read_u8(&mut self, context: &'static str) -> ParseResult<u8> {
        if self.pos >= self.data.len() {
            return Err(Error::UnexpectedEof { context });
        }
        let byte = self.data[self.pos];
        self.pos += 1;
        Ok(byte)
    }

    /// Reads exactly `len` bytes as a sub-slice, advancing the cursor.
    pub(super) fn read_bytes(
        &mut self,
        len: usize,
        context: &'static str,
    ) -> ParseResult<&'a [u8]> {
        if self.pos + len > self.data.len() {
            return Err(Error::UnexpectedEof { context });
        }
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }

    /// Reads a LEB128-encoded `u32`.
    pub(super) fn read_u32(&mut self, context: &'static str) -> ParseResult<u32> {
        let mut result: u32 = 0;
        let mut shift = 0;
        for _ in 0..5 {
            let byte = self.read_u8(context)?;
            result |= u32::from(byte & 0x7F) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
        }
        Err(Error::Leb128Overflow { context })
    }

    /// Reads a LEB128-encoded `i32`.
    #[allow(dead_code)]
    pub(super) fn read_i32(&mut self, context: &'static str) -> ParseResult<i32> {
        let mut result: i32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8(context)?;
            result |= i32::from(byte & 0x7F) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                if shift < 32 && (byte & 0x40) != 0 {
                    result |= !0 << shift;
                }
                return Ok(result);
            }
            if shift >= 35 {
                return Err(Error::Leb128Overflow { context });
            }
        }
    }

    /// Reads a LEB128-encoded `i64`.
    #[allow(dead_code)]
    pub(super) fn read_i64(&mut self, context: &'static str) -> ParseResult<i64> {
        let mut result: i64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_u8(context)?;
            result |= i64::from(byte & 0x7F) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                if shift < 64 && (byte & 0x40) != 0 {
                    result |= !0 << shift;
                }
                return Ok(result);
            }
            if shift >= 70 {
                return Err(Error::Leb128Overflow { context });
            }
        }
    }

    /// Reads a UTF-8 name (length-prefixed byte string).
    pub(super) fn read_name(&mut self) -> ParseResult<String> {
        let len = self.read_u32("name length")? as usize;
        let bytes = self.read_bytes(len, "name data")?;
        String::from_utf8(bytes.to_vec()).map_err(|_| Error::InvalidUtf8)
    }

    /// Creates a sub-cursor over the next `len` bytes, advancing this cursor past them.
    pub(super) fn sub_cursor(
        &mut self,
        len: usize,
        context: &'static str,
    ) -> ParseResult<Cursor<'a>> {
        let bytes = self.read_bytes(len, context)?;
        Ok(Cursor::new(bytes))
    }

    /// Reads a constant expression, consuming bytes until the `end` opcode (`0x0B`).
    pub(super) fn read_const_expr(&mut self) -> ParseResult<Vec<u8>> {
        let start = self.pos;
        loop {
            let byte = self.read_u8("const expr")?;
            if byte == 0x0B {
                return Ok(self.data[start..self.pos - 1].to_vec());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u8_basic() {
        let mut r = Cursor::new(&[0x42]);
        assert_eq!(r.read_u8("test").unwrap(), 0x42);
        assert!(r.is_empty());
    }

    #[test]
    fn read_u8_eof() {
        let mut r = Cursor::new(&[]);
        assert!(r.read_u8("test").is_err());
    }

    #[test]
    fn read_u32_single_byte() {
        let mut r = Cursor::new(&[0x05]);
        assert_eq!(r.read_u32("test").unwrap(), 5);
    }

    #[test]
    fn read_u32_multi_byte() {
        let mut r = Cursor::new(&[0xe5, 0x8e, 0x26]);
        assert_eq!(r.read_u32("test").unwrap(), 624_485);
    }

    #[test]
    fn read_u32_overflow() {
        let mut r = Cursor::new(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);
        assert!(r.read_u32("test").is_err());
    }

    #[test]
    fn read_i32_positive() {
        let mut r = Cursor::new(&[0x05]);
        assert_eq!(r.read_i32("test").unwrap(), 5);
    }

    #[test]
    fn read_i32_negative() {
        let mut r = Cursor::new(&[0x7b]);
        assert_eq!(r.read_i32("test").unwrap(), -5);
    }

    #[test]
    fn read_i64_positive() {
        let mut r = Cursor::new(&[0x05]);
        assert_eq!(r.read_i64("test").unwrap(), 5);
    }

    #[test]
    fn read_i64_negative() {
        let mut r = Cursor::new(&[0x7b]);
        assert_eq!(r.read_i64("test").unwrap(), -5);
    }

    #[test]
    fn read_name_valid() {
        let mut r = Cursor::new(&[0x03, b'a', b'b', b'c']);
        assert_eq!(r.read_name().unwrap(), "abc");
    }

    #[test]
    fn read_name_invalid_utf8() {
        let mut r = Cursor::new(&[0x02, 0xff, 0xfe]);
        assert!(r.read_name().is_err());
    }

    #[test]
    fn sub_cursor() {
        let mut r = Cursor::new(&[0x01, 0x02, 0x03, 0x04]);
        let mut sub = r.sub_cursor(2, "test").unwrap();
        assert_eq!(sub.read_u8("test").unwrap(), 0x01);
        assert_eq!(sub.read_u8("test").unwrap(), 0x02);
        assert!(sub.is_empty());
        assert_eq!(r.read_u8("test").unwrap(), 0x03);
    }
}
