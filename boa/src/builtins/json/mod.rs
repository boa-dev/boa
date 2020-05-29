//! This module implements the global `JSON` object.
//!
//! The `JSON` object contains methods for parsing [JavaScript Object Notation (JSON)][spec]
//! and converting values to JSON. It can't be called or constructed, and aside from its
//! two method properties, it has no interesting functionality of its own.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!  - [JSON specification][json]
//!
//! [spec]: https://tc39.es/ecma262/#sec-json
//! [json]: https://www.json.org/json-en.html
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON

use crate::builtins::{
    function::make_builtin_fn,
    object::ObjectKind,
    property::Property,
    value::{ResultValue, Value},
};
use crate::exec::Interpreter;
use serde_json::{self, Value as JSONValue};

#[cfg(test)]
mod tests;

/// `JSON.parse( text[, reviver] )`
///
/// This `JSON` method parses a JSON string, constructing the JavaScript value or object described by the string.
///
/// An optional `reviver` function can be provided to perform a transformation on the resulting object before it is returned.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-json.parse
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse
// TODO: implement optional revever argument.
pub fn parse(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    match serde_json::from_str::<JSONValue>(
        &args
            .get(0)
            .expect("cannot get argument for JSON.parse")
            .clone()
            .to_string(),
    ) {
        Ok(json) => Ok(Value::from(json)),
        Err(err) => Err(Value::from(err.to_string())),
    }
}

/// `JSON.stringify( value[, replacer[, space]] )`
///
/// This `JSON` method converts a JavaScript object or value to a JSON string.
///
/// This medhod optionally replaces values if a `replacer` function is specified or
/// optionally including only the specified properties if a replacer array is specified.
///
/// An optional `space` argument can be supplied of type `String` or `Number` that's used to insert
/// white space into the output JSON string for readability purposes.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-json.stringify
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify
pub fn stringify(_: &mut Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    let object = match args.get(0) {
        Some(obj) if obj.is_symbol() || obj.is_function() => return Ok(Value::undefined()),
        None => return Ok(Value::undefined()),
        Some(obj) => obj,
    };
    let replacer = match args.get(1) {
        Some(replacer) if replacer.is_object() => replacer,
        _ => return Ok(Value::from(object.to_json().to_string())),
    };

    let replacer_as_object = replacer
        .as_object()
        .expect("JSON.stringify replacer was an object");
    if replacer_as_object.is_callable() {
        object
            .as_object()
            .map(|obj| {
                let object_to_return = Value::new_object(None);
                for (key, val) in obj.properties.iter().filter_map(|(key, v)| {
                    if let Some(value) = &v.value {
                        Some((key, value))
                    } else {
                        None
                    }
                }) {
                    let mut this_arg = object.clone();
                    object_to_return.set_property(
                        key.to_owned(),
                        Property::default().value(interpreter.call(
                            replacer,
                            &mut this_arg,
                            &[Value::string(key), val.clone()],
                        )?),
                    );
                }
                Ok(Value::from(object_to_return.to_json().to_string()))
            })
            .ok_or_else(Value::undefined)?
    } else if replacer_as_object.kind == ObjectKind::Array {
        let mut obj_to_return =
            serde_json::Map::with_capacity(replacer_as_object.properties.len() - 1);
        let fields = replacer_as_object.properties.keys().filter_map(|key| {
            if key == "length" {
                None
            } else {
                Some(replacer.get_field(key.to_string()))
            }
        });
        for field in fields {
            if let Some(value) = object
                .get_property(&field.to_string())
                .map(|prop| prop.value.as_ref().map(|v| v.to_json()))
                .flatten()
            {
                obj_to_return.insert(field.to_string(), value);
            }
        }
        Ok(Value::from(JSONValue::Object(obj_to_return).to_string()))
    } else {
        Ok(Value::from(object.to_json().to_string()))
    }
}

/// Create a new `JSON` object.
pub fn create(global: &Value) -> Value {
    let json = Value::new_object(Some(global));

    make_builtin_fn(parse, "parse", &json, 2);
    make_builtin_fn(stringify, "stringify", &json, 3);

    json
}

/// Initialise the `JSON` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field("JSON", create(global));
}
