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

use crate::builtins::BuiltIn;
use crate::object::{ConstructorBuilder, GcObject, Object, PROTOTYPE};
use crate::property::{Attribute, DataDescriptor};
use crate::symbol::WellKnownSymbols;
use crate::{Context, Result, Value};

pub(crate) struct TypedArray;

impl TypedArray {
    pub(crate) fn init(context: &mut Context) -> GcObject {
        let constructor = ConstructorBuilder::with_standard_object(
            context,
            Self::construct,
            context.standard_objects().typed_array_object().clone(),
        )
        .property(
            "name",
            "TypedArray",
            Attribute::READONLY | Attribute::PERMANENT,
        )
        .static_method(Self::from, "from", 3)
        .build();
        constructor
    }

    pub(crate) fn from(this: &Value, arguments: &[Value], context: &mut Context) -> Result<Value> {
        // The third argument to from is an optional this value. If it's present, use it instead of our this
        let this = arguments.get(2).cloned().unwrap_or(this.clone());
        let map_fn = arguments.get(1).cloned().unwrap_or(Value::undefined());

        let mut mapping = if !map_fn.is_null_or_undefined()
            && !map_fn.as_object().map(|o| o.is_callable()).unwrap_or(false)
        {
            return context.throw_type_error("mapFn is not callable");
        } else {
            true
        };

        let iter = crate::builtins::iterable::get_iterator(context, arguments[0].clone())?;
        let mut values = vec![];
        while let Ok(next) = iter.next(context) {
            if next.is_done() {
                break;
            }
            values.push(next.value())
        }
        let constructed_value = this
            .as_object()
            .ok_or_else(|| -> Value { "Not a constructor".into() })?
            .call(&this, &[values.len().into()], context)?;

        for (index, mut value) in values.into_iter().enumerate() {
            if mapping {
                value = map_fn
                    .as_object()
                    .unwrap()
                    .call(&map_fn, &[value.clone()], context)?;
            }
            constructed_value.set_property(
                index.to_string(),
                DataDescriptor::new(value, Attribute::all()),
            );
        }

        Ok(constructed_value)
    }

    pub(crate) fn construct(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        context.throw_type_error("not a constructor")
    }
}

pub(crate) trait TypedArrayInstance {
    const BYTES_PER_ELEMENT: usize;
    const NAME: &'static str;

    fn constructor(new_target: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // Check if the argument is a number
        // -- instantiate a new array with that capacity
        // if it's an existing instance of a UInt8Array

        // -- return a value with a prototype of context.typed_array_prototype

        // TODO: Figure out this magical incantation -- from what i can tell this should result in the value
        // returned from TypedArray:init
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
                    .prototype
                    .clone()
            });

        let argument = args.get(0).expect("Expected an argument");
        match argument {
            Value::Integer(v) => {
                let typed_array = Value::new_object(context);
                typed_array
                    .as_object()
                    .expect("'Invariant. typed_array was created as an object'")
                    .set_prototype_instance(prototype.into());
                let length = DataDescriptor::new(
                    *v,
                    Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
                );
                typed_array.set_property("length", length);

                Ok(typed_array)
            }
            _ => {
                todo!()
            }
        }
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
        let constructor = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().typed_array_object().clone(),
        )
        .name(Self::NAME)
        .build();

        (Self::NAME, constructor.into(), Self::attribute())
    }
}
