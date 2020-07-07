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

use crate::builtins::value::Value;
use gc::{Finalize, Trace};

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
    /// If the type of this can be changed and this can be deleted
    pub configurable: bool,
    /// If the property shows up in enumeration of the object
    pub enumerable: bool,
    /// If this property can be changed with an assignment
    pub writable: Option<bool>,
    /// The value associated with the property
    pub value: Option<Value>,
    /// The function serving as getter
    pub get: Option<Value>,
    /// The function serving as setter
    pub set: Option<Value>,
}

impl Property {
    /// Checks if the provided Value can be used as a property key.
    pub fn is_property_key(value: &Value) -> bool {
        value.is_string() || value.is_symbol() // Uncomment this when we are handeling symbols.
    }

    /// Make a new property with the given value
    /// The difference between New and Default:
    ///
    /// New: zeros everything to make an empty object
    /// Default: Defaults according to the spec
    pub fn new() -> Self {
        Self {
            configurable: false,
            enumerable: false,
            writable: None,
            value: None,
            get: None,
            set: None,
        }
    }

    /// Set configurable
    pub fn configurable(mut self, configurable: bool) -> Self {
        self.configurable = configurable;
        self
    }

    /// Set enumerable
    pub fn enumerable(mut self, enumerable: bool) -> Self {
        self.enumerable = enumerable;
        self
    }

    /// Set writable
    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = Some(writable);
        self
    }

    /// Set value
    pub fn value(mut self, value: Value) -> Self {
        self.value = Some(value);
        self
    }

    /// Set get
    pub fn get(mut self, get: Value) -> Self {
        self.get = Some(get);
        self
    }

    /// Set set
    pub fn set(mut self, set: Value) -> Self {
        self.set = Some(set);
        self
    }

    /// Is this an empty Property?
    ///
    /// `true` if all fields are set to none
    pub fn is_none(&self) -> bool {
        self.get.is_none() && self.set.is_none() && self.writable.is_none()
    }

    /// An accessor Property Descriptor is one that includes any fields named either [[Get]] or [[Set]].
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isaccessordescriptor
    pub fn is_accessor_descriptor(&self) -> bool {
        self.get.is_some() || self.set.is_some()
    }

    /// A data Property Descriptor is one that includes any fields named either [[Value]] or [[Writable]].
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdatadescriptor
    pub fn is_data_descriptor(&self) -> bool {
        self.value.is_some() || self.writable.is_some()
    }

    /// Check if a property is generic.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isgenericdescriptor
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
    fn default() -> Self {
        Self {
            configurable: false,
            enumerable: false,
            writable: None,
            value: None,
            get: None,
            set: None,
        }
    }
}

impl From<&Property> for Value {
    fn from(value: &Property) -> Value {
        let property = Value::new_object(None);
        property.set_field("configurable", Value::from(value.configurable));
        property.set_field("enumerable", Value::from(value.enumerable));
        property.set_field("writable", Value::from(value.writable));
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
        Self {
            configurable: bool::from(&value.get_field("configurable")),
            enumerable: bool::from(&value.get_field("enumerable")),
            writable: Some(bool::from(&value.get_field("writable"))),
            value: Some(value.get_field("value")),
            get: Some(value.get_field("get")),
            set: Some(value.get_field("set")),
        }
    }
}
