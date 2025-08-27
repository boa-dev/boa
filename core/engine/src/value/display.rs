use super::{Display, HashSet, JsValue, JsVariant, fmt};
use crate::{
    JsError, JsObject, JsString,
    builtins::{
        Array, Promise, error::Error, map::ordered_map::OrderedMap, promise::PromiseState,
        set::ordered_set::OrderedSet,
    },
    js_string,
    property::{PropertyDescriptor, PropertyKey},
};
use std::{borrow::Cow, fmt::Write};

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

/// A helper function for printing objects
/// Can be used to print both properties and internal slots
/// All of the overloads take:
/// - The object to be printed
/// - The function with which to print
/// - The indentation for the current level (for nested objects)
/// - A `HashSet` with the addresses of the already printed objects for the current branch
///   (used to avoid infinite loops when there are cyclic deps)
fn print_obj_value_all(
    obj: &JsObject,
    display_fn: fn(&JsValue, &mut HashSet<usize>, usize, bool) -> String,
    indent: usize,
    encounters: &mut HashSet<usize>,
) -> Vec<String> {
    let mut internals = print_obj_value_internals(obj, display_fn, indent, encounters);
    let mut props = print_obj_value_props(obj, display_fn, indent, encounters, true);

    props.reserve(internals.len());
    props.append(&mut internals);
    props
}

fn print_obj_value_internals(
    obj: &JsObject,
    display_fn: fn(&JsValue, &mut HashSet<usize>, usize, bool) -> String,
    indent: usize,
    encounters: &mut HashSet<usize>,
) -> Vec<String> {
    let object = obj.borrow();
    if let Some(object) = object.prototype() {
        vec![format!(
            "{:>width$}{}: {}",
            "",
            "__proto__",
            display_fn(
                &object.clone().into(),
                encounters,
                indent.wrapping_add(4),
                true
            ),
            width = indent,
        )]
    } else {
        vec![format!(
            "{:>width$}{}: {}",
            "",
            "__proto__",
            JsValue::null().display(),
            width = indent,
        )]
    }
}

fn print_obj_value_props(
    obj: &JsObject,
    display_fn: fn(&JsValue, &mut HashSet<usize>, usize, bool) -> String,
    indent: usize,
    encounters: &mut HashSet<usize>,
    print_internals: bool,
) -> Vec<String> {
    let mut keys: Vec<_> = obj
        .borrow()
        .properties()
        .index_property_keys()
        .map(PropertyKey::from)
        .collect();
    keys.extend(obj.borrow().properties().shape.keys());
    let mut result = Vec::default();
    for key in keys {
        let val = obj
            .borrow()
            .properties()
            .get(&key)
            .expect("There should be a value");
        if val.is_data_descriptor() {
            let v = &val.expect_value();
            result.push(format!(
                "{:>width$}{}: {}",
                "",
                key,
                display_fn(v, encounters, indent.wrapping_add(4), print_internals),
                width = indent,
            ));
        } else {
            let display = match (val.set().is_some(), val.get().is_some()) {
                (true, true) => "Getter & Setter",
                (true, false) => "Setter",
                (false, true) => "Getter",
                _ => "No Getter/Setter",
            };
            result.push(format!("{key:>indent$}: {display}"));
        }
    }
    result
}

pub(crate) fn log_string_from(x: &JsValue, print_internals: bool, print_children: bool) -> String {
    match x.variant() {
        // We don't want to print private (compiler) or prototype properties
        JsVariant::Object(v) => {
            // Can use the private "type" field of an Object to match on
            // which type of Object it represents for special printing
            if let Some(s) = v.downcast_ref::<JsString>() {
                format!("String {{ \"{}\" }}", s.to_std_string_escaped())
            } else if let Some(b) = v.downcast_ref::<bool>() {
                format!("Boolean {{ {b} }}")
            } else if let Some(r) = v.downcast_ref::<f64>() {
                if r.is_sign_negative() && *r == 0.0 {
                    "Number { -0 }".to_string()
                } else {
                    let mut buffer = ryu_js::Buffer::new();
                    format!("Number {{ {} }}", buffer.format(*r))
                }
            } else if v.is::<Array>() {
                let len = v
                    .borrow()
                    .properties()
                    .get(&js_string!("length").into())
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
            } else if let Some(map) = v.downcast_ref::<OrderedMap<JsValue>>() {
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
            } else if let Some(set) = v.downcast_ref::<OrderedSet>() {
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
            } else if v.is::<Error>() {
                let name: Cow<'static, str> = v
                    .get_property(&js_string!("name").into())
                    .as_ref()
                    .and_then(PropertyDescriptor::value)
                    .map_or_else(
                        || "<error>".into(),
                        |v| {
                            v.as_string()
                                .as_ref()
                                .map_or_else(
                                    || v.display().to_string(),
                                    JsString::to_std_string_escaped,
                                )
                                .into()
                        },
                    );
                let message = v
                    .get_property(&js_string!("message").into())
                    .as_ref()
                    .and_then(PropertyDescriptor::value)
                    .map(|v| {
                        v.as_string().as_ref().map_or_else(
                            || v.display().to_string(),
                            JsString::to_std_string_escaped,
                        )
                    })
                    .unwrap_or_default();
                let mut result = if name.is_empty() {
                    message
                } else if message.is_empty() {
                    name.to_string()
                } else {
                    format!("{name}: {message}")
                };
                let data = v
                    .downcast_ref::<Error>()
                    .expect("already checked object type");

                if let Some(position) = &data.position.0 {
                    write!(&mut result, "{position}").expect("should not fail");
                }
                result
            } else if let Some(promise) = v.downcast_ref::<Promise>() {
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
            } else if v.is_constructor() {
                // FIXME: ArrayBuffer is not [class ArrayBuffer] but we cannot distinguish it.
                let name = v
                    .get_property(&PropertyKey::from(js_string!("name")))
                    .and_then(|d| Some(d.value()?.as_string()?.to_std_string_escaped()));
                match name {
                    Some(name) => format!("[class {name}]"),
                    None => "[class (anonymous)]".to_string(),
                }
            } else if v.is_callable() {
                let name = v
                    .get_property(&PropertyKey::from(js_string!("name")))
                    .and_then(|d| Some(d.value()?.as_string()?.to_std_string_escaped()));
                match name {
                    Some(name) => format!("[Function: {name}]"),
                    None => "[Function (anonymous)]".to_string(),
                }
            } else {
                x.display_obj(print_internals)
            }
        }
        _ => x.display().to_string(),
    }
}

