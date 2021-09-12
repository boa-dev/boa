//! This module implements the global `Set` objest.
//!
//! The JavaScript `Set` class is a global object that is used in the construction of sets; which
//! are high-level, collections of values.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-set-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set

use crate::{
    builtins::{iterable::get_iterator, BuiltIn},
    context::StandardObjects,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        ObjectData,
    },
    property::{Attribute, PropertyNameKind},
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult, JsValue,
};
use ordered_set::OrderedSet;

pub mod set_iterator;
use set_iterator::SetIterator;

use super::JsArgs;

pub mod ordered_set;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct Set(OrderedSet<JsValue>);

impl BuiltIn for Set {
    const NAME: &'static str = "Set";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructable(false)
            .build();

        let size_getter = FunctionBuilder::native(context, Self::size_getter)
            .constructable(false)
            .name("get size")
            .build();

        let iterator_symbol = WellKnownSymbols::iterator();

        let to_string_tag = WellKnownSymbols::to_string_tag();

        let values_function = FunctionBuilder::native(context, Self::values)
            .name("values")
            .length(0)
            .constructable(false)
            .build();

        let set_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().set_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
        .method(Self::add, "add", 1)
        .method(Self::clear, "clear", 0)
        .method(Self::delete, "delete", 1)
        .method(Self::entries, "entries", 0)
        .method(Self::for_each, "forEach", 1)
        .method(Self::has, "has", 1)
        .property(
            "keys",
            values_function.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .accessor("size", Some(size_getter), None, Attribute::CONFIGURABLE)
        .property(
            "values",
            values_function.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            iterator_symbol,
            values_function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            to_string_tag,
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .build();

        (Self::NAME, set_object.into(), Self::attribute())
    }
}

impl Set {
    pub(crate) const LENGTH: usize = 0;

    /// Create a new set
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1
        if new_target.is_undefined() {
            return context
                .throw_type_error("calling a builtin Set constructor without new is forbidden");
        }

        // 2
        let prototype =
            get_prototype_from_constructor(new_target, StandardObjects::set_object, context)?;

        let obj = context.construct_object();
        obj.set_prototype_instance(prototype.into());

        let set = JsValue::new(obj);
        // 3
        set.set_data(ObjectData::set(OrderedSet::default()));

        let iterable = args.get_or_undefined(0);
        // 4
        if iterable.is_null_or_undefined() {
            return Ok(set);
        }

        // 5
        let adder = set.get_field("add", context)?;

        // 6
        if !adder.is_function() {
            return context.throw_type_error("'add' of 'newTarget' is not a function");
        }

        // 7
        let iterator_record = get_iterator(iterable, context)?;

        // 8.a
        let mut next = iterator_record.next(context)?;

        // 8
        while !next.done {
            // c
            let next_value = next.value;

            // d, e
            if let Err(status) = context.call(&adder, &set, &[next_value]) {
                return iterator_record.close(Err(status), context);
            }

            next = iterator_record.next(context)?
        }

        // 8.b
        Ok(set)
    }

    /// `get Set [ @@species ]`
    ///
    /// The Set[Symbol.species] accessor property returns the Set constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-set-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/@@species
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `Set.prototype.add( value )`
    ///
    /// This method adds an entry with value into the set. Returns the set object
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.add
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/add
    pub(crate) fn add(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let value = args.get_or_undefined(0);

        if let Some(object) = this.as_object() {
            if let Some(set) = object.borrow_mut().as_set_mut() {
                set.add(if value.as_number().map(|n| n == -0f64).unwrap_or(false) {
                    JsValue::Integer(0)
                } else {
                    value.clone()
                });
            } else {
                return context.throw_type_error("'this' is not a Set");
            }
        } else {
            return context.throw_type_error("'this' is not a Set");
        };

        Ok(this.clone())
    }

