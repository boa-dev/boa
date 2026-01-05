//! Module containing string types public and crate-specific.
use crate::vtable::SequenceString;
use crate::{JsStr, JsStringKind};
use std::alloc::Layout;

mod sealed {
    /// Seal to prevent others from implementing their own string types.
    pub trait Sealed {}
}

/// Internal trait for crate-specific usage. Contains implementation details
/// that should not leak through the API.
#[allow(private_interfaces)]
pub(crate) trait InternalStringType: StringType {
    /// The offset to the data field in the sequence string struct.
    const DATA_OFFSET: usize;

    /// The kind of string produced by this string type.
    const KIND: JsStringKind;

    /// Create the base layout for the sequence string header.
    fn base_layout() -> Layout;

    /// Construct a [`JsStr`] from a slice of characters.
    fn str_ctor(slice: &[Self::Byte]) -> JsStr<'_>;
}

/// Trait that maps the data type to the appropriate internal types and constants.
pub trait StringType: sealed::Sealed {
    /// The unit of a character for this type of string. For example, UTF-16 should
    /// have a 16-bits size, while ASCII should have 8 bits.
    type Byte: Copy + Eq + 'static;
}

// It is good defensive programming to have [`Latin1`] `!Copy`, as it should
// not be used as a value anyway.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum Latin1 {}

impl sealed::Sealed for Latin1 {}
impl StringType for Latin1 {
    type Byte = u8;
}

impl InternalStringType for Latin1 {
    const DATA_OFFSET: usize = size_of::<SequenceString<Self>>();
    const KIND: JsStringKind = JsStringKind::Latin1Sequence;

    fn base_layout() -> Layout {
        Layout::new::<SequenceString<Self>>()
    }

    fn str_ctor(slice: &[Self::Byte]) -> JsStr<'_> {
        JsStr::latin1(slice)
    }
}

// It is good defensive programming to have [`Utf16`] `!Copy`, as it should
// not be used as a value anyway.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum Utf16 {}

impl sealed::Sealed for Utf16 {}
impl StringType for Utf16 {
    type Byte = u16;
}

impl InternalStringType for Utf16 {
    const DATA_OFFSET: usize = size_of::<SequenceString<Self>>();
    const KIND: JsStringKind = JsStringKind::Utf16Sequence;

    fn base_layout() -> Layout {
        Layout::new::<SequenceString<Self>>()
    }

    fn str_ctor(slice: &[Self::Byte]) -> JsStr<'_> {
        JsStr::utf16(slice)
    }
}
