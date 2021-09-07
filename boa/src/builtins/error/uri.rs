//! This module implements the global `URIError` object.
//!
//! The `URIError` object represents an error when a global URI handling
//! function was used in a wrong way.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-urierror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/URIError

use crate::{
    builtins::BuiltIn,
    context::StandardObjects,
    object::{internal_methods::get_prototype_from_constructor, ConstructorBuilder, ObjectData},
    profiler::BoaProfiler,
    property::Attribute,
    Context, JsResult, JsValue,
};

/// JavaScript `URIError` impleentation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct UriError;

impl BuiltIn for UriError {
    const NAME: &'static str = "URIError";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let error_prototype = context.standard_objects().error_object().prototype();
        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        let uri_error_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().uri_error_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(error_prototype.into())
        .property("name", Self::NAME, attribute)
        .property("message", "", attribute)
        .build();

        (Self::NAME, uri_error_object.into(), Self::attribute())
    }
}

impl UriError {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype =
            get_prototype_from_constructor(new_target, StandardObjects::error_object, context)?;
        let obj = context.construct_object();
        obj.set_prototype_instance(prototype.into());
        let this = JsValue::new(obj);
        if let Some(message) = args.get(0) {
            if !message.is_undefined() {
                this.set_field("message", message.to_string(context)?, false, context)?;
            }
        }

        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        this.set_data(ObjectData::error());
        Ok(this)
    }
}
