use crate::{
    object::{
        internal_methods::InternalMethodContext, JsObject, CONSTRUCTOR, PROTOTYPE,
    },
    property::PropertyDescriptorBuilder,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue, builtins::{function::OrdinaryFunction, OrdinaryObject},
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
    const COST: u8 = 6;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let prototype_value = context.vm.pop();
        let prototype = match &prototype_value {
            JsValue::Object(proto) => Some(proto.clone()),
            JsValue::Null => None,
            JsValue::Undefined => Some(context.intrinsics().constructors().object().prototype()),
            _ => unreachable!(),
        };

        // 9.Let proto be OrdinaryObjectCreate(protoParent).
        let proto = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrdinaryObject,
        );
        let class = context.vm.pop();

        {
            let class_object = class.as_object().expect("class must be object");
            class_object
                .define_property_or_throw(
                    PROTOTYPE,
                    PropertyDescriptorBuilder::new()
                        .value(proto.clone())
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                    context,
                )
                .expect("cannot fail per spec");
            class_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("class must be function object")
                .set_home_object(proto.clone());
        }

        proto
            .__define_own_property__(
                &CONSTRUCTOR.into(),
                PropertyDescriptorBuilder::new()
                    .value(class)
                    .writable(true)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                &mut InternalMethodContext::new(context),
            )
            .expect("cannot fail per spec");

        context.vm.push(proto);
        Ok(CompletionType::Normal)
    }
}
