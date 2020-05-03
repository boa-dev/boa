//! This module implements the JavaScript Value.
//!
//! Javascript values, utility methods and conversion between Javascript values and Rust values.

#[cfg(test)]
mod tests;

use crate::builtins::{
    function::Function,
    object::{
        internal_methods_trait::ObjectInternalMethods, InternalState, InternalStateCell, Object,
        ObjectKind, INSTANCE_PROTOTYPE, PROTOTYPE,
    },
    property::Property,
};
use gc::{Gc, GcCell, GcCellRef};
use gc_derive::{Finalize, Trace};
use serde_json::{map::Map, Number as JSONNumber, Value as JSONValue};
use std::{
    any::Any,
    collections::HashSet,
    f64::NAN,
    fmt::{self, Display},
    ops::{Add, BitAnd, BitOr, BitXor, Deref, DerefMut, Div, Mul, Not, Rem, Shl, Shr, Sub},
    str::FromStr,
};

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
#[must_use]
pub type ResultValue = Result<Value, Value>;

/// A Garbage-collected Javascript value as represented in the interpreter.
pub type Value = Gc<ValueData>;

pub fn undefined() -> Value {
    Gc::new(ValueData::Undefined)
}

/// A Javascript value
#[derive(Trace, Finalize, Debug, Clone)]
pub enum ValueData {
    /// `null` - A null value, for when a value doesn't exist
    Null,
    /// `undefined` - An undefined value, for when a field or index doesn't exist
    Undefined,
    /// `boolean` - A `true` / `false` value, for if a certain criteria is met
    Boolean(bool),
    /// `String` - A UTF-8 string, such as `"Hello, world"`
    String(String),
    /// `Number` - A 64-bit floating point number, such as `3.1415`
    Rational(f64),
    /// `Number` - A 32-bit integer, such as `42`
    Integer(i32),
    /// `Object` - An object, such as `Math`, represented by a binary tree of string keys to Javascript values
    Object(Box<GcCell<Object>>),
    /// `Symbol` - A Symbol Type - Internally Symbols are similar to objects, except there are no properties, only internal slots
    Symbol(Box<GcCell<Object>>),
}

impl ValueData {
    /// Returns a new empty object
    pub fn new_obj(global: Option<&Value>) -> Value {
        if let Some(glob) = global {
            let obj_proto = glob.get_field_slice("Object").get_field_slice(PROTOTYPE);

            let obj = Object::create(obj_proto);
            Gc::new(Self::Object(Box::new(GcCell::new(obj))))
        } else {
            let obj = Object::default();
            Gc::new(Self::Object(Box::new(GcCell::new(obj))))
        }
    }

    /// Similar to `new_obj`, but you can pass a prototype to create from,
    /// plus a kind
    pub fn new_obj_from_prototype(proto: Value, kind: ObjectKind) -> Value {
        let mut obj = Object::default();
        obj.kind = kind;

        obj.internal_slots
            .insert(INSTANCE_PROTOTYPE.to_string(), proto);

        Gc::new(Self::Object(Box::new(GcCell::new(obj))))
    }

    /// This will tell us if we can exten an object or not, not properly implemented yet
    ///
    /// For now always returns true.
    ///
    /// For scalar types it should be false, for objects check the private field for extensibilaty.
    /// By default true.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/seal would turn extensible to false/>
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/freeze would also turn extensible to false/>
    pub fn is_extensible(&self) -> bool {
        true
    }

