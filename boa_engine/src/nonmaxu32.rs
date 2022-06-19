//! This module implements `NonMaxU32`
//! which is known not to equal `u32::MAX`.
//!
//! This would be useful for integers like `https://tc39.es/ecma262/#array-index`.

use boa_gc::{unsafe_empty_trace, Finalize, Trace};
use std::fmt;
use std::ops::{BitOr, BitOrAssign};

/// An integer that is known not to equal `u32::MAX`.
///
/// This enables some memory layout optimization.
/// For example, `Option<NonMaxU32>` is the same size as `u32`:
///
/// ```rust
/// use std::mem::size_of;
/// use boa_engine::nonmaxu32::NonMaxU32;
/// assert_eq!(size_of::<Option<NonMaxU32>>(), size_of::<u32>());
/// ```
#[derive(Copy, Clone, Eq, Finalize, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct NonMaxU32(u32);

// Safety: `NonMaxU32` does not contain any objects which needs to be traced,
// so this is safe.
unsafe impl Trace for NonMaxU32 {
    unsafe_empty_trace!();
}

/// An error type returned when a checked integral type conversion fails (mimics [`std::num::TryFromIntError`])
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryFromIntError(());

impl fmt::Display for TryFromIntError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        "out of range integral type conversion attempted".fmt(fmt)
    }
}

impl From<core::num::TryFromIntError> for TryFromIntError {
    fn from(_: core::num::TryFromIntError) -> Self {
        Self(())
    }
}

impl From<core::convert::Infallible> for TryFromIntError {
    fn from(never: core::convert::Infallible) -> Self {
        match never {}
    }
}

/// An error type returned when an integer cannot be parsed (mimics [`std::num::ParseIntError`])
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParseIntError(());

impl fmt::Display for ParseIntError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        "unable to parse integer".fmt(fmt)
    }
}

impl From<core::num::ParseIntError> for ParseIntError {
    fn from(_: core::num::ParseIntError) -> Self {
        Self(())
    }
}

impl NonMaxU32 {
    /// Creates a non-u32-max without checking the value.
    ///
    /// # Safety
    ///
    /// The value must not be `u32::MAX`.
    #[inline]
    pub const unsafe fn new_unchecked(n: u32) -> Self {
        // SAFETY: this is guaranteed to be safe by the caller.
        Self(n)
    }

    /// Creates a non-u32-max if the given value is not `u32::MAX`.
    #[inline]
    pub const fn new(n: u32) -> Option<Self> {
        if n == u32::MAX {
            None
        } else {
            // SAFETY: we just checked that there's no `u32::MAX`
            Some(Self(n))
        }
    }

    /// Returns the value as a primitive type.
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for NonMaxU32 {
    type Error = TryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(TryFromIntError(()))
    }
}

impl From<NonMaxU32> for u32 {
    /// Converts a `NonMaxU32` into an `u32`
    fn from(nonzero: NonMaxU32) -> Self {
        nonzero.0
    }
}

impl core::str::FromStr for NonMaxU32 {
    type Err = ParseIntError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(u32::from_str(value)?).ok_or(ParseIntError(()))
    }
}

impl BitOr for NonMaxU32 {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        // Safety: since `self` and `rhs` are both nonzero, the
        // result of the bitwise-or will be nonzero.
        unsafe { NonMaxU32::new_unchecked(self.get() | rhs.get()) }
    }
}

impl BitOr<u32> for NonMaxU32 {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: u32) -> Self::Output {
        // Safety: since `self` is nonzero, the result of the
        // bitwise-or will be nonzero regardless of the value of
        // `rhs`.
        unsafe { NonMaxU32::new_unchecked(self.get() | rhs) }
    }
}

impl BitOr<NonMaxU32> for u32 {
    type Output = NonMaxU32;

    #[inline]
    fn bitor(self, rhs: NonMaxU32) -> Self::Output {
        // Safety: since `rhs` is nonzero, the result of the
        // bitwise-or will be nonzero regardless of the value of
        // `self`.
        unsafe { NonMaxU32::new_unchecked(self | rhs.get()) }
    }
}

impl BitOrAssign for NonMaxU32 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitOrAssign<u32> for NonMaxU32 {
    #[inline]
    fn bitor_assign(&mut self, rhs: u32) {
        *self = *self | rhs;
    }
}

impl fmt::Debug for NonMaxU32 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl fmt::Display for NonMaxU32 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}
