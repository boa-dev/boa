//! A dedicated parser for `JSON.parse`.
//!
//! It reads the text's code units directly (Latin-1 or UTF-16, without
//! converting the string) and builds the resulting values in one pass,
//! instead of validating the text, parsing it as a script, compiling and
//! evaluating it. Nesting is handled with an explicit stack, so deeply
//! nested input cannot overflow the native stack.
//!
//! When a reviver is given, it also records the source text of every
//! primitive value (the `JsonNode` tree), for the reviver's `context.source`.

use super::JsonNode;
use rustc_hash::FxHashMap;

use crate::{
    Context, JsNativeError, JsResult, JsString, JsValue,
    builtins::{Array, OrdinaryObject},
    object::JsObject,
    property::{CompletePropertyDescriptor, PropertyKey},
};

/// A code unit of the text: Latin-1 (`u8`) or UTF-16 (`u16`).
pub(super) trait Unit: Copy + Eq + std::hash::Hash {
    fn unit(self) -> u16;
    fn string(units: &[Self]) -> JsString;
    /// Returns `units` as bytes, borrowed or copied into `buffer`, if they fit;
    /// every unit must be ASCII.
    fn ascii_bytes<'a>(units: &'a [Self], buffer: &'a mut [u8]) -> Option<&'a [u8]>;
}

impl Unit for u8 {
    #[inline]
    fn unit(self) -> u16 {
        u16::from(self)
    }

    #[inline]
    fn ascii_bytes<'a>(units: &'a [Self], _buffer: &'a mut [u8]) -> Option<&'a [u8]> {
        Some(units)
    }

    fn string(units: &[Self]) -> JsString {
        JsString::from(crate::string::JsStr::latin1(units))
    }
}

impl Unit for u16 {
    #[inline]
    fn unit(self) -> u16 {
        self
    }

    #[inline]
    fn ascii_bytes<'a>(units: &'a [Self], buffer: &'a mut [u8]) -> Option<&'a [u8]> {
        let buffer = buffer.get_mut(..units.len())?;
        for (byte, unit) in buffer.iter_mut().zip(units) {
            *byte = *unit as u8;
        }
        Some(buffer)
    }

    fn string(units: &[Self]) -> JsString {
        JsString::from(units)
    }
}

/// A container being filled.
enum Frame {
    Array {
        items: Vec<JsValue>,
        nodes: Vec<JsonNode>,
    },
    Object {
        object: JsObject,
        /// The key of the value being parsed.
        key: PropertyKey,
        entries: Vec<(String, JsonNode)>,
    },
}

struct Parser<'a, T: Unit> {
    text: &'a [T],
    pos: usize,
    track_source: bool,
    /// Keys without escapes, by their text: objects of the same kind repeat
    /// them, and each is allocated once.
    keys: FxHashMap<&'a [T], PropertyKey>,
}

const fn is_whitespace(c: u16) -> bool {
    matches!(c, 0x09 | 0x0A | 0x0D | 0x20)
}

