use std::collections::HashSet;
use std::fmt;

use crate::{JsObject, JsValue, js_string, property::DescriptorKind};

pub(super) fn log_array_to(
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

            if let Some(desc) = x.borrow().properties().get(&i.into()) {
                match desc.kind() {
                    DescriptorKind::Data { value, .. } => {
                        if let Some(value) = value {
                            super::value::log_value_to(f, value, print_internals, false)?;
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
                        unreachable!("found generic descriptor in array")
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

pub(super) fn log_array_compact(
    f: &mut fmt::Formatter<'_>,
    x: &JsObject,
    depth: u32,
    print_internals: bool,
    encounters: &mut HashSet<usize>,
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
        if let Some(value) = x
            .borrow()
            .properties()
            .get(&i.into())
            .and_then(|x| x.value().cloned())
        {
            super::value::log_value_compact(f, &value, depth + 1, print_internals, encounters)?;
        } else {
            f.write_str("<empty>")?;
        }
    }
    f.write_str(" ]")
}
