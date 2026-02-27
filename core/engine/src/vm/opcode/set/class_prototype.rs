use crate::value::JsVariant;
use crate::vm::opcode::VaryingOperand;
use crate::{
    Context,
    builtins::{OrdinaryObject, function::OrdinaryFunction},
    object::{CONSTRUCTOR, JsObject, PROTOTYPE, internal_methods::InternalMethodPropertyContext},
    property::PropertyDescriptorBuilder,
    vm::opcode::Operation,
};

/// `SetClassProtoType` implements the Opcode Operation for `Opcode::SetClassPrototype`
///
/// Operation:
///  - Set the prototype of a class object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetClassPrototype;

impl SetClassPrototype {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, prototype, class): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &Context,
    ) {
        let prototype = context.vm_mut().get_register(prototype.into()).clone();
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
        )
        .upcast();
        let class = context.vm_mut().get_register(class.into()).clone();

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
                &mut InternalMethodPropertyContext::new(context),
            )
            .expect("cannot fail per spec");

        context.vm_mut().set_register(dst.into(), proto.into());
    }
}

impl Operation for SetClassPrototype {
    const NAME: &'static str = "SetClassPrototype";
    const INSTRUCTION: &'static str = "INST - SetClassPrototype";
    const COST: u8 = 6;
}
