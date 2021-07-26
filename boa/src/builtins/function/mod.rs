//! This module implements the global `Function` object as well as creates Native Functions.
//!
//! Objects wrap `Function`s and expose them via call/construct slots.
//!
//! `The `Function` object is used for matching text with a pattern.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-function-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function

use crate::{
    builtins::BuiltIn,
    context::StandardObjects,
    object::{
        function::Function, internal_methods::get_prototype_from_constructor, ConstructorBuilder,
        FunctionBuilder, ObjectData,
    },
    property::Attribute,
    BoaProfiler, Context, JsResult, JsValue,
};

use super::JsArgs;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy)]
pub struct BuiltInFunctionObject;

impl BuiltInFunctionObject {
    pub const LENGTH: usize = 1;

    fn constructor(
        new_target: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype =
            get_prototype_from_constructor(new_target, StandardObjects::function_object, context)?;
        let this = JsValue::new_object(context);

        this.as_object()
            .expect("this should be an object")
            .set_prototype_instance(prototype.into());

        this.set_data(ObjectData::function(Function::Native {
            function: |_, _, _| Ok(JsValue::undefined()),
            constructable: true,
        }));
        Ok(this)
    }

    fn prototype(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    /// `Function.prototype.call`
    ///
    /// The call() method invokes self with the first argument as the `this` value.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.call
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call
    fn call(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg = args.get_or_undefined(0);
        // TODO?: 3. Perform PrepareForTailCall
        let start = if !args.is_empty() { 1 } else { 0 };
        context.call(this, this_arg, &args[start..])
    }

    /// `Function.prototype.apply`
    ///
    /// The apply() method invokes self with the first argument as the `this` value
    /// and the rest of the arguments provided as an array (or an array-like object).
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.apply
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/apply
    fn apply(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg = args.get_or_undefined(0);
        let arg_array = args.get_or_undefined(1);
        if arg_array.is_null_or_undefined() {
            // TODO?: 3.a. PrepareForTailCall
            return context.call(this, this_arg, &[]);
        }
        let arg_list = arg_array.create_list_from_array_like(&[], context)?;
        // TODO?: 5. PrepareForTailCall
        context.call(this, this_arg, &arg_list)
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_string(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = {
            // Is there a case here where if there is no name field on a value
            // name should default to None? Do all functions have names set?
            let value = this.get_field("name", &mut *context)?;
            if value.is_null_or_undefined() {
                None
            } else {
                Some(value.to_string(context)?)
            }
        };

        let function = {
            let object = this
                .as_object()
                .map(|object| object.borrow().as_function().cloned());

            if let Some(Some(function)) = object {
                function
            } else {
                return context.throw_type_error("Not a function");
            }
        };

        match (&function, name) {
            (
                Function::Native {
                    function: _,
                    constructable: _,
                },
                Some(name),
            ) => Ok(format!("function {}() {{\n  [native Code]\n}}", &name).into()),
            (Function::Ordinary { body, params, .. }, Some(name)) => {
                let arguments: String = params
                    .iter()
                    .map(|param| param.name())
                    .collect::<Vec<&str>>()
                    .join(", ");

                let statement_list = &*body;
                // This is a kluge. The implementaion in browser seems to suggest that
                // the value here is printed exactly as defined in source. I'm not sure if
                // that's possible here, but for now here's a dumb heuristic that prints functions
                let is_multiline = {
                    let value = statement_list.to_string();
                    value.lines().count() > 1
                };
                if is_multiline {
                    Ok(
                        // ?? For some reason statement_list string implementation
                        // sticks a \n at the end no matter what
                        format!(
                            "{}({}) {{\n{}}}",
                            &name,
                            arguments,
                            statement_list.to_string()
                        )
                        .into(),
                    )
                } else {
                    Ok(format!(
                        "{}({}) {{{}}}",
                        &name,
                        arguments,
                        // The trim here is to remove a \n stuck at the end
                        // of the statement_list to_string method
                        statement_list.to_string().trim()
                    )
                    .into())
                }
            }

            _ => Ok("TODO".into()),
        }
    }
}

impl BuiltIn for BuiltInFunctionObject {
    const NAME: &'static str = "Function";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event("function", "init");

        let function_prototype = context.standard_objects().function_object().prototype();
        FunctionBuilder::native(context, Self::prototype)
            .name("")
            .length(0)
            .constructable(false)
            .build_function_prototype(&function_prototype);

        let function_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().function_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::call, "call", 1)
        .method(Self::apply, "apply", 1)
        .method(Self::to_string, "toString", 0)
        .build();

        (Self::NAME, function_object.into(), Self::attribute())
    }
}
