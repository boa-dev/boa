use crate::js::value::{from_value, to_value, FromValue, ToValue, Value, ValueData};
use gc::Gc;
use gc_derive::{Finalize, Trace};

/// A Javascript Property AKA The Property Descriptor   
/// [[SPEC] - The Property Descriptor Specification Type](https://tc39.github.io/ecma262/#sec-property-descriptor-specification-type)   
/// [[SPEC] - Default Attribute Values](https://tc39.github.io/ecma262/#table-4)
#[derive(Trace, Finalize, Clone, Debug)]
pub struct Property {
    /// If the type of this can be changed and this can be deleted
    pub configurable: bool,
    /// If the property shows up in enumeration of the object
    pub enumerable: bool,
    /// If this property can be changed with an assignment
    pub writable: bool,
    /// The value associated with the property
    pub value: Value,
    /// The function serving as getter
    pub get: Value,
    /// The function serving as setter
    pub set: Value,
}

impl Property {
    /// Checks if the provided Value can be used as a property key.
    pub fn is_property_key(value: &Value) -> bool {
        value.is_string() // || value.is_symbol() // Uncomment this when we are handeling symbols.
    }

    /// Make a new property with the given value
    pub fn new(value: Value) -> Self {
        Self {
            configurable: false,
            enumerable: false,
            writable: false,
            value,
            get: Gc::new(ValueData::Undefined),
            set: Gc::new(ValueData::Undefined),
        }
    }
}

impl ToValue for Property {
    fn to_value(&self) -> Value {
        let prop = ValueData::new_obj(None);
        prop.set_field_slice("configurable", to_value(self.configurable));
        prop.set_field_slice("enumerable", to_value(self.enumerable));
        prop.set_field_slice("writable", to_value(self.writable));
        prop.set_field_slice("value", self.value.clone());
        prop.set_field_slice("get", self.get.clone());
        prop.set_field_slice("set", self.set.clone());
        prop
    }
}

impl FromValue for Property {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(Self {
            configurable: from_value(v.get_field_slice("configurable")).unwrap(),
            enumerable: from_value(v.get_field_slice("enumerable")).unwrap(),
            writable: from_value(v.get_field_slice("writable")).unwrap(),
            value: v.get_field_slice("value"),
            get: v.get_field_slice("get"),
            set: v.get_field_slice("set"),
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