impl<'a, T: Unit> Parser<'a, T> {
    #[inline]
    fn peek(&self) -> Option<u16> {
        self.text.get(self.pos).map(|c| c.unit())
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !is_whitespace(c) {
                break;
            }
            self.pos += 1;
        }
    }

    /// A `SyntaxError` at the current position, worded as before
    /// (`<message> at line <l> column <c>`).
    fn error(&self, message: &str) -> crate::JsError {
        let end = self.pos.min(self.text.len());
        let mut line = 1;
        let mut column = 0;
        for c in &self.text[..end] {
            if c.unit() == u16::from(b'\n') {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        // The column of the offending character (1-based), or of the end.
        if self.pos < self.text.len() {
            column += 1;
        }
        JsNativeError::syntax()
            .with_message(format!("{message} at line {line} column {column}"))
            .into()
    }

    fn eof_or(&self, eof: &str, other: &str) -> crate::JsError {
        if self.pos >= self.text.len() {
            self.error(eof)
        } else {
            self.error(other)
        }
    }

    fn source(&self, start: usize) -> String {
        let units: Vec<u16> = self.text[start..self.pos]
            .iter()
            .map(|c| c.unit())
            .collect();
        String::from_utf16_lossy(&units)
    }

    /// Parse a string starting at the opening quote.
    fn string(&mut self) -> JsResult<JsString> {
        self.pos += 1;
        let start = self.pos;
        // Fast path: no escape sequence.
        loop {
            match self.peek() {
                None => return Err(self.error("EOF while parsing a string")),
                Some(0x22) => {
                    let s = T::string(&self.text[start..self.pos]);
                    self.pos += 1;
                    return Ok(s);
                }
                Some(0x5C) => break,
                Some(c) if c < 0x20 => {
                    return Err(self.error(
                        "control character (\\u0000-\\u001F) found while parsing a string",
                    ));
                }
                Some(_) => self.pos += 1,
            }
        }
        let mut out: Vec<u16> = self.text[start..self.pos]
            .iter()
            .map(|c| c.unit())
            .collect();
        loop {
            match self.peek() {
                None => return Err(self.error("EOF while parsing a string")),
                Some(0x22) => {
                    self.pos += 1;
                    return Ok(JsString::from(&out[..]));
                }
                Some(0x5C) => {
                    self.pos += 1;
                    let Some(escape) = self.peek() else {
                        return Err(self.error("EOF while parsing a string"));
                    };
                    let decoded = match escape {
                        0x22 => 0x22,
                        0x5C => 0x5C,
                        0x2F => 0x2F,
                        0x62 => 0x08,
                        0x66 => 0x0C,
                        0x6E => 0x0A,
                        0x72 => 0x0D,
                        0x74 => 0x09,
                        0x75 => {
                            let mut value = 0u16;
                            for i in 1..=4 {
                                let digit = self
                                    .text
                                    .get(self.pos + i)
                                    .and_then(|c| char::from_u32(u32::from(c.unit())))
                                    .and_then(|c| c.to_digit(16));
                                let Some(d) = digit else {
                                    self.pos += i;
                                    return Err(
                                        self.eof_or("EOF while parsing a string", "invalid escape")
                                    );
                                };
                                value = value * 16 + d as u16;
                            }
                            self.pos += 4;
                            value
                        }
                        _ => return Err(self.error("invalid escape")),
                    };
                    out.push(decoded);
                    self.pos += 1;
                }
                Some(c) if c < 0x20 => {
                    return Err(self.error(
                        "control character (\\u0000-\\u001F) found while parsing a string",
                    ));
                }
                Some(c) => {
                    out.push(c);
                    self.pos += 1;
                }
            }
        }
    }

    /// An object key: from the cache when it has no escapes.
    fn cached_key(&mut self) -> JsResult<PropertyKey> {
        let start = self.pos + 1;
        let text: &'a [T] = self.text;
        let mut end = start;
        while let Some(c) = text.get(end).map(|c| c.unit()) {
            if c == 0x22 {
                let raw = &text[start..end];
                if let Some(key) = self.keys.get(raw) {
                    self.pos = end + 1;
                    return Ok(key.clone());
                }
                let key = PropertyKey::from(self.string()?);
                self.keys.insert(raw, key.clone());
                return Ok(key);
            }
            if c == 0x5C || c < 0x20 {
                break;
            }
            end += 1;
        }
        Ok(PropertyKey::from(self.string()?))
    }

    fn digits(&mut self) -> usize {
        let start = self.pos;
        while matches!(self.peek(), Some(0x30..=0x39)) {
            self.pos += 1;
        }
        self.pos - start
    }

    /// Parse a number (JSON's grammar, ECMA-404).
    fn number(&mut self) -> JsResult<JsValue> {
        let start = self.pos;
        let negative = self.peek() == Some(0x2D);
        if negative {
            self.pos += 1;
        }
        match self.peek() {
            Some(0x30) => self.pos += 1,
            Some(0x31..=0x39) => {
                self.digits();
            }
            _ => return Err(self.eof_or("EOF while parsing a value", "invalid number")),
        }
        let int_end = self.pos;
        let mut integer = true;
        if self.peek() == Some(0x2E) {
            integer = false;
            self.pos += 1;
            if self.digits() == 0 {
                return Err(self.eof_or("EOF while parsing a value", "invalid number"));
            }
        }
        if matches!(self.peek(), Some(0x65 | 0x45)) {
            integer = false;
            self.pos += 1;
            if matches!(self.peek(), Some(0x2B | 0x2D)) {
                self.pos += 1;
            }
            if self.digits() == 0 {
                return Err(self.eof_or("EOF while parsing a value", "invalid number"));
            }
        }
        // Small integers: no float parsing (and an int32 value, as before).
        let digits_start = start + usize::from(negative);
        if integer && int_end - digits_start <= 9 {
            let mut value: i32 = 0;
            for c in &self.text[digits_start..int_end] {
                value = value * 10 + i32::from(c.unit() - 0x30);
            }
            if negative {
                if value == 0 {
                    return Ok(JsValue::from(-0.0));
                }
                value = -value;
            }
            return Ok(JsValue::from(value));
        }
        // The number is ASCII (checked above): parse its bytes without allocating.
        let units = &self.text[start..self.pos];
        let mut buffer = [0u8; 64];
        let parsed = if let Some(bytes) = T::ascii_bytes(units, &mut buffer) {
            fast_float2::parse::<f64, _>(bytes)
        } else {
            let bytes: Vec<u8> = units.iter().map(|c| c.unit() as u8).collect();
            fast_float2::parse::<f64, _>(&bytes)
        };
        let value = parsed.map_err(|_| self.error("invalid number"))?;
        Ok(JsValue::from(value))
    }

    fn literal(&mut self, word: &[u8], value: JsValue) -> JsResult<JsValue> {
        for (i, expected) in word.iter().enumerate() {
            match self.text.get(self.pos + i) {
                Some(c) if c.unit() == u16::from(*expected) => {}
                Some(_) => {
                    self.pos += i;
                    return Err(self.error("expected ident"));
                }
                None => {
                    self.pos += i;
                    return Err(self.error("EOF while parsing a value"));
                }
            }
        }
        self.pos += word.len();
        Ok(value)
    }

    /// Parse an object key and its `:`, leaving the position on the value.
    fn key(&mut self) -> JsResult<PropertyKey> {
        match self.peek() {
            Some(0x22) => {}
            None => return Err(self.error("EOF while parsing an object")),
            Some(_) => return Err(self.error("key must be a string")),
        }
        let key = self.cached_key()?;
        self.skip_whitespace();
        match self.peek() {
            Some(0x3A) => self.pos += 1,
            None => return Err(self.error("EOF while parsing an object")),
            Some(_) => return Err(self.error("expected `:`")),
        }
        self.skip_whitespace();
        Ok(key)
    }

    fn parse(&mut self, context: &mut Context) -> JsResult<(JsValue, Option<JsonNode>)> {
        let mut stack: Vec<Frame> = Vec::new();
        self.skip_whitespace();
        'value: loop {
            // Parse one value, or open a container and go parse its first member.
            let start = self.pos;
            let (mut value, mut node) = match self.peek() {
                Some(0x7B) => {
                    self.pos += 1;
                    self.skip_whitespace();
                    // From the ordinary-object template, as object literals
                    // are: objects with the same keys share their shape.
                    let object = context
                        .intrinsics()
                        .templates()
                        .ordinary_object()
                        .create(OrdinaryObject, Vec::default());
                    if self.peek() == Some(0x7D) {
                        self.pos += 1;
                        (
                            object.into(),
                            self.track_source.then(|| JsonNode::Object(Vec::new())),
                        )
                    } else {
                        let key = self.key()?;
                        stack.push(Frame::Object {
                            object,
                            key,
                            entries: Vec::new(),
                        });
                        continue 'value;
                    }
                }
                Some(0x5B) => {
                    self.pos += 1;
                    self.skip_whitespace();
                    if self.peek() == Some(0x5D) {
                        self.pos += 1;
                        let array = Array::create_array_from_list([], context);
                        (
                            array.into(),
                            self.track_source.then(|| JsonNode::Array(Vec::new())),
                        )
                    } else {
                        stack.push(Frame::Array {
                            items: Vec::new(),
                            nodes: Vec::new(),
                        });
                        continue 'value;
                    }
                }
                Some(0x22) => {
                    let s = self.string()?;
                    (s.into(), None)
                }
                Some(0x74) => (self.literal(b"true", JsValue::from(true))?, None),
                Some(0x66) => (self.literal(b"false", JsValue::from(false))?, None),
                Some(0x6E) => (self.literal(b"null", JsValue::null())?, None),
                Some(0x2D | 0x30..=0x39) => (self.number()?, None),
                None => {
                    return Err(match stack.last() {
                        None => self.error("EOF while parsing a value"),
                        Some(Frame::Array { .. }) => self.error("EOF while parsing a list"),
                        Some(Frame::Object { .. }) => self.error("EOF while parsing an object"),
                    });
                }
                Some(_) => return Err(self.error("expected value")),
            };
            if self.track_source && node.is_none() {
                node = Some(JsonNode::Primitive(self.source(start)));
            }

            // Hand the value to its container, closing every container that ends here.
            loop {
                self.skip_whitespace();
                match stack.last_mut() {
                    None => {
                        if self.pos < self.text.len() {
                            return Err(self.error("trailing characters"));
                        }
                        return Ok((value, node));
                    }
                    Some(Frame::Array { items, nodes }) => {
                        items.push(value);
                        if let Some(n) = node {
                            nodes.push(n);
                        }
                        match self.peek() {
                            Some(0x2C) => {
                                self.pos += 1;
                                self.skip_whitespace();
                                if self.peek() == Some(0x5D) {
                                    return Err(self.error("trailing comma"));
                                }
                                continue 'value;
                            }
                            Some(0x5D) => {
                                self.pos += 1;
                                let Some(Frame::Array { items, nodes }) = stack.pop() else {
                                    unreachable!()
                                };
                                value = Array::create_array_from_list(items, context).into();
                                node = self.track_source.then_some(JsonNode::Array(nodes));
                            }
                            None => return Err(self.error("EOF while parsing a list")),
                            Some(_) => return Err(self.error("expected `,` or `]`")),
                        }
                    }
                    Some(Frame::Object {
                        object,
                        key,
                        entries,
                    }) => {
                        if let Some(n) = node {
                            entries.push((key.to_string(), n));
                        }
                        // CreateDataProperty on a fresh, extensible ordinary
                        // object cannot fail nor run code: insert the data
                        // property directly. `"__proto__"` is an own property,
                        // and a repeated key keeps its first position.
                        object.insert_property(
                            key.clone(),
                            CompletePropertyDescriptor::Data {
                                value,
                                writable: true,
                                enumerable: true,
                                configurable: true,
                            },
                        );
                        match self.peek() {
                            Some(0x2C) => {
                                self.pos += 1;
                                self.skip_whitespace();
                                if self.peek() == Some(0x7D) {
                                    return Err(self.error("trailing comma"));
                                }
                                let next = self.key()?;
                                if let Some(Frame::Object { key, .. }) = stack.last_mut() {
                                    *key = next;
                                }
                                continue 'value;
                            }
                            Some(0x7D) => {
                                self.pos += 1;
                                let Some(Frame::Object {
                                    object, entries, ..
                                }) = stack.pop()
                                else {
                                    unreachable!()
                                };
                                value = object.into();
                                node = self.track_source.then_some(JsonNode::Object(entries));
                            }
                            None => return Err(self.error("EOF while parsing an object")),
                            Some(_) => return Err(self.error("expected `,` or `}`")),
                        }
                    }
                }
            }
        }
    }
}

/// Parse `text` as a JSON text. With `track_source`, also return the source
/// tree used by the reviver.
pub(super) fn parse<T: Unit>(
    text: &[T],
    track_source: bool,
    context: &mut Context,
) -> JsResult<(JsValue, Option<JsonNode>)> {
    Parser {
        text,
        pos: 0,
        track_source,
        keys: FxHashMap::default(),
    }
    .parse(context)
}
