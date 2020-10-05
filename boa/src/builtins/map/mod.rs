#![allow(clippy::mutable_key_type)]

use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, ObjectData, PROTOTYPE},
    property::{Attribute, Property},
    BoaProfiler, Context, Result, Value,
};
use ordered_map::OrderedMap;

pub mod ordered_map;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct Map(OrderedMap<Value, Value>);

impl BuiltIn for Map {
    const NAME: &'static str = "Map";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let map_object = ConstructorBuilder::new(context, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .method(Self::set, "set", 2)
            .method(Self::delete, "delete", 1)
            .method(Self::get, "get", 1)
            .method(Self::clear, "clear", 0)
            .method(Self::has, "has", 1)
            .method(Self::for_each, "forEach", 1)
            .callable(false)
            .build();

        (Self::NAME, map_object.into(), Self::attribute())
    }
}

impl Map {
    pub(crate) const LENGTH: usize = 1;

    /// Create a new map
    pub(crate) fn constructor(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        // Set Prototype
        let prototype = ctx.global_object().get_field("Map").get_field(PROTOTYPE);

        this.as_object_mut()
            .expect("this is map object")
            .set_prototype_instance(prototype);
        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)

        // add our arguments in
        let data = match args.len() {
            0 => OrderedMap::new(),
            _ => match &args[0] {
                Value::Object(object) => {
                    let object = object.borrow();
                    if let Some(map) = object.as_map_ref().cloned() {
                        map
                    } else if object.is_array() {
                        let mut map = OrderedMap::new();
                        let len = args[0].get_field("length").to_integer(ctx)? as i32;
                        for i in 0..len {
                            let val = &args[0].get_field(i.to_string());
                            let (key, value) = Self::get_key_value(val).ok_or_else(|| {
                                ctx.construct_type_error(
                                    "iterable for Map should have array-like objects",
                                )
                            })?;
                            map.insert(key, value);
                        }
                        map
                    } else {
                        return Err(ctx.construct_type_error(
                            "iterable for Map should have array-like objects",
                        ));
                    }
                }
                _ => {
                    return Err(
                        ctx.construct_type_error("iterable for Map should have array-like objects")
                    )
                }
            },
        };

        // finally create length property
        Self::set_size(this, data.len());

        this.set_data(ObjectData::Map(data));

        Ok(this.clone())
    }

    /// Helper function to set the size property.
    fn set_size(this: &Value, size: usize) {
        let size = Property::data_descriptor(
            size.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );

        this.set_property("size".to_string(), size);
    }

    /// `Map.prototype.set( key, value )`
    ///
    /// This method associates the value with the key. Returns the map object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.set
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/set
    pub(crate) fn set(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let (key, value) = match args.len() {
            0 => (Value::Undefined, Value::Undefined),
            1 => (args[0].clone(), Value::Undefined),
            _ => (args[0].clone(), args[1].clone()),
        };

        let size = if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(map) = object.as_map_mut() {
                map.insert(key, value);
                map.len()
            } else {
                return Err(ctx.construct_type_error("'this' is not a Map"));
            }
        } else {
            return Err(ctx.construct_type_error("'this' is not a Map"));
        };

        Self::set_size(this, size);
        Ok(this.clone())
    }

    /// `Map.prototype.delete( key )`
    ///
    /// This method removes the element associated with the key, if it exists. Returns true if there was an element, false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/delete
    pub(crate) fn delete(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let undefined = Value::Undefined;
        let key = match args.len() {
            0 => &undefined,
            _ => &args[0],
        };

        let (deleted, size) = if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(map) = object.as_map_mut() {
                let deleted = map.remove(key).is_some();
                (deleted, map.len())
            } else {
                return Err(ctx.construct_type_error("'this' is not a Map"));
            }
        } else {
            return Err(ctx.construct_type_error("'this' is not a Map"));
        };
        Self::set_size(this, size);
        Ok(deleted.into())
    }

    /// `Map.prototype.get( key )`
    ///
    /// This method returns the value associated with the key, or undefined if there is none.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.get
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/get
    pub(crate) fn get(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let undefined = Value::Undefined;
        let key = match args.len() {
            0 => &undefined,
            _ => &args[0],
        };

        if let Value::Object(ref object) = this {
            let object = object.borrow();
            if let Some(map) = object.as_map_ref() {
                return Ok(if let Some(result) = map.get(key) {
                    result.clone()
                } else {
                    Value::Undefined
                });
            }
        }

        Err(ctx.construct_type_error("'this' is not a Map"))
    }

    /// `Map.prototype.clear( )`
    ///
    /// This method removes all entries from the map.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.clear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/clear
    pub(crate) fn clear(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
        this.set_data(ObjectData::Map(OrderedMap::new()));

        Self::set_size(this, 0);

        Ok(Value::Undefined)
    }

    /// `Map.prototype.has( key )`
    ///
    /// This method checks if the map contains an entry with the given key.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.has
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/has
    pub(crate) fn has(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let undefined = Value::Undefined;
        let key = match args.len() {
            0 => &undefined,
            _ => &args[0],
        };

        if let Value::Object(ref object) = this {
            let object = object.borrow();
            if let Some(map) = object.as_map_ref() {
                return Ok(map.contains_key(key).into());
            }
        }

        Err(ctx.construct_type_error("'this' is not a Map"))
    }

    /// `Map.prototype.forEach( callbackFn [ , thisArg ] )`
    ///
    /// This method executes the provided callback function for each key-value pair in the map.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.foreach
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/forEach
    pub(crate) fn for_each(
        this: &Value,
        args: &[Value],
        interpreter: &mut Context,
    ) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from("Missing argument for Map.prototype.forEach"));
        }

        let callback_arg = &args[0];
        let this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);

        if let Value::Object(ref object) = this {
            let object = object.borrow();
            if let Some(map) = object.as_map_ref().cloned() {
                for (key, value) in map {
                    let arguments = [value, key, this.clone()];

                    interpreter.call(callback_arg, &this_arg, &arguments)?;
                }
            }
        }

        Ok(Value::Undefined)
    }

    /// Helper function to get a key-value pair from an array.
    fn get_key_value(value: &Value) -> Option<(Value, Value)> {
        if let Value::Object(object) = value {
            if object.borrow().is_array() {
                let (key, value) = match value.get_field("length").as_number().unwrap() as i32 {
                    0 => (Value::Undefined, Value::Undefined),
                    1 => (value.get_field("0"), Value::Undefined),
                    _ => (value.get_field("0"), value.get_field("1")),
                };
                return Some((key, value));
            }
        }
        None
    }
}
