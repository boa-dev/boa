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
    object::Object,
    object::ObjectInitializer,
    property::{Attribute, PropertyDescriptor, PropertyKey},
    symbol::WellKnownSymbols,
    value::IntegerOrInfinity,
    BoaProfiler, Context, JsResult, JsValue,
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

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;

        let json_object = ObjectInitializer::new(context)
            .function(Self::parse, "parse", 2)
            .function(Self::stringify, "stringify", 3)
            .property(to_string_tag, Self::NAME, attribute)
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
    pub(crate) fn parse(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let arg = args
            .get(0)
            .cloned()
            .unwrap_or_else(JsValue::undefined)
            .to_string(context)?;

        match serde_json::from_str::<JSONValue>(&arg) {
            Ok(json) => {
                let j = JsValue::from_json(json, context);
                match args.get(1) {
                    Some(reviver) if reviver.is_function() => {
                        let mut holder: JsValue = context.construct_object().into();
                        holder.set_field("", j, true, context)?;
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
        reviver: &JsValue,
        context: &mut Context,
        holder: &mut JsValue,
        key: &PropertyKey,
    ) -> JsResult<JsValue> {
        let value = holder.get_field(key.clone(), context)?;

        if let JsValue::Object(ref object) = value {
            let keys: Vec<_> = object.borrow().properties().keys().collect();

            for key in keys {
                let v = Self::walk(reviver, context, &mut value.clone(), &key);
                match v {
                    Ok(v) if !v.is_undefined() => {
                        value.set_field(key, v, false, context)?;
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
    pub(crate) fn stringify(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = match args.get(0) {
            None => return Ok(JsValue::undefined()),
            Some(obj) => obj,
        };
        const SPACE_INDENT: &str = "          ";
        let gap = if let Some(space) = args.get(2) {
            let space = if let Some(space_obj) = space.as_object() {
                if let Some(space) = space_obj.borrow().as_number() {
                    JsValue::new(space)
                } else if let Some(space) = space_obj.borrow().as_string() {
                    JsValue::new(space)
                } else {
                    space.clone()
                }
            } else {
                space.clone()
            };
            if space.is_number() {
                let space_mv = match space.to_integer_or_infinity(context)? {
                    IntegerOrInfinity::NegativeInfinity => 0,
                    IntegerOrInfinity::PositiveInfinity => 10,
                    IntegerOrInfinity::Integer(i) if i < 1 => 0,
                    IntegerOrInfinity::Integer(i) => std::cmp::min(i, 10) as usize,
                };
                JsValue::new(&SPACE_INDENT[..space_mv])
            } else if let Some(string) = space.as_string() {
                JsValue::new(&string[..std::cmp::min(string.len(), 10)])
            } else {
                JsValue::new("")
            }
        } else {
            JsValue::new("")
        };

        let gap = &gap.to_string(context)?;

        let replacer = match args.get(1) {
            Some(replacer) if replacer.is_object() => replacer,
            _ => {
                if let Some(value) = object.to_json(context)? {
                    return Ok(JsValue::new(json_to_pretty_string(&value, gap)));
                } else {
                    return Ok(JsValue::undefined());
                }
            }
        };

        let replacer_as_object = replacer
            .as_object()
            .expect("JSON.stringify replacer was an object");
        if replacer_as_object.is_callable() {
            object
                .as_object()
                .map(|obj| {
                    let object_to_return = JsValue::new(Object::default());
                    for key in obj.borrow().properties().keys() {
                        let val = obj.__get__(&key, obj.clone().into(), context)?;
                        let this_arg = object.clone();
                        object_to_return.set_property(
                            key.to_owned(),
                            PropertyDescriptor::builder()
                                .value(context.call(
                                    replacer,
                                    &this_arg,
                                    &[JsValue::new(key.clone()), val.clone()],
                                )?)
                                .writable(true)
                                .enumerable(true)
                                .configurable(true),
                        )
                    }
                    if let Some(value) = object_to_return.to_json(context)? {
                        Ok(JsValue::new(json_to_pretty_string(&value, gap)))
                    } else {
                        Ok(JsValue::undefined())
                    }
                })
                .ok_or_else(JsValue::undefined)?
        } else if replacer_as_object.is_array() {
            let mut obj_to_return = serde_json::Map::new();
            let replacer_as_object = replacer_as_object.borrow();
            let fields = replacer_as_object.properties().keys().filter_map(|key| {
                if key == "length" {
                    None
                } else {
                    Some(
                        replacer
                            .get_property(key)
                            .as_ref()
                            .map(|d| d.value())
                            .flatten()
                            .cloned()
                            .unwrap_or_default(),
                    )
                }
            });
            for field in fields {
                let v = object.get_field(field.to_string(context)?, context)?;
                if !v.is_undefined() {
                    if let Some(value) = v.to_json(context)? {
                        obj_to_return.insert(field.to_string(context)?.to_string(), value);
                    }
                }
            }
            Ok(JsValue::new(json_to_pretty_string(
                &JSONValue::Object(obj_to_return),
                gap,
            )))
        } else if let Some(value) = object.to_json(context)? {
            Ok(JsValue::new(json_to_pretty_string(&value, gap)))
        } else {
            Ok(JsValue::undefined())
        }
    }
}

fn json_to_pretty_string(json: &JSONValue, gap: &str) -> String {
    if gap.is_empty() {
        return json.to_string();
    }
    let formatter = PrettyFormatter::with_indent(gap.as_bytes());
    let mut writer = Vec::with_capacity(128);
    let mut serializer = Serializer::with_formatter(&mut writer, formatter);
    json.serialize(&mut serializer)
        .expect("JSON serialization failed");
    unsafe {
        // The serde json serializer always produce correct UTF-8
        String::from_utf8_unchecked(writer)
    }
}
