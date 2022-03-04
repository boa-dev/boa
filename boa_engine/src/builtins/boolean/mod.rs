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
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsObject, ObjectData,
    },
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

/// Boolean implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Boolean;

impl BuiltIn for Boolean {
    /// The name of the object.
    const NAME: &'static str = "Boolean";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().boolean().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::to_string, "toString", 0)
        .method(Self::value_of, "valueOf", 0)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
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
        let data = args.get(0).map_or(false, JsValue::to_boolean);
        if new_target.is_undefined() {
            return Ok(JsValue::new(data));
        }
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::boolean, context)?;
        let boolean = JsObject::from_proto_and_data(prototype, ObjectData::boolean(data));

        Ok(boolean.into())
    }

    /// An Utility function used to get the internal `[[BooleanData]]`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisbooleanvalue
    fn this_boolean_value(value: &JsValue, context: &mut Context) -> JsResult<bool> {
        value
            .as_boolean()
            .or_else(|| value.as_object().and_then(|obj| obj.borrow().as_boolean()))
            .ok_or_else(|| context.construct_type_error("'this' is not a boolean"))
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
