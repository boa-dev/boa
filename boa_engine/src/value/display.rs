use std::borrow::Cow;

use crate::{
    builtins::promise::PromiseState, object::ObjectKind, property::PropertyDescriptor,
    string::utf16, JsError, JsString,
};

use super::{fmt, Display, HashSet, JsValue};

/// This object is used for displaying a `Value`.
#[derive(Debug, Clone, Copy)]
pub struct ValueDisplay<'value> {
    pub(super) value: &'value JsValue,
    pub(super) internals: bool,
}

impl ValueDisplay<'_> {
    /// Display internal information about value.
    ///
    /// By default this is `false`.
    #[inline]
    #[must_use]
    pub const fn internals(mut self, yes: bool) -> Self {
        self.internals = yes;
        self
    }
}

/// A helper macro for printing objects
/// Can be used to print both properties and internal slots
/// All of the overloads take:
/// - The object to be printed
/// - The function with which to print
/// - The indentation for the current level (for nested objects)
/// - A `HashSet` with the addresses of the already printed objects for the current branch
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
        {
            let object = $obj.borrow();
            if let Some(object) = object.prototype() {
                vec![format!(
                    "{:>width$}: {}",
                    "__proto__",
                    $display_fn(&object.clone().into(), $encounters, $indent.wrapping_add(4), true),
                    width = $indent,
                )]
            } else {
                vec![format!(
                    "{:>width$}: {}",
                    "__proto__",
                    JsValue::Null.display(),
                    width = $indent,
                )]
            }
        }
    };
    (props of $obj:expr, $display_fn:ident, $indent:expr, $encounters:expr, $print_internals:expr) => {
        {let mut keys: Vec<_> = $obj.borrow().properties().index_property_keys().map(crate::property::PropertyKey::Index).collect();
        keys.extend($obj.borrow().properties().shape.keys());
        let mut result = Vec::default();
        for key in keys {
            let val = $obj.borrow().properties().get(&key).expect("There should be a value");
            if val.is_data_descriptor() {
                let v = &val.expect_value();
                result.push(format!(
                    "{:>width$}: {}",
                    key,
                    $display_fn(v, $encounters, $indent.wrapping_add(4), $print_internals),
                    width = $indent,
                ));
            } else {
               let display = match (val.set().is_some(), val.get().is_some()) {
                    (true, true) => "Getter & Setter",
                    (true, false) => "Setter",
                    (false, true) => "Getter",
                    _ => "No Getter/Setter"
                };
               result.push(format!("{:>width$}: {}", key, display, width = $indent));
            }
        }
        result}
    };
}

