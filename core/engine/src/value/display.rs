use super::{Display, HashSet, JsValue, JsVariant, fmt};
use crate::{
    JsError, JsObject, JsString,
    builtins::{
        Array, Promise, error::Error, map::ordered_map::OrderedMap, promise::PromiseState,
        set::ordered_set::OrderedSet,
    },
    js_string,
    property::{DescriptorKind, PropertyDescriptor, PropertyKey},
};
use std::borrow::Cow;
use std::fmt::Write;

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

fn print_obj_value_internals(
    f: &mut fmt::Formatter<'_>,
    obj: &JsObject,
    indent: usize,
    encounters: &mut HashSet<usize>,
) -> fmt::Result {
    let object = obj.borrow();
    write!(f, "{:>indent$}__proto__: ", "")?;
    if let Some(object) = object.prototype() {
        log_object_to_internal(
            f,
            &object.clone().into(),
            encounters,
            indent.wrapping_add(4),
            true,
        )?;
    } else {
        write!(f, "{}", JsValue::null().display())?;
    }
    f.write_char(',')?;
    f.write_char('\n')
}

fn print_obj_value_props(
    f: &mut fmt::Formatter<'_>,
    obj: &JsObject,
    indent: usize,
    encounters: &mut HashSet<usize>,
    print_internals: bool,
) -> fmt::Result {
    let mut keys: Vec<_> = obj
        .borrow()
        .properties()
        .index_property_keys()
        .map(PropertyKey::from)
        .collect();
    keys.extend(obj.borrow().properties().shape.keys());

    let mut first = true;
    for key in keys {
        if first {
            first = false;
        } else {
            f.write_char(',')?;
            f.write_char('\n')?;
        }
        let val = obj
            .borrow()
            .properties()
            .get(&key)
            .expect("There should be a value");

        write!(f, "{:>width$}{}: ", "", key, width = indent)?;
        if val.is_data_descriptor() {
            let v = val.expect_value();
            log_object_to_internal(f, v, encounters, indent.wrapping_add(4), print_internals)?;
        } else {
            let display = match (val.set().is_some(), val.get().is_some()) {
                (true, true) => "Getter & Setter",
                (true, false) => "Setter",
                (false, true) => "Getter",
                _ => "No Getter/Setter",
            };
            write!(f, "{key}: {display}")?;
        }
    }
    f.write_char('\n')?;
    Ok(())
}

fn log_array_to(
    f: &mut fmt::Formatter<'_>,
    x: &JsObject,
    print_internals: bool,
    print_children: bool,
) -> fmt::Result {
    let len = x
        .borrow()
        .properties()
        .get(&js_string!("length").into())
        .expect("array object must have 'length' property")
        .value()
        .and_then(JsValue::as_number)
        .map(|n| n as i32)
        .unwrap_or_default();

    if print_children {
        if len == 0 {
            return f.write_str("[]");
        }

        f.write_str("[ ")?;
        let mut first = true;
        for i in 0..len {
            if first {
                first = false;
            } else {
                f.write_str(", ")?;
            }

            // Introduce recursive call to stringify any objects
            // which are part of the Array

            if let Some(desc) = x.borrow().properties().get(&i.into()) {
                match desc.kind() {
                    DescriptorKind::Data { value, .. } => {
                        if let Some(value) = value {
                            log_value_to(f, value, print_internals, false)?;
                        } else {
                            f.write_str("undefined")?;
                        }
                    }
                    DescriptorKind::Accessor { get, set } => {
                        let display = match (get.is_some(), set.is_some()) {
                            (true, true) => "[Getter/Setter]",
                            (true, false) => "[Getter]",
                            (false, true) => "[Setter]",
                            _ => "<empty>",
                        };
                        f.write_str(display)?;
                    }
                    DescriptorKind::Generic => {
                        f.write_str("undefined")?;
                    }
                }
            } else {
                f.write_str("<empty>")?;
            }
        }
        write!(f, " ]")
    } else {
        write!(f, "Array({len})")
    }
}

