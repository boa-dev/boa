//! This module implements the Property Descriptor.
//!
//! The Property Descriptor type is used to explain the manipulation and reification of Object property attributes.
//! Values of the Property Descriptor type are Records. Each field's name is an attribute name
//! and its value is a corresponding attribute value as specified in [6.1.7.1][section].
//! In addition, any field may be present or absent.
//! The schema name used within this specification to tag literal descriptions of Property Descriptor records is “PropertyDescriptor”.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
//! [section]: https://tc39.es/ecma262/#sec-property-attributes

use crate::{
    gc::{Finalize, Trace},
    object::GcObject,
    value::{RcString, RcSymbol, Value},
};
use std::{convert::TryFrom, fmt};

mod attribute;
pub use attribute::Attribute;

#[derive(Debug, Clone)]
pub struct DataDescriptor {
    value: Value,
    attribute: Attribute,
}

impl DataDescriptor {
    pub fn new<V>(value: V, attribute: Attribute) -> Self
    where
        V: Into<Value>,
    {
        Self {
            value: value.into(),
            attribute,
        }
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    pub fn attributes(&self) -> Attribute {
        self.attribute
    }

    #[inline]
    pub fn configurable(&self) -> bool {
        self.attribute.configurable()
    }

    #[inline]
    pub fn set_configurable(&mut self, configurable: bool) {
        self.attribute.set_configurable(configurable)
    }

    /// Set enumerable
    #[inline]
    pub fn enumerable(&self) -> bool {
        self.attribute.enumerable()
    }

    #[inline]
    pub fn set_enumerable(&mut self, enumerable: bool) {
        self.attribute.set_enumerable(enumerable)
    }

    #[inline]
    pub fn writable(&self) -> bool {
        self.attribute.writable()
    }

    #[inline]
    pub fn set_writable(&mut self, writable: bool) {
        self.attribute.set_writable(writable)
    }
}

impl From<DataDescriptor> for PropertyDescriptor {
    fn from(value: DataDescriptor) -> Self {
        Self {
            attribute: value.attributes(),
            value: Some(value.value()),
            get: None,
            set: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccessorDescriptor {
    /// The function serving as getter
    get: Option<GcObject>,
    /// The function serving as setter
    set: Option<GcObject>,
    attribute: Attribute,
}

impl AccessorDescriptor {
    pub fn new(get: Option<GcObject>, set: Option<GcObject>, mut attribute: Attribute) -> Self {
        // Accessors can not have writable attribute.
        attribute.remove(Attribute::WRITABLE);
        Self {
            get,
            set,
            attribute,
        }
    }

    pub fn get(&self) -> Option<GcObject> {
        self.get.clone()
    }

    pub fn set(&self) -> Option<GcObject> {
        self.get.clone()
    }

    pub fn attributes(&self) -> Attribute {
        self.attribute
    }

    #[inline]
    pub fn configurable(&self) -> bool {
        self.attribute.configurable()
    }

    #[inline]
    pub fn set_configurable(&mut self, configurable: bool) {
        self.attribute.set_configurable(configurable)
    }

    /// Set enumerable
    #[inline]
    pub fn enumerable(&self) -> bool {
        self.attribute.enumerable()
    }

    #[inline]
    pub fn set_enumerable(&mut self, enumerable: bool) {
        self.attribute.set_enumerable(enumerable)
    }
}

impl From<AccessorDescriptor> for PropertyDescriptor {
    fn from(value: AccessorDescriptor) -> Self {
        Self {
            attribute: value.attributes(),
            get: value.get().map(Into::into),
            set: value.get().map(Into::into),
            value: None,
        }
    }
}

/// This represents a Javascript Property AKA The Property Descriptor.
///
/// Property descriptors present in objects come in two main flavors:
///  - data descriptors
///  - accessor descriptors
///
/// A data descriptor is a property that has a value, which may or may not be writable.
/// An accessor descriptor is a property described by a getter-setter pair of functions.
/// A descriptor must be one of these two flavors; it cannot be both.
///
/// Any field in a JavaScript Property may be present or absent.
///
/// More information:
/// - [MDN documentation][mdn]
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-property-descriptor-specification-type
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty
#[derive(Trace, Finalize, Clone, Debug)]
pub struct PropertyDescriptor {
    pub(crate) attribute: Attribute,
    /// The value associated with the property
    pub value: Option<Value>,
    /// The function serving as getter
    pub get: Option<Value>,
    /// The function serving as setter
    pub set: Option<Value>,
}

impl PropertyDescriptor {
    /// Get the
    #[inline]
    pub fn configurable(&self) -> bool {
        self.attribute.configurable()
    }

    #[inline]
    pub fn set_configurable(&mut self, configurable: bool) {
        self.attribute.set_configurable(configurable)
    }

    /// Set enumerable
    #[inline]
    pub fn enumerable(&self) -> bool {
        self.attribute.enumerable()
    }

    /// Set writable
    #[inline]
    pub fn writable(&self) -> bool {
        self.attribute.writable()
    }

    /// An accessor Property Descriptor is one that includes any fields named either [[Get]] or [[Set]].
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isaccessordescriptor
    #[inline]
    pub fn is_accessor_descriptor(&self) -> bool {
        self.get.is_some() || self.set.is_some()
    }

    /// A data Property Descriptor is one that includes any fields named either [[Value]] or [[Writable]].
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdatadescriptor
    #[inline]
    pub fn is_data_descriptor(&self) -> bool {
        self.value.is_some() || self.attribute.writable()
    }

    /// Check if a property is generic descriptor.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isgenericdescriptor
    #[inline]
    pub fn is_generic_descriptor(&self) -> bool {
        !self.is_accessor_descriptor() && !self.is_data_descriptor()
    }
}

/// This abstracts away the need for IsPropertyKey by transforming the PropertyKey
/// values into an enum with both valid types: String and Symbol
///
/// More information:
/// - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ispropertykey
#[derive(Trace, Finalize, Debug, Clone)]
pub enum PropertyKey {
    String(RcString),
    Symbol(RcSymbol),
    Index(u32),
}

impl From<RcString> for PropertyKey {
    #[inline]
    fn from(string: RcString) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(string)
        }
    }
}

impl From<&str> for PropertyKey {
    #[inline]
    fn from(string: &str) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(string.into())
        }
    }
}

