use crate::{
    object::{JsObject, ObjectData},
    property::PropertyDescriptorBuilder,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsValue,
};

/// `SetClassProtoType` implements the Opcode Operation for `Opcode::SetClassPrototype`
///
/// Operation:
///  - Set the prototype of a class object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetClassPrototype;

impl Operation for SetClassPrototype {
    const NAME: &'static str = "SetClassPrototype";
    const INSTRUCTION: &'static str = "INST - SetClassPrototype";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let prototype_value = context.vm.pop();
        let prototype = match &prototype_value {
            JsValue::Object(proto) => Some(proto.clone()),
            JsValue::Null => None,
            JsValue::Undefined => Some(
                context
                    .intrinsics()
                    .constructors()
                    .object()
                    .prototype
                    .clone(),
            ),
            _ => unreachable!(),
        };

        let proto = JsObject::from_proto_and_data(prototype, ObjectData::ordinary());
        let class = context.vm.pop();

        {
            let class_object = class.as_object().expect("class must be object");
            class_object
                .define_property_or_throw(
                    "prototype",
                    PropertyDescriptorBuilder::new()
                        .value(proto.clone())
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                    context,
                )
                .expect("cannot fail per spec");
            let mut class_object_mut = class_object.borrow_mut();
            let class_function = class_object_mut
                .as_function_mut()
                .expect("class must be function object");
            class_function.set_home_object(proto.clone());
        }

        proto
            .__define_own_property__(
                "constructor".into(),
                PropertyDescriptorBuilder::new()
                    .value(class)
                    .writable(true)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            )
            .expect("cannot fail per spec");

        context.vm.push(proto);
        Ok(ShouldExit::False)
    }
}
