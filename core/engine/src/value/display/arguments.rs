use crate::{JsObject, JsValue, js_string};
use std::collections::HashSet;
use std::fmt::{self, Write};

/// Formats an Arguments exotic object for display.
///
/// Always uses multiline output:
/// ```text
/// [Arguments] {
///   0: "first",
///   1: 42
/// }
/// ```
pub(super) fn log_arguments_to(
    f: &mut fmt::Formatter<'_>,
    x: &JsObject,
    print_internals: bool,
    print_children: bool,
) -> fmt::Result {
    let len = x
        .borrow()
        .properties()
        .get(&js_string!("length").into())
        .and_then(|d| d.value().cloned())
        .and_then(|v| v.as_number())
        .map(|n| n as i32)
        .unwrap_or(0);

    if !print_children {
        return write!(f, "Arguments({len})");
    }

    if len == 0 {
        return f.write_str("[Arguments] {}");
    }

    f.write_str("[Arguments] {\n")?;
    for i in 0..len {
        // FIXME: handle accessor descriptors
        let val = x
            .borrow()
            .properties()
            .get(&i.into())
            .and_then(|d| d.value().cloned());

        let val_str = match val {
            Some(v) => format!(
                "{}",
                CompactValue { value: &v, print_internals }
            ),
            None => "<empty>".to_string(),
        };

        write!(f, "  {i}: {val_str}")?;
        if i + 1 < len {
            f.write_char(',')?;
        }
        f.write_char('\n')?;
    }
    f.write_str("}")
}

struct CompactValue<'a> {
    value: &'a JsValue,
    print_internals: bool,
}

impl fmt::Display for CompactValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::log_value_compact(f, self.value, 0, self.print_internals, &mut HashSet::new())
    }
}
