//! This module implements the `Integer-Indexed` exotic object.
//!
//! An `Integer-Indexed` exotic object is an exotic object that performs
//! special handling of integer index property keys.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects

use crate::{builtins::typed_array::TypedArrayKind, object::JsObject};
use boa_gc::{Finalize, Trace};

/// Type of the array content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ContentType {
    Number,
    BigInt,
}

/// <https://tc39.es/ecma262/#integer-indexed-exotic-object>
#[derive(Debug, Clone, Trace, Finalize)]
pub struct IntegerIndexed {
    viewed_array_buffer: Option<JsObject>,
    #[unsafe_ignore_trace]
    typed_array_name: TypedArrayKind,
    byte_offset: u64,
    byte_length: u64,
    array_length: u64,
}

impl IntegerIndexed {
    pub(crate) const fn new(
        viewed_array_buffer: Option<JsObject>,
        typed_array_name: TypedArrayKind,
        byte_offset: u64,
        byte_length: u64,
        array_length: u64,
    ) -> Self {
        Self {
            viewed_array_buffer,
            typed_array_name,
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
        if let Some(obj) = &self.viewed_array_buffer {
            obj.borrow()
                .as_array_buffer()
                .expect("Typed array must have internal array buffer object")
                .is_detached_buffer()
        } else {
            false
        }
    }

    /// Get the integer indexed object's byte offset.
    pub(crate) const fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    /// Set the integer indexed object's byte offset.
    pub(crate) fn set_byte_offset(&mut self, byte_offset: u64) {
        self.byte_offset = byte_offset;
    }

    /// Get the integer indexed object's typed array name.
    pub(crate) const fn typed_array_name(&self) -> TypedArrayKind {
        self.typed_array_name
    }

    /// Get a reference to the integer indexed object's viewed array buffer.
    pub const fn viewed_array_buffer(&self) -> Option<&JsObject> {
        self.viewed_array_buffer.as_ref()
    }

    ///(crate) Set the integer indexed object's viewed array buffer.
    pub fn set_viewed_array_buffer(&mut self, viewed_array_buffer: Option<JsObject>) {
        self.viewed_array_buffer = viewed_array_buffer;
    }

    /// Get the integer indexed object's byte length.
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    /// Set the integer indexed object's byte length.
    pub(crate) fn set_byte_length(&mut self, byte_length: u64) {
        self.byte_length = byte_length;
    }

    /// Get the integer indexed object's array length.
    pub const fn array_length(&self) -> u64 {
        self.array_length
    }

    /// Set the integer indexed object's array length.
    pub(crate) fn set_array_length(&mut self, array_length: u64) {
        self.array_length = array_length;
    }
}
