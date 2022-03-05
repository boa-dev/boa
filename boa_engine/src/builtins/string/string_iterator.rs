use crate::{
    builtins::{
        function::make_builtin_fn, iterable::create_iter_result_object, string::code_point_at,
    },
    object::{JsObject, ObjectData},
    property::PropertyDescriptor,
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

#[derive(Debug, Clone, Finalize, Trace)]
pub struct StringIterator {
    string: JsValue,
    next_index: i32,
}

impl StringIterator {
    fn new(string: JsValue) -> Self {
        Self {
            string,
            next_index: 0,
        }
    }

    pub fn create_string_iterator(string: JsValue, context: &mut Context) -> JsResult<JsValue> {
        let string_iterator = JsObject::from_proto_and_data(
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .string_iterator(),
            ObjectData::string_iterator(Self::new(string)),
        );
        Ok(string_iterator.into())
    }

    pub fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let mut string_iterator = this.as_object().map(JsObject::borrow_mut);
        let string_iterator = string_iterator
            .as_mut()
            .and_then(|obj| obj.as_string_iterator_mut())
            .ok_or_else(|| context.construct_type_error("`this` is not an ArrayIterator"))?;

        if string_iterator.string.is_undefined() {
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }
        let native_string = string_iterator.string.to_string(context)?;
        let len = native_string.encode_utf16().count() as i32;
        let position = string_iterator.next_index;
        if position >= len {
            string_iterator.string = JsValue::undefined();
            return Ok(create_iter_result_object(
                JsValue::undefined(),
                true,
                context,
            ));
        }
        let (_, code_unit_count, _) = code_point_at(&native_string, position as usize);
        string_iterator.next_index += i32::from(code_unit_count);
        let result_string = crate::builtins::string::String::substring(
            &string_iterator.string,
            &[position.into(), string_iterator.next_index.into()],
            context,
        )?;
        Ok(create_iter_result_object(result_string, false, context))
    }

    /// Create the `%ArrayIteratorPrototype%` object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(
        iterator_prototype: JsObject,
        context: &mut Context,
    ) -> JsObject {
        let _timer = Profiler::global().start_event("String Iterator", "init");

        // Create prototype
        let array_iterator =
            JsObject::from_proto_and_data(iterator_prototype, ObjectData::ordinary());
        make_builtin_fn(Self::next, "next", &array_iterator, 0, context);

        let to_string_tag = WellKnownSymbols::to_string_tag();
        let to_string_tag_property = PropertyDescriptor::builder()
            .value("String Iterator")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        array_iterator.insert(to_string_tag, to_string_tag_property);
        array_iterator
    }
}