pub(crate) fn log_value_to(
    f: &mut fmt::Formatter<'_>,
    x: &JsValue,
    print_internals: bool,
    print_children: bool,
) -> fmt::Result {
    match x.variant() {
        // We don't want to print private (compiler) or prototype properties
        JsVariant::Object(v) => {
            // Can use the private "type" field of an Object to match on
            // which type of Object it represents for special printing
            if let Some(s) = v.downcast_ref::<JsString>() {
                write!(f, "String {{ {:?} }}", s.to_std_string_escaped())
            } else if let Some(b) = v.downcast_ref::<bool>() {
                write!(f, "Boolean {{ {b} }}")
            } else if let Some(r) = v.downcast_ref::<f64>() {
                f.write_str("Number { ")?;
                format_rational(*r, f)?;
                f.write_str(" }")
            } else if v.is::<Array>() {
                log_array_to(f, &v, print_internals, print_children)
            } else if let Some(map) = v.downcast_ref::<OrderedMap<JsValue>>() {
                let size = map.len();
                if size == 0 {
                    return f.write_str("Map(0)");
                }

                if print_children {
                    f.write_str("Map { ")?;
                    let mut first = true;
                    for (key, value) in map.iter() {
                        if first {
                            first = false;
                        } else {
                            f.write_str(", ")?;
                        }
                        log_value_to(f, key, print_internals, false)?;
                        f.write_str(" → ")?;
                        log_value_to(f, value, print_internals, false)?;
                    }
                    f.write_str(" }")
                } else {
                    write!(f, "Map({size})")
                }
            } else if let Some(set) = v.downcast_ref::<OrderedSet>() {
                let size = set.len();

                if size == 0 {
                    return f.write_str("Set(0)");
                }

                if print_children {
                    f.write_str("Set { ")?;
                    let mut first = true;
                    for value in set.iter() {
                        if first {
                            first = false;
                        } else {
                            f.write_str(", ")?;
                        }
                        log_value_to(f, value, print_internals, false)?;
                    }
                    f.write_str(" }")
                } else {
                    write!(f, "Set({size})")
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
                if name.is_empty() {
                    f.write_str(&message)?;
                } else if message.is_empty() {
                    f.write_str(name.as_ref())?;
                } else {
                    write!(f, "{name}: {message}")?;
                }
                let data = v
                    .downcast_ref::<Error>()
                    .expect("already checked object type");

                if let Some(position) = &data.position.0 {
                    write!(f, "{position}")?;
                }
                Ok(())
            } else if let Some(promise) = v.downcast_ref::<Promise>() {
                f.write_str("Promise { ")?;
                match promise.state() {
                    PromiseState::Pending => f.write_str("<pending>")?,
                    PromiseState::Fulfilled(val) => Display::fmt(&val.display(), f)?,
                    PromiseState::Rejected(reason) => {
                        write!(f, "<rejected> {}", JsError::from_opaque(reason.clone()))?;
                    }
                }
                f.write_str(" }")
            } else if v.is_constructor() {
                // FIXME: ArrayBuffer is not [class ArrayBuffer] but we cannot distinguish it.
                let name = v
                    .get_property(&PropertyKey::from(js_string!("name")))
                    .and_then(|d| Some(d.value()?.as_string()?.to_std_string_escaped()));
                match name {
                    Some(name) => write!(f, "[class {name}]"),
                    None => f.write_str("[class (anonymous)]"),
                }
            } else if v.is_callable() {
                let name = v
                    .get_property(&PropertyKey::from(js_string!("name")))
                    .and_then(|d| Some(d.value()?.as_string()?.to_std_string_escaped()));
                match name {
                    Some(name) => write!(f, "[Function: {name}]"),
                    None => f.write_str("[Function (anonymous)]"),
                }
            } else {
                Display::fmt(&x.display_obj(print_internals), f)
            }
        }
        JsVariant::Null => write!(f, "null"),
        JsVariant::Undefined => write!(f, "undefined"),
        JsVariant::Boolean(v) => write!(f, "{v}"),
        JsVariant::Symbol(symbol) => {
            write!(f, "{}", symbol.descriptive_string().to_std_string_escaped())
        }
        JsVariant::String(v) => write!(f, "{:?}", v.to_std_string_escaped()),
        JsVariant::Float64(v) => format_rational(v, f),
        JsVariant::Integer32(v) => write!(f, "{v}"),
        JsVariant::BigInt(num) => write!(f, "{num}n"),
    }
}

fn log_object_to_internal(
    f: &mut fmt::Formatter<'_>,
    data: &JsValue,
    encounters: &mut HashSet<usize>,
    indent: usize,
    print_internals: bool,
) -> fmt::Result {
    if let Some(v) = data.as_object() {
        // The in-memory address of the current object
        let addr = std::ptr::from_ref(v.as_ref()).addr();

        // We need not continue if this object has already been
        // printed up the current chain
        if encounters.contains(&addr) {
            return f.write_str("[Cycle]");
        }

        // Mark the current object as encountered
        encounters.insert(addr);

        if v.is::<Array>() {
            return log_array_to(f, &v, print_internals, false);
        }

        let constructor_name = get_constructor_name_of(&v);
        if let Some(name) = constructor_name {
            write!(f, "{} ", name.to_std_string_lossy())?;
        }
        f.write_str("{\n")?;

        if print_internals {
            print_obj_value_internals(f, &v, indent, encounters)?;
        }
        print_obj_value_props(f, &v, indent, encounters, print_internals)?;
        write!(f, "{:>indent$}}}", "", indent = indent.saturating_sub(4))?;

        // If the current object is referenced in a different branch,
        // it will not cause an infinite printing loop, so it is safe to be printed again
        encounters.remove(&addr);
        Ok(())
    } else {
        // Every other type of data is printed with the display method
        log_value_to(f, data, print_internals, false)
    }
}

/// The constructor can be retrieved as `Object.getPrototypeOf(obj).constructor`.
///
/// Returns `None` if the constructor is `Object` as plain objects don't need a name.
fn get_constructor_name_of(obj: &JsObject) -> Option<JsString> {
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

impl JsValue {
    /// A helper function for specifically printing object values
    #[must_use]
    pub fn display_obj(&self, print_internals: bool) -> String {
        struct DisplayObj<'a>(&'a JsValue, bool);
        impl Display for DisplayObj<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                log_object_to_internal(f, self.0, &mut HashSet::new(), 4, self.1)
            }
        }

        DisplayObj(self, print_internals).to_string()
    }
}

impl Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        log_value_to(f, self.value, self.internals, true)
    }
}

/// This is different from the ECMAScript compliant number to string, in the printing of `-0`.
///
/// This function prints `-0` as `-0` instead of positive `0` as the specification says.
/// This is done to make it easier for the user of the REPL to identify what is a `-0` vs `0`,
/// since the REPL is not bound to the ECMAScript specification, we can do this.
fn format_rational(v: f64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if v.is_sign_negative() && v == 0.0 {
        f.write_str("-0")
    } else {
        let mut buffer = ryu_js::Buffer::new();
        f.write_str(buffer.format(v))
    }
}
