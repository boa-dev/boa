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

use crate::{
    builtins::BuiltIn,
    object::ObjectInitializer,
    property::{Attribute, DataDescriptor, PropertyKey},
    BoaProfiler, Context, Result, Value,
};
use serde::Serialize;
use serde_json::{self, ser::PrettyFormatter, Serializer, Value as JSONValue};

#[cfg(test)]
mod tests;

/// JavaScript `JSON` global object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Json;

impl BuiltIn for Json {
    const NAME: &'static str = "JSON";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let json_object = ObjectInitializer::new(context)
            .function(Self::parse, "parse", 2)
            .function(Self::stringify, "stringify", 3)
            .build();

        (Self::NAME, json_object.into(), Self::attribute())
    }
}

impl Json {
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
    pub(crate) fn parse(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let arg = args
            .get(0)
            .cloned()
            .unwrap_or_else(Value::undefined)
            .to_string(context)?;

        match serde_json::from_str::<JSONValue>(&arg) {
            Ok(json) => {
                let j = Value::from_json(json, context);
                match args.get(1) {
                    Some(reviver) if reviver.is_function() => {
                        let mut holder = Value::new_object(None);
                        holder.set_field("", j);
                        Self::walk(reviver, context, &mut holder, &PropertyKey::from(""))
                    }
                    _ => Ok(j),
                }
            }
            Err(err) => context.throw_syntax_error(err.to_string()),
        }
    }

    /// This is a translation of the [Polyfill implementation][polyfill]
    ///
    /// This function recursively walks the structure, passing each key-value pair to the reviver function
    /// for possible transformation.
    ///
    /// [polyfill]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse
    fn walk(
        reviver: &Value,
        context: &mut Context,
        holder: &mut Value,
        key: &PropertyKey,
    ) -> Result<Value> {
        let value = holder.get_field(key.clone());

        if let Value::Object(ref object) = value {
            let keys: Vec<_> = object.borrow().keys().collect();

            for key in keys {
                let v = Self::walk(reviver, context, &mut value.clone(), &key);
                match v {
                    Ok(v) if !v.is_undefined() => {
                        value.set_field(key, v);
                    }
                    Ok(_) => {
                        value.remove_property(key);
                    }
                    Err(_v) => {}
                }
            }
        }
        context.call(reviver, holder, &[key.into(), value])
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
    pub(crate) fn stringify(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let object = match args.get(0) {
            Some(obj) if obj.is_symbol() || obj.is_function() || obj.is_undefined() => {
                return Ok(Value::undefined())
            }
            None => return Ok(Value::undefined()),
            Some(obj) => obj,
        };
        let replacer = match args.get(1) {
            Some(replacer) if replacer.is_object() => replacer,
            _ => return Ok(Value::from(object.to_json(context)?.to_string())),
        };
        const SPACE_INDENT: &str = "          ";
        let space = match args.get(2) {
            Some(indent) if indent.is_number() => {
                if let Some(indent_length) = indent.as_number() {
                    let indent_length = if indent_length > 10.0 {
                        10.0
                    } else {
                        indent_length
                    };
                    let indent_length = if indent_length < 0.0 || indent_length.is_nan() {
                        0.0
                    } else {
                        indent_length
                    };
                    &SPACE_INDENT[..indent_length as usize]
                } else {
                    ""
                }
            }
            Some(space) if space.is_string() => {
                if let Some(space) = space.as_string() {
                    &space[..std::cmp::min(space.len(), 10)]
                } else {
                    ""
                }
            }
            _ => "",
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
                        .borrow()
                        .iter()
                        // FIXME: handle accessor descriptors
                        .map(|(k, v)| (k, v.as_data_descriptor().unwrap().value()))
                    {
                        let this_arg = object.clone();
                        object_to_return.set_property(
                            key.to_owned(),
                            DataDescriptor::new(
                                context.call(
                                    replacer,
                                    &this_arg,
                                    &[Value::from(key.clone()), val.clone()],
                                )?,
                                Attribute::all(),
                            ),
                        );
                    }
                    Ok(Value::from(json_to_pretty_string(
                        &object_to_return.to_json(context)?,
                        space,
                    )))
                })
                .ok_or_else(Value::undefined)?
        } else if replacer_as_object.is_array() {
            let mut obj_to_return = serde_json::Map::new();
            let replacer_as_object = replacer_as_object.borrow();
            let fields = replacer_as_object.keys().filter_map(|key| {
                if key == "length" {
                    None
                } else {
                    Some(replacer.get_field(key))
                }
            });
            for field in fields {
                if let Some(value) = object
                    .get_property(field.to_string(context)?)
                    // FIXME: handle accessor descriptors
                    .map(|prop| prop.as_data_descriptor().unwrap().value().to_json(context))
                    .transpose()?
                {
                    obj_to_return.insert(field.to_string(context)?.to_string(), value);
                }
            }
            Ok(Value::from(json_to_pretty_string(
                &JSONValue::Object(obj_to_return),
                space,
            )))
        } else {
            Ok(Value::from(json_to_pretty_string(
                &object.to_json(context)?,
                space,
            )))
        }
    }
}

fn json_to_pretty_string(json: &JSONValue, space: &str) -> String {
    if space.len() == 0 {
        return json.to_string();
    }
    let formatter = PrettyFormatter::with_indent(space.as_bytes());
    let mut writer = Vec::with_capacity(128);
    let mut serializer = Serializer::with_formatter(&mut writer, formatter);
    json.serialize(&mut serializer)
        .expect("JSON serialization failed");
    unsafe {
        // The serde json serializer always produce correct UTF-8
        String::from_utf8_unchecked(writer)
    }
}
