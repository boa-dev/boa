//! This module implements the global `SyntaxError` object.
//!
//! The SyntaxError object represents an error when trying to interpret syntactically invalid code.
//! It is thrown when the JavaScript context encounters tokens or token order that does not conform
//! to the syntax of the language when parsing code.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SyntaxError

use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, ObjectData, PROTOTYPE},
    profiler::BoaProfiler,
    property::Attribute,
    Context, Result, Value,
};

/// JavaScript `SyntaxError` impleentation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SyntaxError;

impl BuiltIn for SyntaxError {
    const NAME: &'static str = "SyntaxError";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let error_prototype = context.standard_objects().error_object().prototype();
        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        let syntax_error_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().syntax_error_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(error_prototype.into())
        .property("name", Self::NAME, attribute)
        .property("message", "", attribute)
        .build();

        (Self::NAME, syntax_error_object.into(), Self::attribute())
    }
}

impl SyntaxError {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.__get__(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.standard_objects().error_object().prototype());
        let obj = context.construct_object();
        obj.set_prototype_instance(prototype.into());
        let this = Value::from(obj);
        if let Some(message) = args.get(0) {
            if !message.is_undefined() {
                this.set_field("message", message.to_string(context)?, false, context)?;
            }
        }

        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        this.set_data(ObjectData::Error);
        Ok(this)
    }
}