    /// Returns true if the value is an object
    pub fn is_object(&self) -> bool {
        match *self {
            Self::Object(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a symbol
    pub fn is_symbol(&self) -> bool {
        match *self {
            Self::Symbol(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a function
    pub fn is_function(&self) -> bool {
        match *self {
            Self::Object(ref o) => {
                let borrowed_obj = o.borrow();
                borrowed_obj.is_callable() || borrowed_obj.is_constructor()
            }
            _ => false,
        }
    }

    /// Returns true if the value is undefined
    pub fn is_undefined(&self) -> bool {
        match *self {
            Self::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is null
    pub fn is_null(&self) -> bool {
        match *self {
            Self::Null => true,
            _ => false,
        }
    }

    /// Returns true if the value is null or undefined
    pub fn is_null_or_undefined(&self) -> bool {
        match *self {
            Self::Null | Self::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is a 64-bit floating-point number
    pub fn is_double(&self) -> bool {
        match *self {
            Self::Rational(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is integer.
    #[allow(clippy::float_cmp)]
    pub fn is_integer(&self) -> bool {
        // If it can fit in a i32 and the trucated version is
        // equal to the original then it is an integer.
        let is_racional_intiger = |n: f64| n == ((n as i32) as f64);

        match *self {
            Self::Integer(_) => true,
            Self::Rational(n) if is_racional_intiger(n) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a number
    pub fn is_num(&self) -> bool {
        self.is_double()
    }

    /// Returns true if the value is a string
    pub fn is_string(&self) -> bool {
        match *self {
            Self::String(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a boolean
    pub fn is_boolean(&self) -> bool {
        match *self {
            Self::Boolean(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is true
    ///
    /// [toBoolean](https://tc39.es/ecma262/#sec-toboolean)
    pub fn is_true(&self) -> bool {
        match *self {
            Self::Object(_) => true,
            Self::String(ref s) if !s.is_empty() => true,
            Self::Rational(n) if n != 0.0 && !n.is_nan() => true,
            Self::Integer(n) if n != 0 => true,
            Self::Boolean(v) => v,
            _ => false,
        }
    }

    /// Converts the value into a 64-bit floating point number
    pub fn to_num(&self) -> f64 {
        match *self {
            Self::Object(_) | Self::Symbol(_) | Self::Undefined => NAN,
            Self::String(ref str) => match FromStr::from_str(str) {
                Ok(num) => num,
                Err(_) => NAN,
            },
            Self::Rational(num) => num,
            Self::Boolean(true) => 1.0,
            Self::Boolean(false) | Self::Null => 0.0,
            Self::Integer(num) => f64::from(num),
        }
    }

    /// Converts the value into a 32-bit integer
    pub fn to_int(&self) -> i32 {
        match *self {
            Self::Object(_)
            | Self::Undefined
            | Self::Symbol(_)
            | Self::Null
            | Self::Boolean(false) => 0,
            Self::String(ref str) => match FromStr::from_str(str) {
                Ok(num) => num,
                Err(_) => 0,
            },
            Self::Rational(num) => num as i32,
            Self::Boolean(true) => 1,
            Self::Integer(num) => num,
        }
    }

    pub fn as_object(&self) -> Option<GcCellRef<'_, Object>> {
        match *self {
            ValueData::Object(ref o) => Some(o.borrow()),
            _ => None,
        }
    }

    /// remove_prop removes a property from a Value object.
    ///
    /// It will return a boolean based on if the value was removed, if there was no value to remove false is returned
    pub fn remove_prop(&self, field: &str) {
        match *self {
            Self::Object(ref obj) => obj.borrow_mut().deref_mut().properties.remove(field),
            _ => None,
        };
    }

    /// Resolve the property in the object.
    ///
    /// A copy of the Property is returned.
    pub fn get_prop(&self, field: &str) -> Option<Property> {
        // Spidermonkey has its own GetLengthProperty: https://searchfox.org/mozilla-central/source/js/src/vm/Interpreter-inl.h#154
        // This is only for primitive strings, String() objects have their lengths calculated in string.rs
        if self.is_string() && field == "length" {
            if let Self::String(ref s) = *self {
                return Some(Property::default().value(to_value(s.len() as i32)));
            }
        }

        if self.is_undefined() {
            return None;
        }

        let obj: Object = match *self {
            Self::Object(ref obj) => {
                let hash = obj.clone();
                // TODO: This will break, we should return a GcCellRefMut instead
                // into_inner will consume the wrapped value and remove it from the hashmap
                hash.into_inner()
            }
            Self::Symbol(ref obj) => {
                let hash = obj.clone();
                hash.into_inner()
            }
            _ => return None,
        };

        match obj.properties.get(field) {
            Some(val) => Some(val.clone()),
            None => match obj.internal_slots.get(&INSTANCE_PROTOTYPE.to_string()) {
                Some(value) => value.get_prop(field),
                None => None,
            },
        }
    }

    /// update_prop will overwrite individual [Property] fields, unlike
    /// Set_prop, which will overwrite prop with a new Property
    /// Mostly used internally for now
    pub fn update_prop(
        &self,
        field: &str,
        value: Option<Value>,
        enumerable: Option<bool>,
        writable: Option<bool>,
        configurable: Option<bool>,
    ) {
        let obj: Option<Object> = match self {
            Self::Object(ref obj) => Some(obj.borrow_mut().deref_mut().clone()),
            _ => None,
        };

        if let Some(mut obj_data) = obj {
            // Use value, or walk up the prototype chain
            if let Some(ref mut prop) = obj_data.properties.get_mut(field) {
                prop.value = value;
                prop.enumerable = enumerable;
                prop.writable = writable;
                prop.configurable = configurable;
            }
        }
    }

    /// Resolve the property in the object.
    ///
    /// Returns a copy of the Property.
    pub fn get_internal_slot(&self, field: &str) -> Value {
        let obj: Object = match *self {
            Self::Object(ref obj) => {
                let hash = obj.clone();
                hash.into_inner()
            }
            Self::Symbol(ref obj) => {
                let hash = obj.clone();
                hash.into_inner()
            }
            _ => return Gc::new(Self::Undefined),
        };

        match obj.internal_slots.get(field) {
            Some(val) => val.clone(),
            None => Gc::new(Self::Undefined),
        }
    }

    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    /// get_field recieves a Property from get_prop(). It should then return the [[Get]] result value if that's set, otherwise fall back to [[Value]]
    /// TODO: this function should use the get Value if its set
    pub fn get_field(&self, field: Value) -> Value {
        match *field {
            // Our field will either be a String or a Symbol
            Self::String(ref s) => {
                match self.get_prop(s) {
                    Some(prop) => {
                        // If the Property has [[Get]] set to a function, we should run that and return the Value
                        let prop_getter = match prop.get {
                            Some(_) => None,
                            None => None,
                        };

                        // If the getter is populated, use that. If not use [[Value]] instead
                        if let Some(val) = prop_getter {
                            val
                        } else {
                            let val = prop
                                .value
                                .as_ref()
                                .expect("Could not get property as reference");
                            val.clone()
                        }
                    }
                    None => Gc::new(Self::Undefined),
                }
            }
            Self::Symbol(_) => unimplemented!(),
            _ => Gc::new(Self::Undefined),
        }
    }

    /// Check whether an object has an internal state set.
    pub fn has_internal_state(&self) -> bool {
        if let Self::Object(ref obj) = *self {
            obj.borrow().state.is_some()
        } else {
            false
        }
    }

    /// Get the internal state of an object.
    pub fn get_internal_state(&self) -> Option<InternalStateCell> {
        if let Self::Object(ref obj) = *self {
            obj.borrow()
                .state
                .as_ref()
                .map(|state| state.deref().clone())
        } else {
            None
        }
    }

    /// Run a function with a reference to the internal state.
    ///
    /// # Panics
    ///
    /// This will panic if this value doesn't have an internal state or if the internal state doesn't
    /// have the concrete type `S`.
    pub fn with_internal_state_ref<S: Any + InternalState, R, F: FnOnce(&S) -> R>(
        &self,
        f: F,
    ) -> R {
        if let Self::Object(ref obj) = *self {
            let o = obj.borrow();
            let state = o
                .state
                .as_ref()
                .expect("no state")
                .downcast_ref()
                .expect("wrong state type");
            f(state)
        } else {
            panic!("not an object");
        }
    }

    /// Run a function with a mutable reference to the internal state.
    ///
    /// # Panics
    ///
    /// This will panic if this value doesn't have an internal state or if the internal state doesn't
    /// have the concrete type `S`.
    pub fn with_internal_state_mut<S: Any + InternalState, R, F: FnOnce(&mut S) -> R>(
        &self,
        f: F,
    ) -> R {
        if let Self::Object(ref obj) = *self {
            let mut o = obj.borrow_mut();
            let state = o
                .state
                .as_mut()
                .expect("no state")
                .downcast_mut()
                .expect("wrong state type");
            f(state)
        } else {
            panic!("not an object");
        }
    }

    /// Check to see if the Value has the field, mainly used by environment records
    pub fn has_field(&self, field: &str) -> bool {
        self.get_prop(field).is_some()
    }

    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    pub fn get_field_slice(&self, field: &str) -> Value {
        // get_field used to accept strings, but now Symbols accept it needs to accept a value
        // So this function will now need to Box strings back into values (at least for now)
        let f = Gc::new(Self::String(field.to_string()));
        self.get_field(f)
    }

    /// Set the field in the value
    /// Field could be a Symbol, so we need to accept a Value (not a string)
    pub fn set_field(&self, field: Value, val: Value) -> Value {
        if let Self::Object(ref obj) = *self {
            if obj.borrow().kind == ObjectKind::Array {
                if let Ok(num) = field.to_string().parse::<usize>() {
                    if num > 0 {
                        let len: i32 = from_value(self.get_field_slice("length"))
                            .expect("Could not convert argument to i32");
                        if len < (num + 1) as i32 {
                            self.set_field_slice("length", to_value(num + 1));
                        }
                    }
                }
            }

            // Symbols get saved into a different bucket to general properties
            if field.is_symbol() {
                obj.borrow_mut().set(field, val.clone());
            } else {
                obj.borrow_mut()
                    .set(to_value(field.to_string()), val.clone());
            }
        }

        val
    }

    /// Set the field in the value
    pub fn set_field_slice(&self, field: &str, val: Value) -> Value {
        // set_field used to accept strings, but now Symbols accept it needs to accept a value
        // So this function will now need to Box strings back into values (at least for now)
        let f = Gc::new(Self::String(field.to_string()));
        self.set_field(f, val)
    }

    /// Set the private field in the value
    pub fn set_internal_slot(&self, field: &str, val: Value) -> Value {
        if let Self::Object(ref obj) = *self {
            obj.borrow_mut()
                .internal_slots
                .insert(field.to_string(), val.clone());
        }
        val
    }

    /// Set the kind of an object
    pub fn set_kind(&self, kind: ObjectKind) {
        if let Self::Object(ref obj) = *self {
            (*obj.deref().borrow_mut()).kind = kind;
        }
    }

    /// Set the property in the value
    pub fn set_prop(&self, field: String, prop: Property) -> Property {
        if let Self::Object(ref obj) = *self {
            obj.borrow_mut().properties.insert(field, prop.clone());
        }
        prop
    }

    /// Set the property in the value
    pub fn set_prop_slice(&self, field: &str, prop: Property) -> Property {
        self.set_prop(field.to_string(), prop)
    }

    /// Set internal state of an Object. Discards the previous state if it was set.
    pub fn set_internal_state<T: Any + InternalState>(&self, state: T) {
        if let Self::Object(ref obj) = *self {
            obj.borrow_mut()
                .state
                .replace(Box::new(InternalStateCell::new(state)));
        }
    }

    /// Consume the function and return a Value
    pub fn from_func(native_func: Function) -> Value {
        // Object with Kind set to function
        let mut new_func = crate::builtins::object::Object::function();
        // Get Length
        let length = native_func.params.len();
        // Set [[Call]] internal slot
        new_func.set_call(native_func);
        // Wrap Object in GC'd Value
        let new_func_val = to_value(new_func);
        // Set length to parameters
        new_func_val.set_field_slice("length", to_value(length));
        new_func_val
    }

    /// Convert from a JSON value to a JS value
    pub fn from_json(json: JSONValue) -> Self {
        match json {
            JSONValue::Number(v) => {
                Self::Rational(v.as_f64().expect("Could not convert value to f64"))
            }
            JSONValue::String(v) => Self::String(v),
            JSONValue::Bool(v) => Self::Boolean(v),
            JSONValue::Array(vs) => {
                let mut new_obj = Object::default();
                for (idx, json) in vs.iter().enumerate() {
                    new_obj.properties.insert(
                        idx.to_string(),
                        Property::default().value(to_value(json.clone())),
                    );
                }
                new_obj.properties.insert(
                    "length".to_string(),
                    Property::default().value(to_value(vs.len() as i32)),
                );
                Self::Object(Box::new(GcCell::new(new_obj)))
            }
            JSONValue::Object(obj) => {
                let mut new_obj = Object::default();
                for (key, json) in obj.iter() {
                    new_obj.properties.insert(
                        key.clone(),
                        Property::default().value(to_value(json.clone())),
                    );
                }

                Self::Object(Box::new(GcCell::new(new_obj)))
            }
            JSONValue::Null => Self::Null,
        }
    }

    /// Conversts the `Value` to `JSON`.
    pub fn to_json(&self) -> JSONValue {
        match *self {
            Self::Null | Self::Symbol(_) | Self::Undefined => JSONValue::Null,
            Self::Boolean(b) => JSONValue::Bool(b),
            Self::Object(ref obj) => {
                let new_obj = obj
                    .borrow()
                    .properties
                    .iter()
                    .map(|(k, _)| (k.clone(), self.get_field_slice(k).to_json()))
                    .collect::<Map<String, JSONValue>>();
                JSONValue::Object(new_obj)
            }
            Self::String(ref str) => JSONValue::String(str.clone()),
            Self::Rational(num) => JSONValue::Number(
                JSONNumber::from_f64(num).expect("Could not convert to JSONNumber"),
            ),
            Self::Integer(val) => JSONValue::Number(JSONNumber::from(val)),
        }
    }

    /// Get the type of the value
    ///
    /// https://tc39.es/ecma262/#sec-typeof-operator
    pub fn get_type(&self) -> &'static str {
        match *self {
            Self::Rational(_) | Self::Integer(_) => "number",
            Self::String(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Symbol(_) => "symbol",
            Self::Null => "null",
            Self::Undefined => "undefined",
            Self::Object(ref o) => {
                if o.deref().borrow().is_callable() {
                    "function"
                } else {
                    "object"
                }
            }
        }
    }

    pub fn as_num_to_power(&self, other: Self) -> Self {
        Self::Rational(self.to_num().powf(other.to_num()))
    }
}

impl Default for ValueData {
    fn default() -> Self {
        Self::Undefined
    }
}

/// A helper macro for printing objects
/// Can be used to print both properties and internal slots
/// All of the overloads take:
/// - The object to be printed
/// - The function with which to print
/// - The indentation for the current level (for nested objects)
/// - A HashSet with the addresses of the already printed objects for the current branch
///      (used to avoid infinite loops when there are cyclic deps)
macro_rules! print_obj_value {
    (all of $obj:expr, $display_fn:ident, $indent:expr, $encounters:expr) => {
        {
            let mut internals = print_obj_value!(internals of $obj, $display_fn, $indent, $encounters);
            let mut props = print_obj_value!(props of $obj, $display_fn, $indent, $encounters, true);

            props.reserve(internals.len());

            props.append(&mut internals);

            props
        }
    };
    (internals of $obj:expr, $display_fn:ident, $indent:expr, $encounters:expr) => {
        print_obj_value!(impl internal_slots, $obj, |(key, val)| {
            format!(
                "{}{}: {}",
                String::from_utf8(vec![b' '; $indent])
                                .expect("Could not create indentation string"),
                key,
                $display_fn(&val, $encounters, $indent.wrapping_add(4), true)
            )
        })
    };
    (props of $obj:expr, $display_fn:ident, $indent:expr, $encounters:expr, $print_internals:expr) => {
        print_obj_value!(impl properties, $obj, |(key, val)| {
            let v = &val
                .value
                .as_ref()
                .expect("Could not get the property's value");

            format!(
                "{}{}: {}",
                String::from_utf8(vec![b' '; $indent])
                                .expect("Could not create indentation string"),
                key,
                $display_fn(v, $encounters, $indent.wrapping_add(4), $print_internals)
            )
        })
    };

    // A private overload of the macro
    // DO NOT use directly
    (impl $field:ident, $v:expr, $f:expr) => {
        $v
            .borrow()
            .$field
            .iter()
            .map($f)
            .collect::<Vec<String>>()
    };
}

pub(crate) fn log_string_from(x: &ValueData, print_internals: bool) -> String {
    match x {
        // We don't want to print private (compiler) or prototype properties
        ValueData::Object(ref v) => {
            // Can use the private "type" field of an Object to match on
            // which type of Object it represents for special printing
            match v.borrow().kind {
                ObjectKind::String => from_value(
                    v.borrow()
                        .internal_slots
                        .get("StringData")
                        .expect("Cannot get primitive value from String")
                        .clone(),
                )
                .expect("Cannot clone primitive value from String"),
                ObjectKind::Boolean => {
                    let bool_data = v.borrow().get_internal_slot("BooleanData").to_string();

                    format!("Boolean {{ {} }}", bool_data)
                }
                ObjectKind::Array => {
                    let len: i32 = from_value(
                        v.borrow()
                            .properties
                            .get("length")
                            .unwrap()
                            .value
                            .clone()
                            .expect("Could not borrow value"),
                    )
                    .expect("Could not convert JS value to i32");

                    if len == 0 {
                        return String::from("[]");
                    }

                    let arr = (0..len)
                        .map(|i| {
                            // Introduce recursive call to stringify any objects
                            // which are part of the Array
                            log_string_from(
                                &v.borrow()
                                    .properties
                                    .get(&i.to_string())
                                    .unwrap()
                                    .value
                                    .clone()
                                    .expect("Could not borrow value"),
                                print_internals,
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(", ");

                    format!("[ {} ]", arr)
                }
                _ => display_obj(&x, print_internals),
            }
        }
        ValueData::Symbol(ref sym) => {
            let desc: Value = sym.borrow().get_internal_slot("Description");
            match *desc {
                ValueData::String(ref st) => format!("Symbol(\"{}\")", st.to_string()),
                _ => String::from("Symbol()"),
            }
        }

        _ => format!("{}", x),
    }
}

/// A helper function for specifically printing object values
pub(crate) fn display_obj(v: &ValueData, print_internals: bool) -> String {
    // A simple helper for getting the address of a value
    // TODO: Find a more general place for this, as it can be used in other situations as well
    fn address_of<T>(t: &T) -> usize {
        let my_ptr: *const T = t;
        my_ptr as usize
    }

    // We keep track of which objects we have encountered by keeping their
    // in-memory address in this set
    let mut encounters = HashSet::new();

    fn display_obj_internal(
        data: &ValueData,
        encounters: &mut HashSet<usize>,
        indent: usize,
        print_internals: bool,
    ) -> String {
        if let ValueData::Object(ref v) = *data {
            // The in-memory address of the current object
            let addr = address_of(v.borrow().deref());

            // We need not continue if this object has already been
            // printed up the current chain
            if encounters.contains(&addr) {
                return String::from("[Cycle]");
            }

            // Mark the current object as encountered
            encounters.insert(addr);

            let result = if print_internals {
                print_obj_value!(all of v, display_obj_internal, indent, encounters).join(",\n")
            } else {
                print_obj_value!(props of v, display_obj_internal, indent, encounters, print_internals)
                        .join(",\n")
            };

            // If the current object is referenced in a different branch,
            // it will not cause an infinte printing loop, so it is safe to be printed again
            encounters.remove(&addr);

            let closing_indent = String::from_utf8(vec![b' '; indent.wrapping_sub(4)])
                .expect("Could not create the closing brace's indentation string");

            format!("{{\n{}\n{}}}", result, closing_indent)
        } else {
            // Every other type of data is printed as is
            format!("{}", data)
        }
    }

    display_obj_internal(v, &mut encounters, 4, print_internals)
}

impl Display for ValueData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Symbol(ref v) => match *v.borrow().get_internal_slot("Description") {
                // If a description exists use it
                Self::String(ref v) => write!(f, "{}", format!("Symbol({})", v)),
                _ => write!(f, "Symbol()"),
            },
            Self::String(ref v) => write!(f, "{}", v),
            Self::Rational(v) => write!(
                f,
                "{}",
                match v {
                    _ if v.is_nan() => "NaN".to_string(),
                    _ if v.is_infinite() && v.is_sign_negative() => "-Infinity".to_string(),
                    _ if v.is_infinite() => "Infinity".to_string(),
                    _ => v.to_string(),
                }
            ),
            Self::Object(_) => write!(f, "{}", log_string_from(self, true)),
            Self::Integer(v) => write!(f, "{}", v),
        }
    }
}

impl PartialEq for ValueData {
    fn eq(&self, other: &Self) -> bool {
        match (self.clone(), other.clone()) {
            // TODO: fix this
            // _ if self.ptr.to_inner() == &other.ptr.to_inner() => true,
            _ if self.is_null_or_undefined() && other.is_null_or_undefined() => true,
            (Self::String(_), _) | (_, Self::String(_)) => self.to_string() == other.to_string(),
            (Self::Boolean(a), Self::Boolean(b)) if a == b => true,
            (Self::Rational(a), Self::Rational(b)) if a == b && !a.is_nan() && !b.is_nan() => true,
            (Self::Rational(a), _) if a == other.to_num() => true,
            (_, Self::Rational(a)) if a == self.to_num() => true,
            (Self::Integer(a), Self::Integer(b)) if a == b => true,
            _ => false,
        }
    }
}

impl Add for ValueData {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::String(ref s), ref o) => {
                Self::String(format!("{}{}", s.clone(), &o.to_string()))
            }
            (ref s, Self::String(ref o)) => Self::String(format!("{}{}", s.to_string(), o)),
            (ref s, ref o) => Self::Rational(s.to_num() + o.to_num()),
        }
    }
}
impl Sub for ValueData {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::Rational(self.to_num() - other.to_num())
    }
}
impl Mul for ValueData {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::Rational(self.to_num() * other.to_num())
    }
}
impl Div for ValueData {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self::Rational(self.to_num() / other.to_num())
    }
}
impl Rem for ValueData {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        Self::Rational(self.to_num() % other.to_num())
    }
}
impl BitAnd for ValueData {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self::Integer(self.to_int() & other.to_int())
    }
}
impl BitOr for ValueData {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self::Integer(self.to_int() | other.to_int())
    }
}
impl BitXor for ValueData {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        Self::Integer(self.to_int() ^ other.to_int())
    }
}
impl Shl for ValueData {
    type Output = Self;
    fn shl(self, other: Self) -> Self {
        Self::Integer(self.to_int() << other.to_int())
    }
}
impl Shr for ValueData {
    type Output = Self;
    fn shr(self, other: Self) -> Self {
        Self::Integer(self.to_int() >> other.to_int())
    }
}
impl Not for ValueData {
    type Output = Self;
    fn not(self) -> Self {
        Self::Boolean(!self.is_true())
    }
}

/// Conversion to Javascript values from Rust values
pub trait ToValue {
    /// Convert this value to a Rust value
    fn to_value(&self) -> Value;
}
/// Conversion to Rust values from Javascript values
pub trait FromValue {
    /// Convert this value to a Javascript value
    fn from_value(value: Value) -> Result<Self, &'static str>
    where
        Self: Sized;
}

impl ToValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl FromValue for Value {
    fn from_value(value: Value) -> Result<Self, &'static str> {
        Ok(value)
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(self.clone()))
    }
}

impl FromValue for String {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_string())
    }
}

impl<'s> ToValue for &'s str {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(
            String::from_str(*self).expect("Could not convert string to self to String"),
        ))
    }
}

impl ToValue for char {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(self.to_string()))
    }
}
impl FromValue for char {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_string()
            .chars()
            .next()
            .expect("Could not get next char"))
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Rational(*self))
    }
}
impl FromValue for f64 {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_num())
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Integer(*self))
    }
}
impl FromValue for i32 {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_int())
    }
}

