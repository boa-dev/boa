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
    property::Property,
    value::{ResultValue, Value},
};
use crate::{exec::Interpreter, BoaProfiler};
use serde_json::{self, Value as JSONValue};

#[cfg(test)]
mod tests;

/// JavaScript `JSON` global object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Json;

impl Json {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "JSON";

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
    pub(crate) fn parse(_: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        match serde_json::from_str::<JSONValue>(
            &ctx.to_string(args.get(0).expect("cannot get argument for JSON.parse"))?,
        ) {
            Ok(json) => {
                let j = Value::from_json(json, ctx);
                match args.get(1) {
                    Some(reviver) if reviver.is_function() => {
                        let mut holder = Value::new_object(None);
                        holder.set_field(Value::from(""), j);
                        Self::walk(reviver, ctx, &mut holder, Value::from(""))
                    }
                    _ => Ok(j),
                }
            }
            Err(err) => Err(Value::from(err.to_string())),
        }
    }

    /// This is a translation of the [Polyfill implementation][polyfill]
    ///
    /// This function recursively walks the structure, passing each key-value pair to the reviver function
    /// for possible transformation.
    ///
    /// [polyfill]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse
    fn walk(reviver: &Value, ctx: &mut Interpreter, holder: &mut Value, key: Value) -> ResultValue {
        let mut value = holder.get_field(key.clone());

        let obj = value.as_object().as_deref().cloned();
        if let Some(obj) = obj {
            for key in obj.properties().keys() {
                let v = Self::walk(reviver, ctx, &mut value, Value::from(key.as_str()));
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
        ctx.call(reviver, holder, &[key, value])
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
    pub(crate) fn stringify(_: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let object = match args.get(0) {
            Some(obj) if obj.is_symbol() || obj.is_function() => return Ok(Value::undefined()),
            None => return Ok(Value::undefined()),
            Some(obj) => obj,
        };
        let replacer = match args.get(1) {
            Some(replacer) if replacer.is_object() => replacer,
            _ => return Ok(Value::from(object.to_json(ctx)?.to_string())),
        };

        let replacer_as_object = replacer
            .as_object()
            .expect("JSON.stringify replacer was an object");
        if replacer_as_object.is_callable() {
            object
                .as_object()
                .map(|obj| {
                    let object_to_return = Value::new_object(None);
                    for (key, val) in obj
                        .properties()
                        .iter()
                        .filter_map(|(k, v)| v.value.as_ref().map(|value| (k, value)))
                    {
                        let mut this_arg = object.clone();
                        object_to_return.set_property(
                            key.to_owned(),
                            Property::default().value(ctx.call(
                                replacer,
                                &mut this_arg,
                                &[Value::string(key), val.clone()],
                            )?),
                        );
                    }
                    Ok(Value::from(object_to_return.to_json(ctx)?.to_string()))
                })
                .ok_or_else(Value::undefined)?
        } else if replacer_as_object.is_array() {
            let mut obj_to_return =
                serde_json::Map::with_capacity(replacer_as_object.properties().len() - 1);
            let fields = replacer_as_object.properties().keys().filter_map(|key| {
                if key == "length" {
                    None
                } else {
                    Some(replacer.get_field(key.to_owned()))
                }
            });
            for field in fields {
                if let Some(value) = object
                    .get_property(&ctx.to_string(&field)?)
                    .and_then(|prop| prop.value.as_ref().map(|v| v.to_json(ctx)))
                    .transpose()?
                {
                    obj_to_return.insert(field.to_string(), value);
                }
            }
            Ok(Value::from(JSONValue::Object(obj_to_return).to_string()))
        } else {
            Ok(Value::from(object.to_json(ctx)?.to_string()))
        }
    }

    /// Create a new `JSON` object.
    pub(crate) fn create(global: &Value) -> Value {
        let json = Value::new_object(Some(global));

        make_builtin_fn(Self::parse, "parse", &json, 2);
        make_builtin_fn(Self::stringify, "stringify", &json, 3);

        json
    }

    /// Initialise the `JSON` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        (Self::NAME, Self::create(global))
    }
}
