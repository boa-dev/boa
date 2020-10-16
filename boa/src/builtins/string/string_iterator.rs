use crate::builtins::string::code_point_at;
use crate::{
    builtins::{function::make_builtin_fn, iterable::create_iter_result_object},
    object::ObjectData,
    property::{Attribute, DataDescriptor},
    BoaProfiler, Context, Result, Value,
};
use gc::{Finalize, Trace};

#[derive(Debug, Clone, Finalize, Trace)]
pub struct StringIterator {
    string: Value,
    next_index: i32,
}

impl StringIterator {
    fn new(string: Value) -> Self {
        Self {
            string,
            next_index: 0,
        }
    }

    pub fn create_string_iterator(ctx: &mut Context, string: Value) -> Result<Value> {
        let string_iterator = Value::new_object(Some(ctx.global_object()));
        string_iterator.set_data(ObjectData::StringIterator(Self::new(string)));
        string_iterator
            .as_object()
            .expect("array iterator object")
            .set_prototype_instance(ctx.iterator_prototypes().string_iterator().into());
        Ok(string_iterator)
    }

    pub fn next(this: &Value, _args: &[Value], ctx: &mut Context) -> Result<Value> {
        if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(string_iterator) = object.as_string_iterator_mut() {
                if string_iterator.string.is_undefined() {
                    return Ok(create_iter_result_object(ctx, Value::undefined(), true));
                }
                let native_string = string_iterator.string.to_string(ctx)?;
                let len = native_string.encode_utf16().count() as i32;
                let position = string_iterator.next_index;
                if position >= len {
                    string_iterator.string = Value::undefined();
                    return Ok(create_iter_result_object(ctx, Value::undefined(), true));
                }
                let (_, code_unit_count, _) =
                    code_point_at(native_string, position).expect("Invalid code point position");
                string_iterator.next_index += code_unit_count as i32;
                let result_string = crate::builtins::string::String::substring(
                    &string_iterator.string,
                    &[position.into(), string_iterator.next_index.into()],
                    ctx,
                )?;
                Ok(create_iter_result_object(ctx, result_string, false))
            } else {
                ctx.throw_type_error("`this` is not an ArrayIterator")
            }
        } else {
            ctx.throw_type_error("`this` is not an ArrayIterator")
        }
    }

    /// Create the %ArrayIteratorPrototype% object
    ///
    /// More information:
    ///  - [ECMA reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
    pub(crate) fn create_prototype(ctx: &mut Context, iterator_prototype: Value) -> Value {
        let global = ctx.global_object();
        let _timer = BoaProfiler::global().start_event("String Iterator", "init");

        // Create prototype
        let array_iterator = Value::new_object(Some(global));
        make_builtin_fn(Self::next, "next", &array_iterator, 0, ctx);
        array_iterator
            .as_object()
            .expect("array iterator prototype object")
            .set_prototype_instance(iterator_prototype);

        let to_string_tag = ctx.well_known_symbols().to_string_tag_symbol();
        let to_string_tag_property =
            DataDescriptor::new("String Iterator", Attribute::CONFIGURABLE);
        array_iterator.set_property(to_string_tag, to_string_tag_property);
        array_iterator
    }
}
