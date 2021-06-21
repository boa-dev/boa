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
use crate::builtins::{ArrayIterator, BigInt, BuiltIn};
use crate::context::StandardConstructor;
use crate::gc::{Finalize, Trace};
use crate::object::{ConstructorBuilder, FunctionBuilder, GcObject, ObjectData, PROTOTYPE};
use crate::property::{Attribute, DataDescriptor};
use crate::symbol::WellKnownSymbols;
use crate::value::Numeric;
use crate::{Context, Result, Value};

#[derive(Debug, Clone, PartialOrd, PartialEq, Trace, Finalize)]
pub(crate) enum TypedArrayStorageClass {
    I8(Vec<i8>),
    U8(Vec<u8>),
    I16(Vec<i16>),
    U16(Vec<u16>),
    I32(Vec<i32>),
    U32(Vec<u32>),
    F32(Vec<f32>),
    F64(Vec<f64>),
    BigInt64(Vec<BigInt>),
}

impl TypedArrayStorageClass {
    fn get_typed_array_content_type(&self) -> TypedArrayContentType {
        match self {
            Self::BigInt64(_) => TypedArrayContentType::BigInt,
            _ => TypedArrayContentType::Number,
        }
    }

    unsafe fn set_length(&mut self, len: usize) {
        use TypedArrayStorageClass::*;
        match self {
            I8(value) => value.set_len(len),
            U8(value) => value.set_len(len),
            I16(value) => value.set_len(len),
            U16(value) => value.set_len(len),
            I32(value) => value.set_len(len),
            U32(value) => value.set_len(len),
            F32(value) => value.set_len(len),
            F64(value) => value.set_len(len),
            BigInt64(value) => value.set_len(len),
        }
    }

    pub(crate) fn set_value_at_index(
        &mut self,
        index: u32,
        value: Value,
        context: &mut Context,
    ) -> Result<Value> {
        use TypedArrayStorageClass::*;
        let numeric = value.to_numeric(context)?;
        //  TODO: implement the type conversion methods
        let index = index as usize;
        // TODO: Handle out of bounds exceptions here
        match (self, numeric) {
            (I8(value), Numeric::Number(number)) => value[index] = number as i8,
            (U8(value), Numeric::Number(number)) => value[index] = number as u8,
            (I16(value), Numeric::Number(number)) => value[index] = number as i16,
            (U16(value), Numeric::Number(number)) => value[index] = number as u16,
            (I32(value), Numeric::Number(number)) => value[index] = number as i32,
            (U32(value), Numeric::Number(number)) => value[index] = number as u32,
            (F32(value), Numeric::Number(number)) => value[index] = number as f32,
            (F64(value), Numeric::Number(number)) => value[index] = number,
            (BigInt64(value), Numeric::BigInt(big_int)) => {
                value[index] = big_int.as_inner().clone()
            }
            _ => return context.throw_type_error("Must set numeric value for typed array"),
        };

        Ok(Value::undefined())
    }

    pub(crate) fn get_value_at_index(&self, index: u32) -> Option<Value> {
        use TypedArrayStorageClass::*;
        let numeric = match self {
            I8(value) => value.get(index as usize).cloned().map(Numeric::from),
            U8(value) => value.get(index as usize).cloned().map(Numeric::from),
            I16(value) => value.get(index as usize).cloned().map(Numeric::from),
            U16(value) => value.get(index as usize).cloned().map(Numeric::from),
            I32(value) => value.get(index as usize).cloned().map(Numeric::from),
            U32(value) => value.get(index as usize).cloned().map(Numeric::from),
            F32(value) => value
                .get(index as usize)
                .cloned()
                .map(|v| Numeric::from(v as f64)),
            F64(value) => value.get(index as usize).cloned().map(Numeric::from),
            BigInt64(value) => value.get(index as usize).cloned().map(Numeric::from),
        };

        Some(numeric.map(Value::from).unwrap_or(Value::undefined()))
    }

    pub(crate) fn length(&self) -> usize {
        use TypedArrayStorageClass::*;
        let capacity = match self {
            I8(value) => value.capacity(),
            U8(value) => value.capacity(),
            I16(value) => value.capacity(),
            U16(value) => value.capacity(),
            I32(value) => value.capacity(),
            U32(value) => value.capacity(),
            F32(value) => value.capacity(),
            F64(value) => value.capacity(),
            BigInt64(value) => value.capacity(),
        };

        capacity
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Trace, Finalize)]
pub(crate) enum TypedArrayContentType {
    Number,
    BigInt,
}

