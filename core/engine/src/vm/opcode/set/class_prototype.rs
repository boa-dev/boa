use crate::{
    builtins::{function::OrdinaryFunction, OrdinaryObject},
    object::{internal_methods::InternalMethodContext, JsObject, CONSTRUCTOR, PROTOTYPE},
    property::PropertyDescriptorBuilder,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `SetClassProtoType` implements the Opcode Operation for `Opcode::SetClassPrototype`
///
/// Operation:
///  - Set the prototype of a class object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetClassPrototype;

impl SetClassPrototype {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        dst: u32,
        prototype: u32,
        class: u32,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;
        let prototype_value = &context.vm.stack[(rp + prototype) as usize];
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
        let class = context.vm.stack[(rp + class) as usize].clone();

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

        context.vm.stack[(rp + dst) as usize] = proto.into();
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetClassPrototype {
    const NAME: &'static str = "SetClassPrototype";
    const INSTRUCTION: &'static str = "INST - SetClassPrototype";
    const COST: u8 = 6;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        let prototype = u32::from(context.vm.read::<u8>());
        let class = u32::from(context.vm.read::<u8>());
        Self::operation(dst, prototype, class, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        let prototype = u32::from(context.vm.read::<u16>());
        let class = u32::from(context.vm.read::<u16>());
        Self::operation(dst, prototype, class, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let prototype = context.vm.read::<u32>();
        let class = context.vm.read::<u32>();
        Self::operation(dst, prototype, class, context)
    }
}
