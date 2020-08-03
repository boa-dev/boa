use super::*;

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
        {
            let object = $obj.borrow();
            if object.prototype().is_object() {
                vec![format!(
                    "{:>width$}: {}",
                    "__proto__",
                    $display_fn(object.prototype(), $encounters, $indent.wrapping_add(4), true),
                    width = $indent,
                )]
            } else {
                vec![format!(
                    "{:>width$}: {}",
                    "__proto__",
                    object.prototype(),
                    width = $indent,
                )]
            }
        }
    };
    (props of $obj:expr, $display_fn:ident, $indent:expr, $encounters:expr, $print_internals:expr) => {
        print_obj_value!(impl properties, $obj, |(key, val)| {
            let v = &val
                .value
                .as_ref()
                .expect("Could not get the property's value");

            format!(
                "{:>width$}: {}",
                key,
                $display_fn(v, $encounters, $indent.wrapping_add(4), $print_internals),
                width = $indent,
            )
        })
    };

    // A private overload of the macro
    // DO NOT use directly
    (impl $field:ident, $v:expr, $f:expr) => {
        $v
            .borrow()
            .$field()
            .iter()
            .map($f)
            .collect::<Vec<String>>()
    };
}

pub(crate) fn log_string_from(x: &Value, print_internals: bool, print_children: bool) -> String {
    match x {
        // We don't want to print private (compiler) or prototype properties
        Value::Object(ref v) => {
            // Can use the private "type" field of an Object to match on
            // which type of Object it represents for special printing
            match v.borrow().data {
                ObjectData::String(ref string) => format!("String {{ \"{}\" }}", string),
                ObjectData::Boolean(boolean) => format!("Boolean {{ {} }}", boolean),
                ObjectData::Number(rational) => {
                    if rational.is_sign_negative() && rational == 0.0 {
                        "Number { -0 }".to_string()
                    } else {
                        let mut buffer = ryu_js::Buffer::new();
                        format!("Number {{ {} }}", buffer.format(rational))
                    }
                }
                ObjectData::Array => {
                    let len = i32::from(
                        &v.borrow()
                            .properties()
                            .get("length")
                            .expect("Could not get Array's length property")
                            .value
                            .clone()
                            .expect("Could not borrow value"),
                    );

                    if print_children {
                        if len == 0 {
                            return String::from("[]");
                        }

                        let arr = (0..len)
                            .map(|i| {
                                // Introduce recursive call to stringify any objects
                                // which are part of the Array
                                log_string_from(
                                    &v.borrow()
                                        .properties()
                                        .get(i.to_string().as_str())
                                        .unwrap()
                                        .value
                                        .clone()
                                        .expect("Could not borrow value"),
                                    print_internals,
                                    false,
                                )
                            })
                            .collect::<Vec<String>>()
                            .join(", ");

                        format!("[ {} ]", arr)
                    } else {
                        format!("Array({})", len)
                    }
                }
                ObjectData::Map(ref map) => {
                    let size = i32::from(
                        &v.borrow()
                            .properties()
                            .get("size")
                            .unwrap()
                            .value
                            .clone()
                            .expect("Could not borrow value"),
                    );
                    if size == 0 {
                        return String::from("Map(0)");
                    }

                    if print_children {
                        let mappings = map
                            .iter()
                            .map(|(key, value)| {
                                let key = log_string_from(key, print_internals, false);
                                let value = log_string_from(value, print_internals, false);
                                format!("{} → {}", key, value)
                            })
                            .collect::<Vec<String>>()
                            .join(", ");
                        format!("Map {{ {} }}", mappings)
                    } else {
                        format!("Map({})", size)
                    }
                }
                _ => display_obj(&x, print_internals),
            }
        }
        Value::Symbol(ref symbol) => symbol.to_string(),
        _ => format!("{}", x),
    }
}

/// A helper function for specifically printing object values
pub(crate) fn display_obj(v: &Value, print_internals: bool) -> String {
    // A simple helper for getting the address of a value
    // TODO: Find a more general place for this, as it can be used in other situations as well
    fn address_of<T>(t: &T) -> usize {
        let my_ptr: *const T = t;
        my_ptr as usize
    }

    // We keep track of which objects we have encountered by keeping their
    // in-memory address in this set
    let mut encounters = HashSet::new();

    if let Value::Object(object) = v {
        if object.borrow().is_error() {
            let name = v.get_field("name");
            let message = v.get_field("message");
            return format!("{}: {}", name, message);
        }
    }

    fn display_obj_internal(
        data: &Value,
        encounters: &mut HashSet<usize>,
        indent: usize,
        print_internals: bool,
    ) -> String {
        if let Value::Object(ref v) = *data {
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
            // it will not cause an infinte printing loop, so it is safe to be printed again
            encounters.remove(&addr);

            let closing_indent = String::from_utf8(vec![b' '; indent.wrapping_sub(4)])
                .expect("Could not create the closing brace's indentation string");

            format!("{{\n{}\n{}}}", result, closing_indent)
        } else {
            // Every other type of data is printed with the display method
            format!("{}", data)
        }
    }

    display_obj_internal(v, &mut encounters, 4, print_internals)
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Symbol(ref symbol) => match symbol.description() {
                Some(description) => write!(f, "Symbol({})", description),
                None => write!(f, "Symbol()"),
            },
            Self::String(ref v) => write!(f, "\"{}\"", v),
            Self::Rational(v) => format_rational(*v, f),
            Self::Object(_) => write!(f, "{}", log_string_from(self, true, true)),
            Self::Integer(v) => write!(f, "{}", v),
            Self::BigInt(ref num) => write!(f, "{}n", num),
        }
    }
}

/// This is different from the ECMAScript compliant number to string, in the printing of `-0`.
///
/// This function prints `-0` as `-0` instead of pasitive `0` as the specification says.
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
