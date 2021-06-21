//! This module implements the `ArrayBuffer` object.
//!
//! The `ArrayBuffer` object is used to represent a generic, fixed-length raw binary data buffer.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-arraybuffer-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer

use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, GcObject, PROTOTYPE},
    property::Attribute,
    symbol::WellKnownSymbols,
    BoaProfiler, Context, Result, Value,
};

#[cfg(test)]
mod tests;

mod buffer;

#[derive(Debug, Clone)]
pub(crate) struct ArrayBuffer;

impl BuiltIn for ArrayBuffer {
    const NAME: &'static str = "ArrayBuffer";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let byte_length = ();

        let array_buffer_object = ConstructorBuilder::new(context, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .property(to_string_tag, Self::NAME, Self::attribute())
            .property("byte_length", byte_length.clone(), Self::attribute())
            .static_method(Self::is_view, "is_view", 1)
            .method(Self::slice, "slice", 2)
            .build();

        (Self::NAME, array_buffer_object.into(), Self::attribute())
    }
}

impl ArrayBuffer {
    pub(crate) const LENGTH: usize = 0;

    /// The constructor function that is used to create derived objects
    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        if new_target.is_undefined() {
            return context.throw_type_error(
                "calling a builtin ArrayBuffer constructor without new is forbidden",
            );
        }

        let fallback_prototype = context.standard_objects().object_object().prototype();
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or(fallback_prototype);

        // Consider first Arg if and only if it's a valid number to represent length. The rest is
        // ignored
        match args.len() {
            0 => Self::construct_array_buffer_empty(prototype, context),
            _ => Self::construct_array_buffer_length(prototype, &args[0], context),
        }
    }

    /// No argument constructor for `ArrayBuffer`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-constructor
    fn construct_array_buffer_empty(proto: GcObject, context: &mut Context) -> Result<Value> {
        todo!()
    }

    /// By length constructor for `ArrayBuffer`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-length
    fn construct_array_buffer_length(
        proto: GcObject,
        arg: &Value,
        context: &mut Context,
    ) -> Result<Value> {
        todo!()
    }

    /// Returns true if arg is one of the ArrayBuffer views, such as typed array objects or a DataView.
    /// Returns false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.isview
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer/isView
    pub(crate) fn is_view(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        todo!()
    }

    /// Returns a new ArrayBuffer whose contents are a copy of this ArrayBuffer's bytes from begin (inclusive)
    /// up to end (exclusive). If either begin or end is negative, it refers to an index from the end of the
    /// array, as opposed to from the beginning.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.prototype.slice
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer/slice
    pub(crate) fn slice(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        todo!()
    }

    /// The read-only size, in bytes, of the ArrayBuffer. This is established when the array is constructed and
    /// cannot be changed.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer.prototype.bytelength
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer/byteLength
    pub(crate) fn byte_length(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        todo!()
    }
}
