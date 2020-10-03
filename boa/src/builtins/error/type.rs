//! This module implements the global `TypeError` object.
//!
//! The `TypeError` object represents an error when an operation could not be performed,
//! typically (but not exclusively) when a value is not of the expected type.
//!
//! A `TypeError` may be thrown when:
//!  - an operand or argument passed to a function is incompatible with the type expected by that operator or function.
//!  - when attempting to modify a value that cannot be changed.
//!  - when attempting to use a value in an inappropriate way.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError

use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, ObjectData},
    property::Attribute,
    BoaProfiler, Context, Result, Value,
};

/// JavaScript `TypeError` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TypeError;

impl BuiltIn for TypeError {
    const NAME: &'static str = "TypeError";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let error_prototype = context.standard_objects().error_object().prototype();
        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        let type_error_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().type_error_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(error_prototype.into())
        .property("name", Self::NAME, attribute)
        .property("message", "", attribute)
        .build();

        (Self::NAME, type_error_object.into(), Self::attribute())
    }
}

impl TypeError {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn constructor(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        if let Some(message) = args.get(0) {
            this.set_field("message", message.to_string(ctx)?);
        }

        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        this.set_data(ObjectData::Error);
        Err(this.clone())
    }
}
