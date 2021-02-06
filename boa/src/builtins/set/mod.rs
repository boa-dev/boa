use crate::{
    builtins::{iterable::get_iterator, BuiltIn},
    object::{ConstructorBuilder, FunctionBuilder, ObjectData, PROTOTYPE},
    property::Attribute,
    BoaProfiler, Context, Result, Value,
};
use ordered_set::OrderedSet;

pub mod set_iterator;
use set_iterator::{SetIterationKind, SetIterator};

pub mod ordered_set;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct Set(OrderedSet<Value>);

impl BuiltIn for Set {
    const NAME: &'static str = "Set";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let species = context.well_known_symbols().species_symbol();

        let species_getter = FunctionBuilder::new(context, Self::species_getter)
            .callable(true)
            .constructable(false)
            .name("get [Symbol.species]")
            .build();

        let size_getter = FunctionBuilder::new(context, Self::size_getter)
            .callable(true)
            .constructable(false)
            .build();

        let iterator_symbol = context.well_known_symbols().iterator_symbol();

        let to_string_tag = context.well_known_symbols().to_string_tag_symbol();

        let values_function = FunctionBuilder::new(context, Self::values)
            .name("values")
            .length(0)
            .callable(true)
            .constructable(false)
            .build();

        let set_object = ConstructorBuilder::new(context, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .static_accessor(species, Some(species_getter), None, Attribute::default())
            .method(Self::add, "add", 1)
            .method(Self::clear, "clear", 0)
            .method(Self::delete, "delete", 1)
            .method(Self::entries, "entries", 0)
            .method(Self::for_each, "forEach", 0)
            .method(Self::has, "has", 1)
            .property(
                "keys",
                values_function.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor("size", Some(size_getter), None, Attribute::default())
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
            .property(to_string_tag, "Set", Attribute::CONFIGURABLE)
            .build();

        (Self::NAME, set_object.into(), Self::attribute())
    }
}

impl Set {
    pub(crate) const LENGTH: usize = 0;

    /// Create a new set
    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        // 1
        if new_target.is_undefined() {
            return context.throw_type_error("Constructor Set requires 'new'");
        }

        // 2
        let set_prototype = context
            .global_object()
            .clone()
            .get(
                &"Set".into(),
                context.global_object().clone().into(),
                context,
            )?
            .get_field(PROTOTYPE, context)?
            .as_object()
            .expect("'Set' global property should be an object");
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or(set_prototype);

        let mut obj = context.construct_object();
        obj.set_prototype_instance(prototype.into());

        let set = Value::from(obj);
        // 3
        set.set_data(ObjectData::Set(OrderedSet::default()));

        let iterable = args.get(0).cloned().unwrap_or_default();
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
        let iterator_record = get_iterator(context, iterable)?;

        // 8.a
        let mut next = iterator_record.next(context)?;

        // 8
        while !next.is_done() {
            // c
            let next_value = next.value();

            // d, e
            context.call(&adder, &set, &[next_value])?;

            next = iterator_record.next(context)?
        }

        // 8.b
        Ok(set)
    }

    /// `get Set [ @@species ]`
    ///
    /// get accessor for the @@species property of Set
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-set-@@species
    fn species_getter(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
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
    pub(crate) fn add(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let value = args.get(0).cloned().unwrap_or_default();

        if let Some(object) = this.as_object() {
            if let Some(set) = object.borrow_mut().as_set_mut() {
                set.add(value);
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
    pub(crate) fn clear(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        if let Some(object) = this.as_object() {
            if object.borrow_mut().is_set() {
                this.set_data(ObjectData::Set(OrderedSet::new()));
                Ok(Value::Undefined)
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
    pub(crate) fn delete(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let value = args.get(0).cloned().unwrap_or_default();

        let res = if let Some(object) = this.as_object() {
            if let Some(set) = object.borrow_mut().as_set_mut() {
                set.delete(&value)
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
    pub(crate) fn entries(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        SetIterator::create_set_iterator(context, this.clone(), SetIterationKind::KeyAndValue)
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
    pub(crate) fn for_each(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from("Missing argument for Set.prototype.forEach"));
        }

        let callback_arg = &args[0];
        let this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let mut index = 0;

        while index < Set::get_size(this, context)? {
            let arguments = if let Value::Object(ref object) = this {
                let object = object.borrow();
                if let Some(set) = object.as_set_ref() {
                    if let Some(value) = set.get_index(index) {
                        Some([value.clone(), value.clone(), this.clone()])
                    } else {
                        None
                    }
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
        let value = match args.len() {
            0 => &undefined,
            _ => &args[0],
        };

        if let Value::Object(ref object) = this {
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
    pub(crate) fn values(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        SetIterator::create_set_iterator(context, this.clone(), SetIterationKind::Value)
    }

    fn size_getter(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Set::get_size(this, context).map(Value::from)
    }

    /// Helper function to get the size of the set.
    fn get_size(set: &Value, context: &mut Context) -> Result<usize> {
        if let Value::Object(ref object) = set {
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
