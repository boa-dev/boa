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
    context::StandardObjects,
    object::{internal_methods::get_prototype_from_constructor, ConstructorBuilder, ObjectData},
    property::Attribute,
    BoaProfiler, Context, JsResult, JsValue,
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

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
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
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get the argument, if any
        let data = args.get(0).map(|x| x.to_boolean()).unwrap_or(false);
        if new_target.is_undefined() {
            return Ok(JsValue::new(data));
        }
        let prototype =
            get_prototype_from_constructor(new_target, StandardObjects::boolean_object, context)?;
        let boolean = JsValue::new_object(context);

        boolean
            .as_object()
            .expect("this should be an object")
            .set_prototype_instance(prototype.into());
        boolean.set_data(ObjectData::boolean(data));

        Ok(boolean)
    }

    /// An Utility function used to get the internal `[[BooleanData]]`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisbooleanvalue
    fn this_boolean_value(value: &JsValue, context: &mut Context) -> JsResult<bool> {
        match value {
            JsValue::Boolean(boolean) => return Ok(*boolean),
            JsValue::Object(ref object) => {
                let object = object.borrow();
                if let Some(boolean) = object.as_boolean() {
                    return Ok(boolean);
                }
            }
            _ => {}
        }

        Err(context.construct_type_error("'this' is not a boolean"))
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
    pub(crate) fn to_string(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let boolean = Self::this_boolean_value(this, context)?;
        Ok(JsValue::new(boolean.to_string()))
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
    pub(crate) fn value_of(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(Self::this_boolean_value(this, context)?))
    }
}
