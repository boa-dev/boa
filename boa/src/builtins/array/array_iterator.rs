use crate::{
    builtins::{
        function::{make_builtin_fn, BuiltInFunction, Function, FunctionFlags},
        Array, Value,
    },
    object::{Object, ObjectData, PROTOTYPE},
    property::Property,
    Context, Result,
};
use gc::{Finalize, Trace};
use std::borrow::Borrow;

#[derive(Debug, Clone, Finalize, Trace)]
pub enum ArrayIterationKind {
    Key,
    Value,
    KeyAndValue,
}

#[derive(Debug, Clone, Finalize, Trace)]
pub struct ArrayIterator {
    array: Value,
    next_index: i32,
    kind: ArrayIterationKind,
}

impl ArrayIterator {
    fn new(array: Value, kind: ArrayIterationKind) -> Self {
        ArrayIterator {
            array,
            kind,
            next_index: 0,
        }
    }

    pub(crate) fn new_array_iterator(
        ctx: &Context,
        array: Value,
        kind: ArrayIterationKind,
    ) -> Result<Value> {
        let array_iterator = Value::new_object(Some(
            &ctx.realm()
                .environment
                .get_global_object()
                .expect("Could not get global object"),
        ));
        array_iterator.set_data(ObjectData::ArrayIterator(Self::new(array, kind)));
        array_iterator
            .as_object_mut()
            .expect("array iterator object")
            .set_prototype_instance(
                ctx.realm()
                    .environment
                    .get_binding_value("Object")
                    .expect("Object was not initialized")
                    .borrow()
                    .get_field(PROTOTYPE),
            );
        make_builtin_fn(Self::next, "next", &array_iterator, 0, ctx);
        let mut function = Object::function(
            Function::BuiltIn(
                BuiltInFunction(|v, _, _| Ok(v.clone())),
                FunctionFlags::CALLABLE,
            ),
            ctx.global_object()
                .get_field("Function")
                .get_field("prototype"),
        );
        function.insert_field("length", Value::from(0));

        let symbol_iterator = ctx.well_known_symbols().iterator_symbol();
        array_iterator.set_field(symbol_iterator, Value::from(function));
        Ok(array_iterator)
    }

    pub(crate) fn next(this: &Value, _args: &[Value], ctx: &mut Context) -> Result<Value> {
        if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(array_iterator) = object.as_array_iterator_mut() {
                let index = array_iterator.next_index;
                if array_iterator.array.is_undefined() {
                    return Ok(Self::create_iter_result_object(
                        ctx,
                        Value::undefined(),
                        true,
                    ));
                }
                let len = array_iterator
                    .array
                    .get_field("length")
                    .as_number()
                    .ok_or_else(|| ctx.construct_type_error("Not an array"))?
                    as i32;
                if array_iterator.next_index >= len {
                    array_iterator.array = Value::undefined();
                    return Ok(Self::create_iter_result_object(
                        ctx,
                        Value::undefined(),
                        true,
                    ));
                }
                array_iterator.next_index = index + 1;
                match array_iterator.kind {
                    ArrayIterationKind::Key => Ok(Self::create_iter_result_object(
                        ctx,
                        Value::integer(index),
                        false,
                    )),
                    ArrayIterationKind::Value => {
                        let element_value = array_iterator.array.get_field(index);
                        Ok(Self::create_iter_result_object(ctx, element_value, false))
                    }
                    ArrayIterationKind::KeyAndValue => {
                        let element_value = array_iterator.array.get_field(index);
                        let result = Array::make_array(
                            &Value::new_object(Some(
                                &ctx.realm()
                                    .environment
                                    .get_global_object()
                                    .expect("Could not get global object"),
                            )),
                            &[Value::integer(index), element_value],
                            ctx,
                        )?;
                        Ok(result)
                    }
                }
            } else {
                ctx.throw_type_error("`this` is not an ArrayIterator")
            }
        } else {
            ctx.throw_type_error("`this` is not an ArrayIterator")
        }
    }

    fn create_iter_result_object(ctx: &mut Context, value: Value, done: bool) -> Value {
        let object = Value::new_object(Some(
            &ctx.realm()
                .environment
                .get_global_object()
                .expect("Could not get global object"),
        ));
        let value_property = Property::default().value(value);
        let done_property = Property::default().value(Value::boolean(done));
        object.set_property("value", value_property);
        object.set_property("done", done_property);
        object
    }
}
