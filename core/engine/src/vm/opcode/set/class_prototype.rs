use crate::value::JsVariant;
use crate::vm::opcode::VaryingOperand;
use crate::{
    builtins::{function::OrdinaryFunction, OrdinaryObject},
    object::{internal_methods::InternalMethodContext, JsObject, CONSTRUCTOR, PROTOTYPE},
    property::PropertyDescriptorBuilder,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `SetClassProtoType` implements the Opcode Operation for `Opcode::SetClassPrototype`
///
/// Operation:
///  - Set the prototype of a class object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetClassPrototype;

impl SetClassPrototype {
    #[allow(clippy::unnecessary_wraps)]
    #[inline(always)]
    pub(crate) fn operation(
        (dst, prototype, class): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let prototype = registers.get(prototype.into());
        let prototype = match prototype.variant() {
            JsVariant::Object(proto) => Some(proto.clone()),
            JsVariant::Null => None,
            JsVariant::Undefined => Some(context.intrinsics().constructors().object().prototype()),
            _ => unreachable!(),
        };

        // 9.Let proto be OrdinaryObjectCreate(protoParent).
        let proto = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            OrdinaryObject,
        );
        let class = registers.get(class.into());

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
                    .value(class.clone())
                    .writable(true)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                &mut InternalMethodContext::new(context),
            )
            .expect("cannot fail per spec");

        registers.set(dst.into(), proto.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetClassPrototype {
    const NAME: &'static str = "SetClassPrototype";
    const INSTRUCTION: &'static str = "INST - SetClassPrototype";
    const COST: u8 = 6;
}
