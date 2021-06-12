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
use crate::object::{ConstructorBuilder, GcObject, Object};
use crate::property::Attribute;
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
        .method(Self::sanity, "sanity", 0)
        .property(
            "length",
            0,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        )
        .build();

        constructor
    }

    // This is a sanity check to see if objects that inherit from this can call this method
    fn sanity(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        Ok("sanity check".into())
    }

    pub(crate) fn construct(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        context.throw_type_error("not a constructor")
    }
}
