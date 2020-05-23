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

use crate::builtins::value::{ResultValue, Value};
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
pub fn parse(_: &mut Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    match serde_json::from_str::<JSONValue>(
        &args
            .get(0)
            .expect("cannot get argument for JSON.parse")
            .clone()
            .to_string(),
    ) {
        Ok(json) => {
            let j = Value::from(json);
            match args.get(1) {
                Some(reviver) if reviver.is_function() => {
                    let mut holder = Value::new_object(None);
                    holder.set_field(Value::from(""), j);
                    walk(reviver, interpreter, &mut holder, Value::from(""))
                }
                _ => Ok(j),
            }
        }
        Err(err) => Err(Value::from(err.to_string())),
    }
}

/// This is a translation of the [Polyfill implementation][polyfill]
/// This function recursively walks the structure. passing each key-value pair to the reviver function
/// for possible transformation
/// [polyfill]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse
fn walk(
    reviver: &Value,
    interpreter: &mut Interpreter,
    holder: &mut Value,
    key: Value,
) -> ResultValue {
    let mut value = holder.get_field(key.clone());

    let obj = value.as_object().as_deref().cloned();
    if let Some(obj) = obj {
        for key in obj.properties.keys() {
            let v = walk(reviver, interpreter, &mut value, Value::from(key.as_str()));
            match v {
                Ok(v) if !v.is_undefined() => {
                    value.set_field(Value::from(key.as_str()), v);
                }
                Ok(_) => {
                    value.remove_property(key.as_str());
                }
                Err(_v) => {}
            }
        }
    }
    interpreter.call(reviver, holder, &[key, value])
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
pub fn stringify(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("cannot get argument for JSON.stringify");
    let json = obj.to_json().to_string();
    Ok(Value::from(json))
}

/// Create a new `JSON` object.
pub fn create(global: &Value) -> Value {
    let json = Value::new_object(Some(global));

    make_builtin_fn!(parse, named "parse", with length 2, of json);
    make_builtin_fn!(stringify, named "stringify", with length 3, of json);

    json
}

/// Initialise the `JSON` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field_slice("JSON", create(global));
}
