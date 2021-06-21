/*
https://tc39.es/ecma262/#sec-typedarray-objects
The %TypedArray% intrinsic object:

* is a constructor function object that all of the TypedArray constructor objects inherit from.
* along with its corresponding prototype object, provides common properties that are inherited
  by all TypedArray constructors and their instances.
* does not have a global name or appear as a property of the global object.
* acts as the abstract superclass of the various TypedArray constructors.
* will throw an error when invoked, because it is an abstract class constructor.
  The TypedArray constructors do not perform a super call to it.
*/

use crate::builtins::array::array_iterator::ArrayIterationKind;
use crate::builtins::{ArrayIterator, BuiltIn};
use crate::context::StandardConstructor;
use crate::object::{ConstructorBuilder, FunctionBuilder, PROTOTYPE};
use crate::property::{Attribute, DataDescriptor};
use crate::symbol::WellKnownSymbols;
use crate::{Context, Result, Value};

pub(crate) struct TypedArray;

impl BuiltIn for TypedArray {
    const NAME: &'static str = "TypedArray";

    fn attribute() -> Attribute {
        Attribute::NON_ENUMERABLE | Attribute::PERMANENT | Attribute::READONLY
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        (
            Self::NAME,
            Self::create_constructor(context),
            Self::attribute(),
        )
    }
}

impl TypedArray {
    pub(crate) fn create_constructor(context: &mut Context) -> Value {
        let constructor = ConstructorBuilder::with_standard_object(
            context,
            Self::construct,
            context.standard_objects().typed_array_object().clone(),
        )
        .name(Self::NAME)
        .static_method(Self::from, "from", 3)
        .static_method(Self::of, "of", 1)
        .build();
        constructor.into()
    }

    fn construct_species(
        this: &Value,
        arguments: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let length = arguments
            .get(0)
            .map(|v| v.get_field("length", context))
            .transpose()?
            .unwrap_or(0.into());

        let species = this
            .as_object()
            .ok_or_else(|| -> Value { "Not a constructor".into() })?
            .get(
                &WellKnownSymbols::species().into(),
                this.clone().into(),
                context,
            )?;

        if let Some(c) = species.as_object() {
            if !c.is_constructable() {
                return context.throw_type_error("Not constructable");
            }
            c.construct(&[length], &c.clone().into(), context)
        } else {
            // Figure out what to do here if the species is nothing
            return Ok(Value::undefined());
        }
    }

    pub(crate) fn of(this: &Value, arguments: &[Value], context: &mut Context) -> Result<Value> {
        let length: Value = arguments.len().into();

        let constructed_value = Self::construct_species(this, &[length], context)?;

        for (index, value) in arguments.iter().enumerate() {
            constructed_value.set_property(
                index.to_string(),
                DataDescriptor::new(value.clone(), Attribute::WRITABLE | Attribute::ENUMERABLE),
            );
        }

        Ok(constructed_value)
    }

    pub(crate) fn from(this: &Value, arguments: &[Value], context: &mut Context) -> Result<Value> {
        // The third argument to from is an optional this value. If it's present, use it instead of our this
        let this = arguments.get(2).cloned().unwrap_or(this.clone());
        let map_fn = arguments.get(1).cloned().unwrap_or(Value::undefined());

        let mapping = match (
            map_fn.is_null_or_undefined(),
            map_fn.as_object().map(|o| o.is_callable()).unwrap_or(false),
        ) {
            // Exists but is not callable
            (false, false) => {
                return context.throw_type_error("mapFn is not a function");
            }
            (false, true) => true,
            _ => false,
        };

        let length = arguments
            .get(0)
            .map(|v| v.get_field("length", context))
            .transpose()?
            .unwrap_or(0.into());

        let iter = crate::builtins::iterable::get_iterator(context, arguments[0].clone())?;

        let constructed_value = Self::construct_species(&this, &[length], context)?;

        let mut index = 0;
        while let Ok(next) = iter.next(context) {
            if next.is_done() {
                break;
            }

            let value = if mapping {
                map_fn
                    .as_object()
                    .unwrap()
                    .call(&map_fn, &[next.value()], context)?
            } else {
                next.value()
            };

            constructed_value.set_property(
                index.to_string(),
                DataDescriptor::new(value, Attribute::WRITABLE | Attribute::ENUMERABLE),
            );
            index += 1;
        }

        Ok(constructed_value)
    }

    pub(crate) fn construct(
        _new_target: &Value,
        _args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        context.throw_type_error("Cannot call TypedArray as a constructor")
    }
}

pub(crate) trait TypedArrayInstance {
    const BYTES_PER_ELEMENT: usize;
    const NAME: &'static str;

    fn constructor(new_target: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| {
                context
                    .standard_objects()
                    .typed_array_object()
                    .constructor
                    .clone()
            });

        let typed_array = Value::new_object(context);

        typed_array
            .as_object()
            .expect("'Invariant. typed_array was created as an object'")
            .set_prototype_instance(prototype.into());

        let length = DataDescriptor::new(
            args[0].clone(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );

        typed_array.set_property("length", length);

        Ok(typed_array)
    }

    fn get_species(this: &Value, _args: &[Value], _context: &mut Context) -> Result<Value> {
        Ok(this.clone())
    }

    fn values(this: &Value, _args: &[Value], context: &mut Context) -> Result<Value> {
        Ok(ArrayIterator::create_array_iterator(
            context,
            this.clone(),
            ArrayIterationKind::Value,
        ))
    }
}

impl<T> BuiltIn for T
where
    T: TypedArrayInstance,
{
    const NAME: &'static str = <T as TypedArrayInstance>::NAME;

    fn attribute() -> Attribute {
        // TODO: Figure out what this means because :shrug:
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let typed_array_prototype = context
            .standard_objects()
            .typed_array_object()
            .prototype()
            .clone()
            .into();

        let values_fn = FunctionBuilder::new(context, Self::values)
            .name("values")
            .length(0)
            .callable(true)
            .constructable(false)
            .build();

        let species_get_fn = FunctionBuilder::new(context, Self::get_species)
            .name("get [Symbol.species]")
            .callable(true)
            .constructable(false)
            .build();

        let constructor = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            StandardConstructor::default(),
        )
        .inherit(typed_array_prototype)
        .property(
            "BYTES_PER_ELEMENT",
            Self::BYTES_PER_ELEMENT,
            Attribute::READONLY | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        )
        .static_accessor(
            WellKnownSymbols::species(),
            Some(species_get_fn),
            None,
            Attribute::CONFIGURABLE,
        )
        .property(
            "BYTES_PER_ELEMENT",
            Self::BYTES_PER_ELEMENT,
            Attribute::PERMANENT | Attribute::READONLY,
        )
        .static_property(
            "BYTES_PER_ELEMENT",
            Self::BYTES_PER_ELEMENT,
            Attribute::PERMANENT | Attribute::READONLY,
        )
        .property(
            WellKnownSymbols::iterator(),
            values_fn,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        // .property(
        //     "values",
        //     values_fn,
        //     Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        // )
        // .property(
        //     WellKnownSymbols::iterator(),
        //     values_fn.clone(),
        //     Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        // )
        .static_method(TypedArray::from, "from", 3)
        .static_method(TypedArray::of, "of", 0)
        .name(Self::NAME)
        .length(3)
        .build();

        (Self::NAME, constructor.into(), Self::attribute())
    }
}
