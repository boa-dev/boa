use crate::builtins::value::{from_value, to_value, FromValue, ToValue, Value, ValueData};
use gc_derive::{Finalize, Trace};

/// A Javascript Property AKA The Property Descriptor   
/// [[SPEC] - The Property Descriptor Specification Type](https://tc39.github.io/ecma262/#sec-property-descriptor-specification-type)   
/// [[SPEC] - Default Attribute Values](https://tc39.github.io/ecma262/#table-4)
///
/// Any field in a JavaScript Property may be present or absent.
#[derive(Trace, Finalize, Clone, Debug)]
pub struct Property {
    /// If the type of this can be changed and this can be deleted
    pub configurable: Option<bool>,
    /// If the property shows up in enumeration of the object
    pub enumerable: Option<bool>,
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
            configurable: None,
            enumerable: None,
            writable: None,
            value: None,
            get: None,
            set: None,
        }
    }

    /// Set configurable
    pub fn configurable(mut self, configurable: bool) -> Self {
        self.configurable = Some(configurable);
        self
    }

    /// Set enumerable
    pub fn enumerable(mut self, enumerable: bool) -> Self {
        self.enumerable = Some(enumerable);
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
        self.get.is_none()
            && self.set.is_none()
            && self.writable.is_none()
            && self.configurable.is_none()
            && self.enumerable.is_none()
    }

    /// An accessor Property Descriptor is one that includes any fields named either [[Get]] or [[Set]].   
    /// <https://tc39.es/ecma262/#sec-isaccessordescriptor>
    pub fn is_accessor_descriptor(&self) -> bool {
        self.get.is_some() || self.set.is_some()
    }

    /// A data Property Descriptor is one that includes any fields named either [[Value]] or [[Writable]].   
    /// https://tc39.es/ecma262/#sec-isdatadescriptor
    pub fn is_data_descriptor(&self) -> bool {
        self.value.is_some() || self.writable.is_some()
    }

    /// https://tc39.es/ecma262/#sec-isgenericdescriptor
    pub fn is_generic_descriptor(&self) -> bool {
        !self.is_accessor_descriptor() && !self.is_data_descriptor()
    }
}

impl Default for Property {
    /// Make a default property
    /// https://tc39.es/ecma262/#table-default-attribute-values
    fn default() -> Self {
        Self {
            configurable: None,
            enumerable: None,
            writable: None,
            value: None,
            get: None,
            set: None,
        }
    }
}

impl ToValue for Property {
    fn to_value(&self) -> Value {
        let prop = ValueData::new_obj(None);
        prop.set_field_slice("configurable", to_value(self.configurable));
        prop.set_field_slice("enumerable", to_value(self.enumerable));
        prop.set_field_slice("writable", to_value(self.writable));
        prop.set_field_slice("value", to_value(self.value.clone()));
        prop.set_field_slice("get", to_value(self.get.clone()));
        prop.set_field_slice("set", to_value(self.set.clone()));
        prop
    }
}

impl FromValue for Property {
    /// Attempt to fetch values "configurable", "enumerable", "writable" from the value,
    /// if they're not there default to false
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(Self {
            configurable: {
                match from_value::<bool>(v.get_field_slice("configurable")) {
                    Ok(v) => Some(v),
                    Err(_) => Some(false),
                }
            },
            enumerable: {
                match from_value::<bool>(v.get_field_slice("enumerable")) {
                    Ok(v) => Some(v),
                    Err(_) => Some(false),
                }
            },
            writable: {
                match from_value(v.get_field_slice("writable")) {
                    Ok(v) => Some(v),
                    Err(_) => Some(false),
                }
            },
            value: Some(v.get_field_slice("value")),
            get: Some(v.get_field_slice("get")),
            set: Some(v.get_field_slice("set")),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_property_key_test() {
        let v = Value::new(ValueData::String(String::from("Boop")));
        assert!(Property::is_property_key(&v));

        let v = Value::new(ValueData::Boolean(true));
        assert!(!Property::is_property_key(&v));
    }
}