impl JsValue {
    /// A helper function for specifically printing object values
    #[must_use]
    pub fn display_obj(&self, print_internals: bool) -> String {
        // A simple helper for getting the address of a value
        // TODO: Find a more general place for this, as it can be used in other situations as well
        fn address_of<T: ?Sized>(t: &T) -> usize {
            let my_ptr: *const T = t;
            my_ptr.cast::<()>() as usize
        }

        fn display_obj_internal(
            data: &JsValue,
            encounters: &mut HashSet<usize>,
            indent: usize,
            print_internals: bool,
        ) -> String {
            if let Some(v) = data.as_object() {
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
                    print_obj_value_all(&v, display_obj_internal, indent, encounters).join(",\n")
                } else {
                    print_obj_value_props(
                        &v,
                        display_obj_internal,
                        indent,
                        encounters,
                        print_internals,
                    )
                    .join(",\n")
                };

                // If the current object is referenced in a different branch,
                // it will not cause an infinite printing loop, so it is safe to be printed again
                encounters.remove(&addr);

                let closing_indent = String::from_utf8(vec![b' '; indent.wrapping_sub(4)])
                    .expect("Could not create the closing brace's indentation string");

                let constructor_name = get_constructor_name_not_object(&v);
                let constructor_prefix = match constructor_name {
                    Some(name) => {
                        format!("{} ", name.to_std_string_lossy())
                    }
                    None => String::new(),
                };

                format!("{constructor_prefix}{{\n{result}\n{closing_indent}}}")
            } else {
                // Every other type of data is printed with the display method
                data.display().to_string()
            }
        }

        /// The constructor can be retrieved as `Object.getPrototypeOf(obj).constructor`.
        ///
        /// Also return `None` if constructor is `Object` as plain object does not need name.
        fn get_constructor_name_not_object(obj: &JsObject) -> Option<JsString> {
            let prototype = obj.prototype()?;

            // To neglect out plain object
            // `Object.getPrototypeOf(Object.prototype)` => null.
            // For user created `Object.create(Object.create(null))`,
            // we also don't need to display its name.
            prototype.prototype()?;

            let constructor_property = prototype
                .borrow()
                .properties()
                .get(&PropertyKey::from(js_string!("constructor")))?;
            let constructor = constructor_property.value()?;

            let name = constructor
                .as_object()?
                .borrow()
                .properties()
                .get(&PropertyKey::from(js_string!("name")))?
                .value()?
                .as_string()?;

            Some(name)
        }

        // We keep track of which objects we have encountered by keeping their
        // in-memory address in this set
        let mut encounters = HashSet::new();

        display_obj_internal(self, &mut encounters, 4, print_internals)
    }
}

impl Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value.variant() {
            JsVariant::Null => write!(f, "null"),
            JsVariant::Undefined => write!(f, "undefined"),
            JsVariant::Boolean(v) => write!(f, "{v}"),
            JsVariant::Symbol(symbol) => {
                write!(f, "{}", symbol.descriptive_string().to_std_string_escaped())
            }
            JsVariant::String(v) => write!(f, "\"{}\"", v.to_std_string_escaped()),
            JsVariant::Float64(v) => format_rational(v, f),
            JsVariant::Object(_) => {
                write!(f, "{}", log_string_from(self.value, self.internals, true))
            }
            JsVariant::Integer32(v) => write!(f, "{v}"),
            JsVariant::BigInt(num) => write!(f, "{num}n"),
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
