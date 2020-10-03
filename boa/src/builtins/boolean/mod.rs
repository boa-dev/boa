//! This module implements the global `Boolean` object.
//!
//! The `Boolean` object is an object wrapper for a boolean value.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-boolean-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean

#[cfg(test)]
mod tests;

use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, ObjectData},
    property::Attribute,
    BoaProfiler, Context, Result, Value,
};

/// Boolean implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Boolean;

impl BuiltIn for Boolean {
    /// The name of the object.
    const NAME: &'static str = "Boolean";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let boolean_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().boolean_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::to_string, "toString", 0)
        .method(Self::value_of, "valueOf", 0)
        .build();

        (Self::NAME, boolean_object.into(), Self::attribute())
    }
}

impl Boolean {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// `[[Construct]]` Create a new boolean object
    ///
    /// `[[Call]]` Creates a new boolean primitive
    pub(crate) fn constructor(this: &Value, args: &[Value], _: &mut Context) -> Result<Value> {
        // Get the argument, if any
        let data = args.get(0).map(|x| x.to_boolean()).unwrap_or(false);
        this.set_data(ObjectData::Boolean(data));

        Ok(Value::from(data))
    }

    /// An Utility function used to get the internal [[BooleanData]].
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisbooleanvalue
    fn this_boolean_value(value: &Value, ctx: &mut Context) -> Result<bool> {
        match value {
            Value::Boolean(boolean) => return Ok(*boolean),
            Value::Object(ref object) => {
                let object = object.borrow();
                if let Some(boolean) = object.as_boolean() {
                    return Ok(boolean);
                }
            }
            _ => {}
        }

        Err(ctx.construct_type_error("'this' is not a boolean"))
    }

    /// The `toString()` method returns a string representing the specified `Boolean` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean-object
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean/toString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(this: &Value, _: &[Value], ctx: &mut Context) -> Result<Value> {
        let boolean = Self::this_boolean_value(this, ctx)?;
        Ok(Value::from(boolean.to_string()))
    }

    /// The valueOf() method returns the primitive value of a `Boolean` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean/valueOf
    #[inline]
    pub(crate) fn value_of(this: &Value, _: &[Value], ctx: &mut Context) -> Result<Value> {
        Ok(Value::from(Self::this_boolean_value(this, ctx)?))
    }
}
