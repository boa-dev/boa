use crate::{
    builtins::function::Function,
    object::{JsObject, ObjectData},
    property::PropertyDescriptor,
    Context, JsResult, JsValue,
};

#[derive(Debug, Default)]
pub struct IntrinsicObjects {
    throw_type_error: JsObject,
}

impl IntrinsicObjects {
    pub fn init(context: &mut Context) -> Self {
        Self {
            throw_type_error: create_throw_type_error(context),
        }
    }

    pub fn throw_type_error(&self) -> JsObject {
        self.throw_type_error.clone()
    }
}

fn create_throw_type_error(context: &mut Context) -> JsObject {
    fn throw_type_error(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        context.throw_type_error("invalid type")
    }

    let function = JsObject::from_proto_and_data(
        context.standard_objects().function_object().prototype(),
        ObjectData::function(Function::Native {
            function: throw_type_error,
            constructor: false,
        }),
    );

    let property = PropertyDescriptor::builder()
        .writable(false)
        .enumerable(false)
        .configurable(false);
    function.insert_property("name", property.clone().value("ThrowTypeError"));
    function.insert_property("length", property.value(0));

    function
}