impl ToValue for usize {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Integer(*self as i32))
    }
}
impl FromValue for usize {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_int() as Self)
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Boolean(*self))
    }
}
impl FromValue for bool {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.is_true())
    }
}

impl<'s, T: ToValue> ToValue for &'s [T] {
    fn to_value(&self) -> Value {
        let mut arr = Object::default();
        for (i, item) in self.iter().enumerate() {
            arr.properties
                .insert(i.to_string(), Property::default().value(item.to_value()));
        }
        to_value(arr)
    }
}
impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self) -> Value {
        let mut arr = Object::default();
        for (i, item) in self.iter().enumerate() {
            arr.properties
                .insert(i.to_string(), Property::default().value(item.to_value()));
        }
        to_value(arr)
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        let len = v.get_field_slice("length").to_int();
        let mut vec = Self::with_capacity(len as usize);
        for i in 0..len {
            vec.push(from_value(v.get_field_slice(&i.to_string()))?)
        }
        Ok(vec)
    }
}

impl ToValue for Object {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Object(Box::new(GcCell::new(self.clone()))))
    }
}

impl FromValue for Object {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        match *v {
            ValueData::Object(ref obj) => Ok(obj.clone().into_inner()),
            _ => Err("Value is not a valid object"),
        }
    }
}

