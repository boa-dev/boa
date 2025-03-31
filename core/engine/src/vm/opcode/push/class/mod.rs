use crate::{
    error::JsNativeError,
    object::PROTOTYPE,
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
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
    #[inline(always)]
    pub(crate) fn operation(
        (dst, class, superclass): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let class = registers.get(class.into());
        let superclass = registers.get(superclass.into());

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

        registers.set(dst.into(), proto_parent);
        Ok(())
    }
}

impl Operation for PushClassPrototype {
    const NAME: &'static str = "PushClassPrototype";
    const INSTRUCTION: &'static str = "INST - PushClassPrototype";
    const COST: u8 = 6;
}