impl From<String> for PropertyKey {
    #[inline]
    fn from(string: String) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(string.into())
        }
    }
}

impl From<Box<str>> for PropertyKey {
    #[inline]
    fn from(string: Box<str>) -> PropertyKey {
        if let Ok(index) = string.parse() {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(string.into())
        }
    }
}

impl From<RcSymbol> for PropertyKey {
    #[inline]
    fn from(symbol: RcSymbol) -> PropertyKey {
        PropertyKey::Symbol(symbol)
    }
}

impl fmt::Display for PropertyKey {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyKey::String(ref string) => string.fmt(f),
            PropertyKey::Symbol(ref symbol) => symbol.fmt(f),
            PropertyKey::Index(index) => index.fmt(f),
        }
    }
}

impl From<&PropertyKey> for Value {
    #[inline]
    fn from(property_key: &PropertyKey) -> Value {
        match property_key {
            PropertyKey::String(ref string) => string.clone().into(),
            PropertyKey::Symbol(ref symbol) => symbol.clone().into(),
            PropertyKey::Index(index) => {
                if let Ok(integer) = i32::try_from(*index) {
                    Value::integer(integer)
                } else {
                    Value::number(*index)
                }
            }
        }
    }
}

impl From<PropertyKey> for Value {
    #[inline]
    fn from(property_key: PropertyKey) -> Value {
        match property_key {
            PropertyKey::String(ref string) => string.clone().into(),
            PropertyKey::Symbol(ref symbol) => symbol.clone().into(),
            PropertyKey::Index(index) => {
                if let Ok(integer) = i32::try_from(index) {
                    Value::integer(integer)
                } else {
                    Value::number(index)
                }
            }
        }
    }
}

impl From<u8> for PropertyKey {
    fn from(value: u8) -> Self {
        PropertyKey::Index(value.into())
    }
}

impl From<u16> for PropertyKey {
    fn from(value: u16) -> Self {
        PropertyKey::Index(value.into())
    }
}

impl From<u32> for PropertyKey {
    fn from(value: u32) -> Self {
        PropertyKey::Index(value)
    }
}

impl From<usize> for PropertyKey {
    fn from(value: usize) -> Self {
        if let Ok(index) = u32::try_from(value) {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(RcString::from(value.to_string()))
        }
    }
}

impl From<isize> for PropertyKey {
    fn from(value: isize) -> Self {
        if let Ok(index) = u32::try_from(value) {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(RcString::from(value.to_string()))
        }
    }
}

impl From<i32> for PropertyKey {
    fn from(value: i32) -> Self {
        if let Ok(index) = u32::try_from(value) {
            PropertyKey::Index(index)
        } else {
            PropertyKey::String(RcString::from(value.to_string()))
        }
    }
}

impl From<f64> for PropertyKey {
    fn from(value: f64) -> Self {
        use num_traits::cast::FromPrimitive;
        if let Some(index) = u32::from_f64(value) {
            return PropertyKey::Index(index);
        }

        PropertyKey::String(ryu_js::Buffer::new().format(value).into())
    }
}

impl PartialEq<&str> for PropertyKey {
    fn eq(&self, other: &&str) -> bool {
        match self {
            PropertyKey::String(ref string) => string == other,
            _ => false,
        }
    }
}