pub(crate) fn log_string_from(x: &JsValue, print_internals: bool, print_children: bool) -> String {
    match x {
        // We don't want to print private (compiler) or prototype properties
        JsValue::Object(ref v) => {
            // Can use the private "type" field of an Object to match on
            // which type of Object it represents for special printing
            match v.borrow().kind() {
                ObjectKind::String(ref string) => {
                    format!("String {{ \"{}\" }}", string.to_std_string_escaped())
                }
                ObjectKind::Boolean(boolean) => format!("Boolean {{ {boolean} }}"),
                ObjectKind::Number(rational) => {
                    if rational.is_sign_negative() && *rational == 0.0 {
                        "Number { -0 }".to_string()
                    } else {
                        let mut buffer = ryu_js::Buffer::new();
                        format!("Number {{ {} }}", buffer.format(*rational))
                    }
                }
                ObjectKind::Array => {
                    let len = v
                        .borrow()
                        .properties()
                        .get(&utf16!("length").into())
                        .expect("array object must have 'length' property")
                        // FIXME: handle accessor descriptors
                        .expect_value()
                        .as_number()
                        .map(|n| n as i32)
                        .unwrap_or_default();

                    if print_children {
                        if len == 0 {
                            return String::from("[]");
                        }

                        let arr = (0..len)
                            .map(|i| {
                                // Introduce recursive call to stringify any objects
                                // which are part of the Array

                                // FIXME: handle accessor descriptors
                                if let Some(value) = v
                                    .borrow()
                                    .properties()
                                    .get(&i.into())
                                    .and_then(|x| x.value().cloned())
                                {
                                    log_string_from(&value, print_internals, false)
                                } else {
                                    String::from("<empty>")
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(", ");

                        format!("[ {arr} ]")
                    } else {
                        format!("Array({len})")
                    }
                }
                ObjectKind::Map(ref map) => {
                    let size = map.len();
                    if size == 0 {
                        return String::from("Map(0)");
                    }

                    if print_children {
                        let mappings = map
                            .iter()
                            .map(|(key, value)| {
                                let key = log_string_from(key, print_internals, false);
                                let value = log_string_from(value, print_internals, false);
                                format!("{key} → {value}")
                            })
                            .collect::<Vec<String>>()
                            .join(", ");
                        format!("Map {{ {mappings} }}")
                    } else {
                        format!("Map({size})")
                    }
                }
                ObjectKind::Set(ref set) => {
                    let size = set.len();

                    if size == 0 {
                        return String::from("Set(0)");
                    }

                    if print_children {
                        let entries = set
                            .iter()
                            .map(|value| log_string_from(value, print_internals, false))
                            .collect::<Vec<String>>()
                            .join(", ");
                        format!("Set {{ {entries} }}")
                    } else {
                        format!("Set({size})")
                    }
                }
                ObjectKind::Error(_) => {
                    let name: Cow<'static, str> = v
                        .get_property(&utf16!("name").into())
                        .as_ref()
                        .and_then(PropertyDescriptor::value)
                        .map_or_else(
                            || "<error>".into(),
                            |v| {
                                v.as_string()
                                    .map_or_else(
                                        || v.display().to_string(),
                                        JsString::to_std_string_escaped,
                                    )
                                    .into()
                            },
                        );
                    let message = v
                        .get_property(&utf16!("message").into())
                        .as_ref()
                        .and_then(PropertyDescriptor::value)
                        .map(|v| {
                            v.as_string().map_or_else(
                                || v.display().to_string(),
                                JsString::to_std_string_escaped,
                            )
                        })
                        .unwrap_or_default();
                    if name.is_empty() {
                        message
                    } else if message.is_empty() {
                        name.to_string()
                    } else {
                        format!("{name}: {message}")
                    }
                }
                ObjectKind::Promise(ref promise) => {
                    format!(
                        "Promise {{ {} }}",
                        match promise.state() {
                            PromiseState::Pending => Cow::Borrowed("<pending>"),
                            PromiseState::Fulfilled(val) => Cow::Owned(val.display().to_string()),
                            PromiseState::Rejected(reason) => Cow::Owned(format!(
                                "<rejected> {}",
                                JsError::from_opaque(reason.clone())
                            )),
                        }
                    )
                }
                _ => x.display_obj(print_internals),
            }
        }
        _ => x.display().to_string(),
    }
}

impl JsValue {
    /// A helper function for specifically printing object values
    pub fn display_obj(&self, print_internals: bool) -> String {
        // A simple helper for getting the address of a value
        // TODO: Find a more general place for this, as it can be used in other situations as well
        fn address_of<T>(t: &T) -> usize {
            let my_ptr: *const T = t;
            my_ptr as usize
        }

        fn display_obj_internal(
            data: &JsValue,
            encounters: &mut HashSet<usize>,
            indent: usize,
            print_internals: bool,
        ) -> String {
            if let JsValue::Object(ref v) = *data {
                // The in-memory address of the current object
                let addr = address_of(v.as_ref());

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
                // it will not cause an infinite printing loop, so it is safe to be printed again
                encounters.remove(&addr);

                let closing_indent = String::from_utf8(vec![b' '; indent.wrapping_sub(4)])
                    .expect("Could not create the closing brace's indentation string");

                format!("{{\n{result}\n{closing_indent}}}")
            } else {
                // Every other type of data is printed with the display method
                data.display().to_string()
            }
        }

        // We keep track of which objects we have encountered by keeping their
        // in-memory address in this set
        let mut encounters = HashSet::new();

        display_obj_internal(self, &mut encounters, 4, print_internals)
    }
}

impl Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            JsValue::Null => write!(f, "null"),
            JsValue::Undefined => write!(f, "undefined"),
            JsValue::Boolean(v) => write!(f, "{v}"),
            JsValue::Symbol(ref symbol) => {
                write!(f, "{}", symbol.descriptive_string().to_std_string_escaped())
            }
            JsValue::String(ref v) => write!(f, "\"{}\"", v.to_std_string_escaped()),
            JsValue::Rational(v) => format_rational(*v, f),
            JsValue::Object(_) => {
                write!(f, "{}", log_string_from(self.value, self.internals, true))
            }
            JsValue::Integer(v) => write!(f, "{v}"),
            JsValue::BigInt(ref num) => write!(f, "{num}n"),
        }
    }
}

/// This is different from the ECMAScript compliant number to string, in the printing of `-0`.
///
/// This function prints `-0` as `-0` instead of positive `0` as the specification says.
/// This is done to make it easer for the user of the REPL to identify what is a `-0` vs `0`,
/// since the REPL is not bound to the ECMAScript specification we can do this.
fn format_rational(v: f64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if v.is_sign_negative() && v == 0.0 {
        f.write_str("-0")
    } else {
        let mut buffer = ryu_js::Buffer::new();
        write!(f, "{}", buffer.format(v))
    }
}