// Corresponds to internal slots
// TODO: Make buffer an acual buffer instead of a Vec<_>
#[derive(Debug, Finalize, Trace)]
pub struct TypedArray {
    typed_array_name: &'static str,
    byte_length: usize,
    byte_offset: usize,
    array_length: usize,
    content_type: TypedArrayContentType,
    pub(crate) buffer: TypedArrayStorageClass,
}

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

    fn get_length(this: &Value, _args: &[Value], context: &mut Context) -> Result<Value> {
        let length = this.as_object().map(|v| match v.borrow().data {
            ObjectData::TypedArray(ref typed_array) => Value::from(typed_array.buffer.length()),
            _ => Value::undefined(),
        });

        if let Some(length) = length {
            Ok(length)
        } else {
            context.throw_type_error("Unable to get length")
        }
    }

    fn construct_species(
        this: &Value,
        arguments: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let length = arguments.get(0).cloned().unwrap_or(0.into());

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

        let source = match arguments.get(0) {
            Some(value) => value.clone(),
            None => return context.throw_type_error("From requires an argument"),
        };

        let length = source.get_field("length", context)?;

        if length.is_null_or_undefined() {
            return context.throw_type_error("TypedArray.from requires an interable");
        }

        let iter = crate::builtins::iterable::get_iterator(context, arguments[0].clone())?;

        let constructed_value = Self::construct_species(&this, &[length], context)?;
        let object = constructed_value
            .as_object()
            .expect("Invariant, Array object was just constructed");

        let mut handle = object.borrow_mut();
        let mut index = 0;

        while let Ok(next) = iter.next(context) {
            if next.is_done() {
                break;
            }
            match handle.data {
                ObjectData::TypedArray(ref mut typed_array) => {
                    typed_array
                        .buffer
                        .set_value_at_index(index, next.value(), context)?;
                }
                _ => {}
            }

            index += 1
        }

        drop(handle);

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

    fn get_storage_class(capacity: usize) -> TypedArrayStorageClass;

    fn _internal_get_storage_class(capacity: usize) -> TypedArrayStorageClass {
        let mut storage_class = Self::get_storage_class(capacity);
        // SAFETY: We guarantee here that we always set the length of the array to its
        // actual internal capacity.
        unsafe { storage_class.set_length(storage_class.length()) };
        storage_class
    }

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

        match args.get(0).cloned() {
            Some(value) if value.is_number() => {
                Self::create_typed_array_by_length(prototype, value, context)
            }
            _ => todo!(),
        }
    }

    fn create_typed_array_by_length(
        proto: GcObject,
        length: Value,
        context: &mut Context,
    ) -> Result<Value> {
        let length = length
            .as_number()
            .expect("Should never be called with a non number length");
        let value = Self::allocate_typed_array(length as usize, context);
        let mut object = value.as_object().unwrap();

        object.set_prototype_instance(proto.into());

        Ok(value)
    }

    fn allocate_typed_array(length: usize, context: &mut Context) -> Value {
        let object = Value::new_object(context);

        let storage_class = Self::_internal_get_storage_class(length);

        let typed_array = TypedArray {
            typed_array_name: Self::NAME,
            byte_length: Self::BYTES_PER_ELEMENT,
            byte_offset: Self::BYTES_PER_ELEMENT,
            array_length: 0,
            content_type: storage_class.get_typed_array_content_type(),
            buffer: storage_class,
        };

        object.set_data(ObjectData::TypedArray(typed_array));

        object
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

        let length_getter = FunctionBuilder::new(context, TypedArray::get_length)
            .name("get [length]")
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
        .accessor(
            "length",
            Some(length_getter),
            None,
            Attribute::PERMANENT | Attribute::READONLY,
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
            values_fn.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            "values",
            values_fn,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .static_method(TypedArray::from, "from", 3)
        .static_method(TypedArray::of, "of", 0)
        .name(Self::NAME)
        .length(3)
        .build();

        (Self::NAME, constructor.into(), Self::attribute())
    }
}