    /// `Set.prototype.clear( )`
    ///
    /// This method removes all entries from the set.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.clear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/clear
    pub(crate) fn clear(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if object.borrow().is_set() {
                this.set_data(ObjectData::set(OrderedSet::new()));
                Ok(JsValue::undefined())
            } else {
                context.throw_type_error("'this' is not a Set")
            }
        } else {
            context.throw_type_error("'this' is not a Set")
        }
    }

    /// `Set.prototype.delete( value )`
    ///
    /// This method removes the entry for the given value if it exists.
    /// Returns true if there was an element, false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.delete
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/delete
    pub(crate) fn delete(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let value = args.get_or_undefined(0);

        let res = if let Some(object) = this.as_object() {
            if let Some(set) = object.borrow_mut().as_set_mut() {
                set.delete(value)
            } else {
                return context.throw_type_error("'this' is not a Set");
            }
        } else {
            return context.throw_type_error("'this' is not a Set");
        };

        Ok(res.into())
    }

    /// `Set.prototype.entries( )`
    ///
    /// This method returns an iterator over the entries of the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.entries
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/entries
    pub(crate) fn entries(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            let object = object.borrow();
            if !object.is_set() {
                return context.throw_type_error(
                    "Method Set.prototype.entries called on incompatible receiver",
                );
            }
        } else {
            return context
                .throw_type_error("Method Set.prototype.entries called on incompatible receiver");
        }

        Ok(SetIterator::create_set_iterator(
            this.clone(),
            PropertyNameKind::KeyAndValue,
            context,
        ))
    }

    /// `Set.prototype.forEach( callbackFn [ , thisArg ] )`
    ///
    /// This method executes the provided callback function for each value in the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.foreach
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/foreach
    pub(crate) fn for_each(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if args.is_empty() {
            return Err(JsValue::new("Missing argument for Set.prototype.forEach"));
        }

        let callback_arg = &args[0];
        let this_arg = args.get_or_undefined(1);
        // TODO: if condition should also check that we are not in strict mode
        let this_arg = if this_arg.is_undefined() {
            JsValue::Object(context.global_object())
        } else {
            this_arg.clone()
        };

        let mut index = 0;

        while index < Set::get_size(this, context)? {
            let arguments = if let JsValue::Object(ref object) = this {
                let object = object.borrow();
                if let Some(set) = object.as_set_ref() {
                    set.get_index(index)
                        .map(|value| [value.clone(), value.clone(), this.clone()])
                } else {
                    return context.throw_type_error("'this' is not a Set");
                }
            } else {
                return context.throw_type_error("'this' is not a Set");
            };

            if let Some(arguments) = arguments {
                context.call(callback_arg, &this_arg, &arguments)?;
            }

            index += 1;
        }

        Ok(JsValue::Undefined)
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
        let value = args.get_or_undefined(0);

        if let JsValue::Object(ref object) = this {
            let object = object.borrow();
            if let Some(set) = object.as_set_ref() {
                return Ok(set.contains(value).into());
            }
        }

        Err(context.construct_type_error("'this' is not a Set"))
    }

    /// `Set.prototype.values( )`
    ///
    /// This method returns an iterator over the values of the set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/values
    pub(crate) fn values(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            let object = object.borrow();
            if !object.is_set() {
                return context.throw_type_error(
                    "Method Set.prototype.values called on incompatible receiver",
                );
            }
        } else {
            return context
                .throw_type_error("Method Set.prototype.values called on incompatible receiver");
        }

        Ok(SetIterator::create_set_iterator(
            this.clone(),
            PropertyNameKind::Value,
            context,
        ))
    }

    fn size_getter(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Set::get_size(this, context).map(JsValue::from)
    }

    /// Helper function to get the size of the set.
    fn get_size(set: &JsValue, context: &mut Context) -> JsResult<usize> {
        if let JsValue::Object(ref object) = set {
            let object = object.borrow();
            if let Some(set) = object.as_set_ref() {
                Ok(set.size())
            } else {
                Err(context.construct_type_error("'this' is not a Set"))
            }
        } else {
            Err(context.construct_type_error("'this' is not a Set"))
        }
    }
}