impl ToValue for JSONValue {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::from_json(self.clone()))
    }
}

impl FromValue for JSONValue {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_json())
    }
}

impl ToValue for () {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Null)
    }
}
impl FromValue for () {
    fn from_value(_: Value) -> Result<(), &'static str> {
        Ok(())
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self) -> Value {
        match *self {
            Some(ref v) => v.to_value(),
            None => Gc::new(ValueData::Null),
        }
    }
}
impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: Value) -> Result<Self, &'static str> {
        Ok(if value.is_null_or_undefined() {
            None
        } else {
            Some(FromValue::from_value(value)?)
        })
    }
}

/// A utility function that just calls `FromValue::from_value`
pub fn from_value<A: FromValue>(v: Value) -> Result<A, &'static str> {
    FromValue::from_value(v)
}

/// A utility function that just calls `ToValue::to_value`
pub fn to_value<A: ToValue>(v: A) -> Value {
    v.to_value()
}

/// The internal comparison abstract operation SameValue(x, y),
/// where x and y are ECMAScript language values, produces true or false.
/// Such a comparison is performed as follows:
///
/// https://tc39.es/ecma262/#sec-samevalue
/// strict mode currently compares the pointers
pub fn same_value(x: &Value, y: &Value, strict: bool) -> bool {
    if strict {
        // Do both Values point to the same underlying valueData?
        let x_ptr = Gc::into_raw(x.clone());
        let y_ptr = Gc::into_raw(y.clone());
        return x_ptr == y_ptr;
    }

    if x.get_type() != y.get_type() {
        return false;
    }

    if x.get_type() == "number" {
        let native_x: f64 = from_value(x.clone()).expect("failed to get value");
        let native_y: f64 = from_value(y.clone()).expect("failed to get value");
        return native_x.abs() - native_y.abs() == 0.0;
    }

    same_value_non_number(x, y)
}

pub fn same_value_non_number(x: &Value, y: &Value) -> bool {
    debug_assert!(x.get_type() == y.get_type());
    match x.get_type() {
        "undefined" => true,
        "null" => true,
        "string" => {
            if x.to_string() == y.to_string() {
                return true;
            }
            false
        }
        "boolean" => {
            from_value::<bool>(x.clone()).expect("failed to get value")
                == from_value::<bool>(y.clone()).expect("failed to get value")
        }
        "object" => *x == *y,
        _ => false,
    }
}
