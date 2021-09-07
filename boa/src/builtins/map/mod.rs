//! This module implements the global `Map` objest.
//!
//! The JavaScript `Map` class is a global object that is used in the construction of maps; which
//! are high-level, key-value stores.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-map-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map

#![allow(clippy::mutable_key_type)]

use crate::{
    builtins::BuiltIn,
    context::StandardObjects,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        ObjectData,
    },
    property::{Attribute, PropertyDescriptor, PropertyNameKind},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult, JsValue,
};
use ordered_map::OrderedMap;

pub mod map_iterator;
use map_iterator::MapIterator;

use self::ordered_map::MapLock;

use super::JsArgs;

pub mod ordered_map;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct Map(OrderedMap<JsValue>);

impl BuiltIn for Map {
    const NAME: &'static str = "Map";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let iterator_symbol = WellKnownSymbols::iterator();

        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructable(false)
            .build();

        let entries_function = FunctionBuilder::native(context, Self::entries)
            .name("entries")
            .length(0)
            .constructable(false)
            .build();

        let map_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().map_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
        .property(
            "entries",
            entries_function.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            to_string_tag,
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
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
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return context
                .throw_type_error("calling a builtin Map constructor without new is forbidden");
        }
        let prototype =
            get_prototype_from_constructor(new_target, StandardObjects::map_object, context)?;

        let obj = context.construct_object();
        obj.set_prototype_instance(prototype.into());
        let this = JsValue::new(obj);

        // add our arguments in
        let data = match args.len() {
            0 => OrderedMap::new(),
            _ => match &args[0] {
                JsValue::Object(object) => {
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
        this.set_data(ObjectData::map(data));

        Ok(this)
    }

    /// `get Map [ @@species ]`
    ///
    /// The `Map [ @@species ]` accessor property returns the Map constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-map-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/@@species
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
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
    pub(crate) fn entries(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        MapIterator::create_map_iterator(context, this.clone(), PropertyNameKind::KeyAndValue)
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
    pub(crate) fn keys(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        MapIterator::create_map_iterator(context, this.clone(), PropertyNameKind::Key)
    }

    /// Helper function to set the size property.
    fn set_size(this: &JsValue, size: usize) {
        let size = PropertyDescriptor::builder()
            .value(size)
            .writable(false)
            .enumerable(false)
            .configurable(false);

        this.set_property("size", size);
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
    pub(crate) fn set(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);

        let size = if let Some(object) = this.as_object() {
            if let Some(map) = object.borrow_mut().as_map_mut() {
                map.insert(key.clone(), value.clone());
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
    pub(crate) fn delete(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

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
    pub(crate) fn get(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

        if let JsValue::Object(ref object) = this {
            let object = object.borrow();
            if let Some(map) = object.as_map_ref() {
                return Ok(if let Some(result) = map.get(key) {
                    result.clone()
                } else {
                    JsValue::undefined()
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
    pub(crate) fn clear(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        this.set_data(ObjectData::map(OrderedMap::new()));

        Self::set_size(this, 0);

        Ok(JsValue::undefined())
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
    pub(crate) fn has(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let key = args.get_or_undefined(0);

        if let JsValue::Object(ref object) = this {
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
    pub(crate) fn for_each(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if args.is_empty() {
            return Err(JsValue::new("Missing argument for Map.prototype.forEach"));
        }

        let callback_arg = &args[0];
        let this_arg = args.get_or_undefined(1);

        let mut index = 0;

        let lock = Map::lock(this, context)?;

        while index < Map::get_full_len(this, context)? {
            let arguments = if let JsValue::Object(ref object) = this {
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
                context.call(callback_arg, this_arg, &arguments)?;
            }

            index += 1;
        }

        drop(lock);

        Ok(JsValue::undefined())
    }

    /// Helper function to get the full size of the map.
    fn get_full_len(map: &JsValue, context: &mut Context) -> JsResult<usize> {
        if let JsValue::Object(ref object) = map {
            let object = object.borrow();
            if let Some(map) = object.as_map_ref() {
                Ok(map.full_len())
            } else {
                Err(context.construct_type_error("'this' is not a Map"))
            }
        } else {
            Err(context.construct_type_error("'this' is not a Map"))
        }
    }

    /// Helper function to lock the map.
    fn lock(map: &JsValue, context: &mut Context) -> JsResult<MapLock> {
        if let JsValue::Object(ref object) = map {
            let mut map = object.borrow_mut();
            if let Some(map) = map.as_map_mut() {
                Ok(map.lock(object.clone()))
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
    pub(crate) fn values(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        MapIterator::create_map_iterator(context, this.clone(), PropertyNameKind::Value)
    }

    /// Helper function to get a key-value pair from an array.
    fn get_key_value(
        value: &JsValue,
        context: &mut Context,
    ) -> JsResult<Option<(JsValue, JsValue)>> {
        if let JsValue::Object(object) = value {
            if object.is_array() {
                let (key, value) =
                    match value.get_field("length", context)?.as_number().unwrap() as i32 {
                        0 => (JsValue::undefined(), JsValue::undefined()),
                        1 => (value.get_field("0", context)?, JsValue::undefined()),
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
