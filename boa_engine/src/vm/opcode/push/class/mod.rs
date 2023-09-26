use crate::{
    builtins::function::{ConstructorKind, FunctionKind},
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

impl Operation for PushClassPrototype {
    const NAME: &'static str = "PushClassPrototype";
    const INSTRUCTION: &'static str = "INST - PushClassPrototype";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let superclass = context.vm.pop();
        let class = context.vm.pop();

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

        let mut class_object_mut = class_object.borrow_mut();
        let class_function = class_object_mut
            .as_function_mut()
            .expect("class must be function object");

        // 17. If ClassHeritageopt is present, set F.[[ConstructorKind]] to derived.
        if let FunctionKind::Ordinary {
            constructor_kind, ..
        } = class_function.kind_mut()
        {
            *constructor_kind = ConstructorKind::Derived;
        }

        drop(class_object_mut);

        context.vm.push(class);
        context.vm.push(proto_parent);

        Ok(CompletionType::Normal)
    }
}
