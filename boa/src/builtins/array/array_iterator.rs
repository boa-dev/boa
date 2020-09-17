use crate::builtins::function::make_builtin_fn;
use crate::builtins::{Array, Value};
use crate::object::{ObjectData, PROTOTYPE};
use crate::property::Property;
use crate::{Context, Result};
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
        interpreter: &Context,
        array: Value,
        kind: ArrayIterationKind,
    ) -> Result<Value> {
        let array_iterator = Value::new_object(Some(
            &interpreter
                .realm()
                .environment
                .get_global_object()
                .expect("Could not get global object"),
        ));
        array_iterator.set_data(ObjectData::ArrayIterator(Self::new(array, kind)));
        array_iterator
            .as_object_mut()
            .expect("array iterator object")
            .set_prototype_instance(
                interpreter
                    .realm()
                    .environment
                    .get_binding_value("Object")
                    .expect("Object was not initialized")
                    .borrow()
                    .get_field(PROTOTYPE),
            );
        make_builtin_fn(Self::next, "next", &array_iterator, 0, interpreter);

        Ok(array_iterator)
    }

    pub(crate) fn next(this: &Value, _args: &[Value], interpreter: &mut Context) -> Result<Value> {
        if let Value::Object(ref object) = this {
            let mut object = object.borrow_mut();
            if let Some(array_iterator) = object.as_array_iterator_mut() {
                let index = array_iterator.next_index;
                if array_iterator.array == Value::undefined() {
                    return Ok(Self::create_iter_result_object(
                        interpreter,
                        Value::undefined(),
                        true,
                    ));
                }
                let len = array_iterator
                    .array
                    .get_field("length")
                    .as_number()
                    .ok_or_else(|| interpreter.construct_type_error("Not an array"))?
                    as i32;
                if array_iterator.next_index >= len {
                    array_iterator.array = Value::undefined();
                    return Ok(Self::create_iter_result_object(
                        interpreter,
                        Value::undefined(),
                        true,
                    ));
                }
                array_iterator.next_index = index + 1;
                match array_iterator.kind {
                    ArrayIterationKind::Key => Ok(Self::create_iter_result_object(
                        interpreter,
                        Value::integer(index),
                        false,
                    )),
                    ArrayIterationKind::Value => {
                        let element_value = array_iterator.array.get_field(index);
                        Ok(Self::create_iter_result_object(
                            interpreter,
                            element_value,
                            false,
                        ))
                    }
                    ArrayIterationKind::KeyAndValue => {
                        let element_value = array_iterator.array.get_field(index);
                        let result = Array::make_array(
                            &Value::new_object(Some(
                                &interpreter
                                    .realm()
                                    .environment
                                    .get_global_object()
                                    .expect("Could not get global object"),
                            )),
                            &[Value::integer(index), element_value],
                            interpreter,
                        )?;
                        Ok(result)
                    }
                }
            } else {
                interpreter.throw_type_error("`this` is not an ArrayIterator")
            }
        } else {
            interpreter.throw_type_error("`this` is not an ArrayIterator")
        }
    }

    fn create_iter_result_object(interpreter: &mut Context, value: Value, done: bool) -> Value {
        let object = Value::new_object(Some(
            &interpreter
                .realm()
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
