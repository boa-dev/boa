use byte_slice_cast::AsByteSlice;

use crate::{Encoding, JStrRef};

/// Wrapper for an interned str pointer, required to
/// quickly check using a hash if a string is inside an [`Interner`][`super::Interner`].
/// This can distinguish between `UTF-8` and `UTF-16` strings, meaning we can
/// save space by storing both types of encodings on a [`Vec<u8>`] buffer.
///
/// # Safety
///
/// This struct could cause Undefined Behaviour on:
/// - Use without ensuring the referenced memory is still allocated.
/// - Construction of an [`InternedStr`] from an invalid `*const u8` pointer.
/// - Construction of an [`InternedStr`] from an invalid length.
/// - Construction of an [`InternedStr`] that is `UTF-8` encoded and has invalid
/// `UTF-8` code points.
/// - Construction of an [`InternedStr`] that is `UTF-16` encoded and is not
/// 2-byte aligned. Note that an `UTF-16` encoded string can have unpaired
/// surrogates.
/// - Construction of an [`InternedStr`] from a [`JStrRef`] without checking
/// if the [`JStrRef`] outlives it.
///
/// In general, this should not be used outside of an [`Interner`][`super::Interner`].
#[derive(Debug, Clone)]
pub(super) struct InternedStr {
    ptr: *const u8,
    len: usize,
    encoding: Encoding,
}

impl InternedStr {
    /// Create a new interned string from the given `*const u8` pointer,
    /// length and encoding kind
    ///
    /// # Safety
    ///
    /// Not maintaining the invariants specified on the struct definition
    /// could cause Undefined Behaviour.
    #[inline]
    pub(super) unsafe fn new(ptr: *const u8, len: usize, encoding: Encoding) -> Self {
        Self { ptr, len, encoding }
    }

    /// Returns a shared reference to the underlying string.
    ///
    /// # Safety
    ///
    /// Not maintaining the invariants specified on the struct definition
    /// could cause Undefined Behaviour.
    #[inline]
    pub(super) unsafe fn as_jstr_ref(&self) -> JStrRef<'_> {
        match self.encoding {
            Encoding::Utf8 => {
                // SAFETY:
                // - The caller must ensure the provided pointer and length
                // refer to a valid `[u8]` slice.
                // - The caller must ensure the provided pointer and length
                // refer to a valid `UTF-8` string.
                unsafe {
                    JStrRef::Utf8(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                        self.ptr, self.len,
                    )))
                }
            }
            Encoding::Utf16 => {
                // SAFETY:
                // - The caller must ensure the provided pointer and length
                // refer to a valid `[u16]` slice.
                // - The caller must ensure the provided pointer and length
                // are 2-byte aligned.
                unsafe {
                    let data: &[u8] = std::slice::from_raw_parts(self.ptr, self.len * 2);
                    let data = data.align_to::<u16>();
                    debug_assert!(data.0.is_empty());
                    debug_assert!(data.2.is_empty());
                    JStrRef::Utf16(data.1)
                }
            }
        }
    }
}

impl std::hash::Hash for InternedStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // SAFETY: The caller must verify the invariants
        // specified in the struct definition.
        unsafe {
            self.as_jstr_ref().hash(state);
        }
    }
}

impl Eq for InternedStr {}

impl PartialEq for InternedStr {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: The caller must verify the invariants
        // specified in the struct definition.
        unsafe { self.as_jstr_ref() == other.as_jstr_ref() }
    }
}

impl<'a> From<JStrRef<'a>> for InternedStr {
    fn from(sref: JStrRef<'a>) -> Self {
        match sref {
            JStrRef::Utf8(s) => Self {
                ptr: s.as_ptr(),
                len: s.len(),
                encoding: Encoding::Utf8,
            },
            JStrRef::Utf16(s) => {
                let bytes = s.as_byte_slice();
                Self {
                    ptr: bytes.as_ptr(),
                    len: s.len(),
                    encoding: Encoding::Utf16,
                }
            }
        }
    }
}
