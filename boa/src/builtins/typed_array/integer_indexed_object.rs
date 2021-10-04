//! This module implements the `Integer-Indexed` exotic object.
//!
//! An `Integer-Indexed` exotic object is an exotic object that performs
//! special handling of integer index property keys.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects

use crate::{
    builtins::typed_array::TypedArrayName,
    gc::{empty_trace, Finalize, Trace},
    object::{JsObject, ObjectData},
    Context, JsResult,
};

/// Type of the array content.
#[derive(Debug, Clone, Copy, Finalize, PartialEq)]
pub(crate) enum ContentType {
    Number,
    BigInt,
}

unsafe impl Trace for ContentType {
    // safe because `ContentType` is `Copy`
    empty_trace!();
}

/// <https://tc39.es/ecma262/#integer-indexed-exotic-object>
#[derive(Debug, Clone, Trace, Finalize)]
pub struct IntegerIndexed {
    viewed_array_buffer: Option<JsObject>,
    typed_array_name: TypedArrayName,
    byte_offset: usize,
    byte_length: usize,
    array_length: usize,
}

impl IntegerIndexed {
    pub(crate) fn new(
        viewed_array_buffer: Option<JsObject>,
        typed_array_name: TypedArrayName,
        byte_offset: usize,
        byte_length: usize,
        array_length: usize,
    ) -> Self {
        Self {
            viewed_array_buffer,
            typed_array_name,
            byte_offset,
            byte_length,
            array_length,
        }
    }

    /// `IntegerIndexedObjectCreate ( prototype )`
    ///
    /// Create a new `JsObject from a prototype and a `IntergetIndexedObject`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-integerindexedobjectcreate
    pub(super) fn create(prototype: JsObject, data: Self, context: &Context) -> JsObject {
        // 1. Let internalSlotsList be « [[Prototype]], [[Extensible]], [[ViewedArrayBuffer]], [[TypedArrayName]], [[ContentType]], [[ByteLength]], [[ByteOffset]], [[ArrayLength]] ».
        // 2. Let A be ! MakeBasicObject(internalSlotsList).
        let a = context.construct_object();

        // 3. Set A.[[GetOwnProperty]] as specified in 10.4.5.1.
        // 4. Set A.[[HasProperty]] as specified in 10.4.5.2.
        // 5. Set A.[[DefineOwnProperty]] as specified in 10.4.5.3.
        // 6. Set A.[[Get]] as specified in 10.4.5.4.
        // 7. Set A.[[Set]] as specified in 10.4.5.5.
        // 8. Set A.[[Delete]] as specified in 10.4.5.6.
        // 9. Set A.[[OwnPropertyKeys]] as specified in 10.4.5.7.
        a.borrow_mut().data = ObjectData::integer_indexed(data);

        // 10. Set A.[[Prototype]] to prototype.
        a.set_prototype_instance(prototype.into());

        // 11. Return A.
        a
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
    pub(crate) fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    /// Set the integer indexed object's byte offset.
    pub(crate) fn set_byte_offset(&mut self, byte_offset: usize) {
        self.byte_offset = byte_offset;
    }

    /// Get the integer indexed object's typed array name.
    pub(crate) fn typed_array_name(&self) -> TypedArrayName {
        self.typed_array_name
    }

    /// Get a reference to the integer indexed object's viewed array buffer.
    pub fn viewed_array_buffer(&self) -> Option<&JsObject> {
        self.viewed_array_buffer.as_ref()
    }

    ///(crate) Set the integer indexed object's viewed array buffer.
    pub fn set_viewed_array_buffer(&mut self, viewed_array_buffer: Option<JsObject>) {
        self.viewed_array_buffer = viewed_array_buffer;
    }

    /// Get the integer indexed object's byte length.
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Set the integer indexed object's byte length.
    pub(crate) fn set_byte_length(&mut self, byte_length: usize) {
        self.byte_length = byte_length;
    }

    /// Get the integer indexed object's array length.
    pub fn array_length(&self) -> usize {
        self.array_length
    }

    /// Set the integer indexed object's array length.
    pub(crate) fn set_array_length(&mut self, array_length: usize) {
        self.array_length = array_length;
    }
}

/// A Data Block
///
/// The Data Block specification type is used to describe a distinct and mutable sequence of
/// byte-sized (8 bit) numeric values. A byte value is an integer value in the range `0` through
/// `255`, inclusive. A Data Block value is created with a fixed number of bytes that each have
/// the initial value `0`.
///
/// For more information, check the [spec][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-data-blocks
#[derive(Debug, Clone, Default, Trace, Finalize)]
pub struct DataBlock {
    inner: Vec<u8>,
}

impl DataBlock {
    /// `CreateByteDataBlock ( size )` abstract operation.
    ///
    /// The abstract operation `CreateByteDataBlock` takes argument `size` (a non-negative
    /// integer). For more information, check the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createbytedatablock
    pub fn create_byte_data_block(size: usize) -> JsResult<Self> {
        // 1. Let db be a new Data Block value consisting of size bytes. If it is impossible to
        //    create such a Data Block, throw a RangeError exception.
        // 2. Set all of the bytes of db to 0.
        // 3. Return db.
        // TODO: waiting on <https://github.com/rust-lang/rust/issues/48043> for having fallible
        // allocation.
        Ok(Self {
            inner: vec![0u8; size],
        })
    }
}
