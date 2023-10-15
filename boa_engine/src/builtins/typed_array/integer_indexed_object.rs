//! This module implements the `Integer-Indexed` exotic object.

use crate::object::JsObject;
use boa_gc::{Finalize, Trace};

use super::TypedArrayKind;

/// An `Integer-Indexed` exotic object is an exotic object that performs
/// special handling of integer index property keys.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects
#[derive(Debug, Clone, Trace, Finalize)]
pub struct IntegerIndexed {
    viewed_array_buffer: JsObject,
    #[unsafe_ignore_trace]
    kind: TypedArrayKind,
    byte_offset: u64,
    byte_length: u64,
    array_length: u64,
}

impl IntegerIndexed {
    pub(crate) const fn new(
        viewed_array_buffer: JsObject,
        kind: TypedArrayKind,
        byte_offset: u64,
        byte_length: u64,
        array_length: u64,
    ) -> Self {
        Self {
            viewed_array_buffer,
            kind,
            byte_offset,
            byte_length,
            array_length,
        }
    }

    /// Abstract operation `IsDetachedBuffer ( arrayBuffer )`.
    ///
    /// Check if `[[ArrayBufferData]]` is null.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdetachedbuffer
    pub(crate) fn is_detached(&self) -> bool {
        self.viewed_array_buffer
            .borrow()
            .as_buffer()
            .expect("Typed array must have internal array buffer object")
            .is_detached()
    }

    /// Get the integer indexed object's byte offset.
    #[must_use]
    pub const fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    /// Get the integer indexed object's typed array kind.
    pub(crate) const fn kind(&self) -> TypedArrayKind {
        self.kind
    }

    /// Get a reference to the integer indexed object's viewed array buffer.
    #[must_use]
    pub const fn viewed_array_buffer(&self) -> &JsObject {
        &self.viewed_array_buffer
    }

    /// Get the integer indexed object's byte length.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    /// Get the integer indexed object's array length.
    #[must_use]
    pub const fn array_length(&self) -> u64 {
        self.array_length
    }
}
