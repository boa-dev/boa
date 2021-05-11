#![allow(clippy::mutable_key_type)]

use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, FunctionBuilder, ObjectData, PROTOTYPE},
    property::{Attribute, DataDescriptor},
    BoaProfiler, Context, Result, Value,
};
use ordered_map::OrderedMap;

pub mod map_iterator;
use map_iterator::{MapIterationKind, MapIterator};

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

        let iterator_symbol = context.well_known_symbols().iterator_symbol();

        let entries_function = FunctionBuilder::new(context, Self::entries)
            .name("entries")
            .length(0)
            .callable(true)
            .constructable(false)
            .build();

        let map_object = ConstructorBuilder::new(context, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .property(
                "entries",
                entries_function.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                iterator_symbol,
                entries_function,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(Self::keys, "keys", 0)
            .method(Self::set, "set", 2)
            .method(Self::delete, "delete", 1)
            .method(Self::get, "get", 1)
            .method(Self::clear, "clear", 0)
            .method(Self::has, "has", 1)
            .method(Self::for_each, "forEach", 1)
            .method(Self::values, "values", 0)
            .build();

        (Self::NAME, map_object.into(), Self::attribute())
    }
}

impl Map {
    pub(crate) const LENGTH: usize = 0;

    /// Create a new map
    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        if new_target.is_undefined() {
            return context.throw_type_error("Constructor Map requires 'new'");
        }
        let map_prototype = context
            .global_object()
            .get(&"Map".into(), context.global_object().into(), context)?
            .get_field(PROTOTYPE, context)?
            .as_object()
            .expect("'Map' global property should be an object");
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or(map_prototype);

        let mut obj = context.construct_object();
        obj.set_prototype_instance(prototype.into());
        let this = Value::from(obj);

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
                        let len = args[0].get_field("length", context)?.to_integer(context)? as i32;
                        for i in 0..len {
                            let val = &args[0].get_field(i, context)?;
                            let (key, value) =
                                Self::get_key_value(val, context)?.ok_or_else(|| {
                                    context.construct_type_error(
                                        "iterable for Map should have array-like objects",
                                    )
                                })?;
                            map.insert(key, value);
                        }
                        map
                    } else {
                        return Err(context.construct_type_error(
                            "iterable for Map should have array-like objects",
                        ));
                    }
                }
                _ => {
                    return Err(context
                        .construct_type_error("iterable for Map should have array-like objects"))
                }
            },
        };

        // finally create size property
        Self::set_size(&this, data.len());

        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        this.set_data(ObjectData::Map(data));

        Ok(this)
    }

    /// `Map.prototype.entries()`
    ///
    /// Returns a new Iterator object that contains the [key, value] pairs for each element in the Map object in insertion order.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#sec-map.prototype.entries
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/entries
    pub(crate) fn entries(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Ok(MapIterator::create_map_iterator(
            context,
            this.clone(),
            MapIterationKind::KeyAndValue,
        ))
    }

    /// `Map.prototype.keys()`
    ///
    /// Returns a new Iterator object that contains the keys for each element in the Map object in insertion order.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.keys
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/keys
    pub(crate) fn keys(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Ok(MapIterator::create_map_iterator(
            context,
            this.clone(),
            MapIterationKind::Key,
        ))
    }

    /// Helper function to set the size property.
    fn set_size(this: &Value, size: usize) {
        let size = DataDescriptor::new(
            size,
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
    pub(crate) fn set(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let (key, value) = match args.len() {
            0 => (Value::Undefined, Value::Undefined),
            1 => (args[0].clone(), Value::Undefined),
            _ => (args[0].clone(), args[1].clone()),
        };

        let size = if let Some(object) = this.as_object() {
            if let Some(map) = object.borrow_mut().as_map_mut() {
                map.insert(key, value);
                map.len()
            } else {
                return Err(context.construct_type_error("'this' is not a Map"));
            }
        } else {
            return Err(context.construct_type_error("'this' is not a Map"));
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
    pub(crate) fn delete(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let undefined = Value::Undefined;
        let key = match args.len() {
            0 => &undefined,
            _ => &args[0],
        };

        let (deleted, size) = if let Some(object) = this.as_object() {
            if let Some(map) = object.borrow_mut().as_map_mut() {
                let deleted = map.remove(key).is_some();
                (deleted, map.len())
            } else {
                return Err(context.construct_type_error("'this' is not a Map"));
            }
        } else {
            return Err(context.construct_type_error("'this' is not a Map"));
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
    pub(crate) fn get(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
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

        Err(context.construct_type_error("'this' is not a Map"))
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
    pub(crate) fn has(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
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

        Err(context.construct_type_error("'this' is not a Map"))
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
    pub(crate) fn for_each(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from("Missing argument for Map.prototype.forEach"));
        }

        let callback_arg = &args[0];
        let this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let mut index = 0;

        while index < Map::get_size(this, context)? {
            let arguments = if let Value::Object(ref object) = this {
                let object = object.borrow();
                if let Some(map) = object.as_map_ref() {
                    if let Some((key, value)) = map.get_index(index) {
                        Some([value.clone(), key.clone(), this.clone()])
                    } else {
                        None
                    }
                } else {
                    return context.throw_type_error("'this' is not a Map");
                }
            } else {
                return context.throw_type_error("'this' is not a Map");
            };

            if let Some(arguments) = arguments {
                context.call(callback_arg, &this_arg, &arguments)?;
            }

            index += 1;
        }

        Ok(Value::Undefined)
    }

    /// Helper function to get the size of the map.
    fn get_size(map: &Value, context: &mut Context) -> Result<usize> {
        if let Value::Object(ref object) = map {
            let object = object.borrow();
            if let Some(map) = object.as_map_ref() {
                Ok(map.len())
            } else {
                Err(context.construct_type_error("'this' is not a Map"))
            }
        } else {
            Err(context.construct_type_error("'this' is not a Map"))
        }
    }

    /// `Map.prototype.values()`
    ///
    /// Returns a new Iterator object that contains the values for each element in the Map object in insertion order.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/values
    pub(crate) fn values(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Ok(MapIterator::create_map_iterator(
            context,
            this.clone(),
            MapIterationKind::Value,
        ))
    }

    /// Helper function to get a key-value pair from an array.
    fn get_key_value(value: &Value, context: &mut Context) -> Result<Option<(Value, Value)>> {
        if let Value::Object(object) = value {
            if object.is_array() {
                let (key, value) =
                    match value.get_field("length", context)?.as_number().unwrap() as i32 {
                        0 => (Value::Undefined, Value::Undefined),
                        1 => (value.get_field("0", context)?, Value::Undefined),
                        _ => (
                            value.get_field("0", context)?,
                            value.get_field("1", context)?,
                        ),
                    };
                return Ok(Some((key, value)));
            }
        }
        Ok(None)
    }
}
