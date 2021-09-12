use crate::{
    builtins::{
        function::make_builtin_fn, iterable::create_iter_result_object, string::code_point_at,
    },
    gc::{Finalize, Trace},
    object::{JsObject, ObjectData},
    property::PropertyDescriptor,
    symbol::WellKnownSymbols,
    BoaProfiler, Context, JsResult, JsValue,
};

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
        let string_iterator = JsValue::new_object(context);
        string_iterator.set_data(ObjectData::string_iterator(Self::new(string)));
        string_iterator
            .as_object()
            .expect("array iterator object")
            .set_prototype_instance(context.iterator_prototypes().string_iterator().into());
        Ok(string_iterator)
    }

    pub fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let JsValue::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(string_iterator) = object.as_string_iterator_mut() {
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
                let (_, code_unit_count, _) =
                    code_point_at(native_string, position).expect("Invalid code point position");
                string_iterator.next_index += code_unit_count as i32;
                let result_string = crate::builtins::string::String::substring(
                    &string_iterator.string,
                    &[position.into(), string_iterator.next_index.into()],
                    context,
                )?;
                Ok(create_iter_result_object(result_string, false, context))
            } else {
                context.throw_type_error("`this` is not an ArrayIterator")
            }
        } else {
            context.throw_type_error("`this` is not an ArrayIterator")
        }
    }

    /// Create the %ArrayIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(iterator_prototype: JsValue, context: &mut Context) -> JsObject {
        let _timer = BoaProfiler::global().start_event("String Iterator", "init");

        // Create prototype
        let array_iterator = context.construct_object();
        make_builtin_fn(Self::next, "next", &array_iterator, 0, context);
        array_iterator.set_prototype_instance(iterator_prototype);

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
