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

use crate::builtins::Value;
use gc::{Finalize, Trace};

pub mod attribute;
pub use attribute::Attribute;

#[cfg(test)]
mod tests;

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
pub struct Property {
    pub(crate) attribute: Attribute,
    /// The value associated with the property
    pub value: Option<Value>,
    /// The function serving as getter
    pub get: Option<Value>,
    /// The function serving as setter
    pub set: Option<Value>,
}

impl Property {
    /// Checks if the provided Value can be used as a property key.
    #[inline]
    pub fn is_property_key(value: &Value) -> bool {
        value.is_string() || value.is_symbol()
    }

    /// Make a new property with the given value
    /// The difference between New and Default:
    ///
    /// New: zeros everything to make an empty object
    /// Default: Defaults according to the spec
    #[inline]
    pub fn new() -> Self {
        Self {
            attribute: Default::default(),
            value: None,
            get: None,
            set: None,
        }
    }

    #[inline]
    pub fn empty() -> Self {
        Self {
            attribute: Attribute::NONE,
            value: None,
            get: None,
            set: None,
        }
    }

    #[inline]
    pub fn data_descriptor(value: Value, attribute: Attribute) -> Self {
        Self {
            attribute,
            value: Some(value),
            get: None,
            set: None,
        }
    }

    /// Get the
    #[inline]
    pub fn configurable(&self) -> bool {
        self.attribute.configurable()
    }

    #[inline]
    pub fn set_configurable(&mut self, configurable: bool) {
        self.attribute.set_configurable(configurable)
    }

    #[inline]
    pub fn configurable_or(&self, value: bool) -> bool {
        if self.attribute.has_configurable() {
            self.attribute.configurable()
        } else {
            value
        }
    }

    /// Set enumerable
    #[inline]
    pub fn enumerable(&self) -> bool {
        self.attribute.enumerable()
    }

    #[inline]
    pub fn enumerable_or(&self, value: bool) -> bool {
        if self.attribute.has_enumerable() {
            self.attribute.enumerable()
        } else {
            value
        }
    }

    /// Set writable
    #[inline]
    pub fn writable(&self) -> bool {
        self.attribute.writable()
    }

    #[inline]
    pub fn writable_or(&self, value: bool) -> bool {
        if self.attribute.has_writable() {
            self.attribute.writable()
        } else {
            value
        }
    }

    /// Set value
    #[inline]
    pub fn value(mut self, value: Value) -> Self {
        self.value = Some(value);
        self
    }

    /// Set get
    #[inline]
    pub fn get(mut self, get: Value) -> Self {
        self.get = Some(get);
        self
    }

    #[inline]
    pub fn has_get(&self) -> bool {
        self.get.is_some()
    }

    /// Set set
    #[inline]
    pub fn set(mut self, set: Value) -> Self {
        self.set = Some(set);
        self
    }

    #[inline]
    pub fn has_set(&self) -> bool {
        self.set.is_some()
    }

    /// Is this an empty Property?
    ///
    /// `true` if all fields are set to none
    #[inline]
    pub fn is_none(&self) -> bool {
        self.value.is_none()
            && self.attribute.is_empty()
            && self.get.is_none()
            && self.set.is_none()
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
        self.value.is_some() || self.attribute.has_writable()
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

impl Default for Property {
    /// Make a default property
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#table-default-attribute-values
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Property> for Value {
    fn from(value: &Property) -> Value {
        let property = Value::new_object(None);
        if value.attribute.has_writable() {
            property.set_field("writable", value.attribute.writable());
        }

        if value.attribute.has_enumerable() {
            property.set_field("enumerable", value.attribute.enumerable());
        }

        if value.attribute.has_configurable() {
            property.set_field("configurable", value.attribute.configurable());
        }

        property.set_field("value", value.value.clone().unwrap_or_else(Value::null));
        property.set_field("get", value.get.clone().unwrap_or_else(Value::null));
        property.set_field("set", value.set.clone().unwrap_or_else(Value::null));
        property
    }
}

impl<'a> From<&'a Value> for Property {
    /// Attempt to fetch values "configurable", "enumerable", "writable" from the value,
    /// if they're not there default to false
    fn from(value: &Value) -> Self {
        let mut attribute = Attribute::empty();

        let writable = value.get_field("writable");
        if !writable.is_undefined() {
            attribute.set_writable(bool::from(&writable));
        }

        let enumerable = value.get_field("enumerable");
        if !enumerable.is_undefined() {
            attribute.set_enumerable(bool::from(&enumerable));
        }

        let configurable = value.get_field("configurable");
        if !configurable.is_undefined() {
            attribute.set_configurable(bool::from(&configurable));
        }

        Self {
            attribute,
            value: Some(value.get_field("value")),
            get: Some(value.get_field("get")),
            set: Some(value.get_field("set")),
        }
    }
}
