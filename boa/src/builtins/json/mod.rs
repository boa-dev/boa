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

use super::value::ValueData;
use crate::builtins::value::{ResultValue, Value};
use crate::exec::{Executor, Interpreter};
use crate::syntax::ast::node::Node;
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
pub fn parse(this: &mut Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    match serde_json::from_str::<JSONValue>(
        &args
            .get(0)
            .expect("cannot get argument for JSON.parse")
            .clone()
            .to_string(),
    ) {
        Ok(json) => {
            let j = Value::from(json);
            if args.len() > 1 {
                let result = match args.get(1) {
                    Some(callback) => {
                        if callback.is_function() {
                            let mut holder = Value::new_object(None);
                            holder.set_field(Value::from(""), j.clone());
                            println!("Value is {}", holder.get_field(Value::from("")));
                            //TODO abhi: check if this arg exists just like `every` in the array module
                            //let mut this_arg = args[2].clone();
                            walk(callback, interpreter, &mut holder, Value::from(""))
                        } else {
                            Ok(j)
                        }
                    }
                    _ => Ok(j),
                };
                result
            } else {
                Ok(j)
            }
            //Ok(j)
            /*    let callback = args.get(1);
                let callback_result = interpreter
                    .call(callback, this: &mut Value, arguments_list: &[Value])
                    .unwrap_or_else(|_| Value::undefined());
                if callback_result.is_true() {
                    Some(element)
                } else {
                    None
                }
            } else {
                Ok(Value::from(json))
            }*/
        }
        Err(err) => Err(Value::from(err.to_string())),
    }
}

fn walk(
    callback: &Value,
    interpreter: &mut Interpreter,
    holder: &mut Value,
    key: Value,
) -> ResultValue {
    let mut value = holder.get_field(key.clone());

    if value.get_type() == "object" {
        let obj = value.as_object().unwrap().clone();
        for (key, _val) in obj.properties.iter() {
            let v = walk(callback, interpreter, &mut value, Value::from(key.as_str()));
            match v {
                Ok(v) => {
                    println!("Ok {}", v);
                    if !v.is_undefined() {
                        value.set_field(Value::from(key.as_str()), v);
                    } else {
                        value.remove_property(key.as_str());
                    }
                }
                Err(v) => {
                    println!("Err {}", v);
                }
            }
        }
    }
    let arguments = [key, value];
    interpreter.call(callback, holder, &arguments)
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
