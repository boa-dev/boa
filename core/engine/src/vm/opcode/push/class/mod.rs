use crate::{
    error::JsNativeError,
    object::PROTOTYPE,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

pub(crate) mod field;
pub(crate) mod private;

pub(crate) use field::*;
pub(crate) use private::*;

/// `PushClassPrototype` implements the Opcode Operation for `Opcode::PushClassPrototype`
///
/// Operation:
///  - Get the prototype of a superclass and push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrototype;

impl PushClassPrototype {
    fn operation(
        dst: u32,
        class: u32,
        superclass: u32,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;
        let class = context
            .vm
            .frame()
            .read_value::<0>(operand_types, class, &context.vm);
        let superclass = context
            .vm
            .frame()
            .read_value::<1>(operand_types, superclass, &context.vm);

        // // Taken from `15.7.14 Runtime Semantics: ClassDefinitionEvaluation`:
        // <https://tc39.es/ecma262/#sec-runtime-semantics-classdefinitionevaluation>
        //
        // 8. Else
        //     f. If superclass is null, then
        let (proto_parent, constructor_parent) = if superclass.is_null() {
            // i. Let protoParent be null.
            // ii. Let constructorParent be %Function.prototype%.
            //
            // NOTE(HalidOdat): We set constructorParent to None, it is resolved in `SetClassPrototype` opcode.
            (JsValue::null(), None)

        // h. Else,
        } else if let Some(superclass) = superclass.as_constructor() {
            // i. Let protoParent be ? Get(superclass, "prototype").
            let proto = superclass.get(PROTOTYPE, context)?;

            // ii. If protoParent is not an Object and protoParent is not null, throw a TypeError exception.
            if !proto.is_object() && !proto.is_null() {
                return Err(JsNativeError::typ()
                    .with_message("superclass prototype must be an object or null")
                    .into());
            }

            // iii. Let constructorParent be superclass.
            (proto, Some(superclass.clone()))

        // g. Else if IsConstructor(superclass) is false, then
        } else {
            // i. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("superclass must be a constructor")
                .into());
        };

        let class_object = class.as_object().expect("class must be object");

        if let Some(constructor_parent) = constructor_parent {
            class_object.set_prototype(Some(constructor_parent));
        }

        context.vm.stack[(rp + dst) as usize] = proto_parent;
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassPrototype {
    const NAME: &'static str = "PushClassPrototype";
    const INSTRUCTION: &'static str = "INST - PushClassPrototype";
    const COST: u8 = 6;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = u32::from(context.vm.read::<u8>());
        let class = u32::from(context.vm.read::<u8>());
        let superclass = u32::from(context.vm.read::<u8>());
        Self::operation(dst, class, superclass, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = u32::from(context.vm.read::<u16>());
        let class = u32::from(context.vm.read::<u16>());
        let superclass = u32::from(context.vm.read::<u16>());
        Self::operation(dst, class, superclass, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u32>();
        let class = context.vm.read::<u32>();
        let superclass = context.vm.read::<u32>();
        Self::operation(dst, class, superclass, operand_types, context)
    }
}
