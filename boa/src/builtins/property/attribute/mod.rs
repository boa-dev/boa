//! This module implements the `Attribute` struct which contains the attibutes for `Property`.

use bitflags::bitflags;
use gc::{unsafe_empty_trace, Finalize, Trace};

#[cfg(test)]
mod tests;

bitflags! {
    /// This struct constains the property flags as describen in the [`ECMAScript specification`][spec].
    ///
    /// It contains the following flags:
    ///  - `[[Writable]]` (`WRITABLE`) - If `false`, attempts by ECMAScript code to change the property's `[[Value]]` attribute using `[[Set]]` will not succeed.
    ///  - `[[Enumerable]]` (`ENUMERABLE`) - If the property will be enumerated by a for-in enumeration.
    ///  - `[[Configurable]]` (`CONFIGURABLE`) - If `false`, attempts to delete the property, change the property to be an `accessor property`, or change its attributes (other than `[[Value]]`, or changing `[[Writable]]` to `false`) will fail.
    ///
    /// Additionaly there are flags for when the flags are defined.
    #[derive(Finalize)]
    pub struct Attribute: u8 {
        /// None of the flags are present.
        const NONE = 0b0000_0000;

        /// The `Writable` attribute decides whether the value associated with the property can be changed or not, from its initial value.
        const WRITABLE = 0b0000_0011;

        /// The property is not writable.
        const READONLY = 0b0000_0010;

        /// Is the `Writable` flag defined.
        const HAS_WRITABLE = 0b0000_0010;

        /// If the property can be enumerated by a `for-in` loop.
        const ENUMERABLE = 0b0000_1100;

        /// The property can not be enumerated in a `for-in`.
        const NON_ENUMERABLE = 0b0000_1000;

        /// Is the `Enumerable` flag defined.
        const HAS_ENUMERABLE = 0b0000_1000;

        /// If the property descriptor can be changed later.
        const CONFIGURABLE = 0b0011_0000;

        /// The property descriptor cannot be changed.
        const PERMANENT = 0b0010_0000;

        /// Is the `Configurable` flag defined.
        const HAS_CONFIGURABLE = 0b0010_0000;
    }
}

// We implement `Trace` manualy rather that wih derive, beacuse `rust-gc`,
// derive `Trace` does not allow `Copy` and `Trace` to be both implemented.
//
// SAFETY: The `Attribute` struct only contains an `u8`
// and therefore it should be safe to implement an empty trace.
unsafe impl Trace for Attribute {
    unsafe_empty_trace!();
}

impl Attribute {
    /// Clear all flags.
    #[inline]
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    /// Is the `writable` flag defined.
    #[inline]
    pub fn has_writable(self) -> bool {
        self.contains(Self::HAS_WRITABLE)
    }

    /// Sets the `writable` flag.
    #[inline]
    pub fn set_writable(&mut self, value: bool) {
        if value {
            *self |= Self::WRITABLE;
        } else {
            *self |= *self & !Self::WRITABLE;
        }
    }

    /// Gets the `writable` flag.
    #[inline]
    pub fn writable(self) -> bool {
        debug_assert!(self.has_writable());
        self.contains(Self::WRITABLE)
    }

    /// Is the `enumerable` flag defined.
    #[inline]
    pub fn has_enumerable(self) -> bool {
        self.contains(Self::HAS_ENUMERABLE)
    }

    /// Sets the `enumerable` flag.
    #[inline]
    pub fn set_enumerable(&mut self, value: bool) {
        if value {
            *self |= Self::ENUMERABLE;
        } else {
            *self |= *self & !Self::ENUMERABLE;
        }
    }

    /// Gets the `enumerable` flag.
    #[inline]
    pub fn enumerable(self) -> bool {
        debug_assert!(self.has_enumerable());
        self.contains(Self::ENUMERABLE)
    }

    /// Is the `configurable` flag defined.
    #[inline]
    pub fn has_configurable(self) -> bool {
        self.contains(Self::HAS_CONFIGURABLE)
    }

    /// Sets the `configurable` flag.
    #[inline]
    pub fn set_configurable(&mut self, value: bool) {
        if value {
            *self |= Self::CONFIGURABLE;
        } else {
            *self |= *self & !Self::CONFIGURABLE;
        }
    }

    /// Gets the `configurable` flag.
    #[inline]
    pub fn configurable(self) -> bool {
        debug_assert!(self.has_configurable());
        self.contains(Self::CONFIGURABLE)
    }
}

impl Default for Attribute {
    /// Returns the default flags according to the [ECMAScript specification][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#table-default-attribute-values
    fn default() -> Self {
        Self::READONLY | Self::NON_ENUMERABLE | Self::PERMANENT
    }
}
