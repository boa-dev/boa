use crate::{
    error::JsNativeError,
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsError,
};

pub(crate) mod logical;
pub(crate) mod macro_defined;

pub(crate) use logical::*;
pub(crate) use macro_defined::*;

/// `NotEq` implements the Opcode Operation for `Opcode::NotEq`
///
/// Operation:
///  - Binary `!=` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct NotEq;

impl Operation for NotEq {
    const NAME: &'static str = "NotEq";
    const INSTRUCTION: &'static str = "INST - NotEq";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();
        let value = !(ok_or_throw_completion!(lhs.equals(&rhs, context), context));
        context.vm.push(value);
        CompletionType::Normal
    }
}

/// `StrictEq` implements the Opcode Operation for `Opcode::StrictEq`
///
/// Operation:
///  - Binary `===` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct StrictEq;

impl Operation for StrictEq {
    const NAME: &'static str = "StrictEq";
    const INSTRUCTION: &'static str = "INST - StrictEq";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();
        context.vm.push(lhs.strict_equals(&rhs));
        CompletionType::Normal
    }
}

/// `StrictNotEq` implements the Opcode Operation for `Opcode::StrictNotEq`
///
/// Operation:
///  - Binary `!==` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct StrictNotEq;

impl Operation for StrictNotEq {
    const NAME: &'static str = "StrictNotEq";
    const INSTRUCTION: &'static str = "INST - StrictNotEq";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();
        context.vm.push(!lhs.strict_equals(&rhs));
        CompletionType::Normal
    }
}

/// `In` implements the Opcode Operation for `Opcode::In`
///
/// Operation:
///  - Binary `in` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct In;

impl Operation for In {
    const NAME: &'static str = "In";
    const INSTRUCTION: &'static str = "INST - In";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let rhs = context.vm.pop();
        let lhs = context.vm.pop();

        let Some(rhs) = rhs.as_object() else {
            let err: JsError = JsNativeError::typ()
                .with_message(format!(
                    "right-hand side of 'in' should be an object, got `{}`",
                    rhs.type_of()
                ))
                .into();
            let err_as_value = err.to_opaque(context);
            context.vm.push(err_as_value);
            return CompletionType::Throw
        };
        let key = ok_or_throw_completion!(lhs.to_property_key(context), context);
        let value = ok_or_throw_completion!(rhs.has_property(key, context), context);
        context.vm.push(value);
        CompletionType::Normal
    }
}

/// `InPrivate` implements the Opcode Operation for `Opcode::InPrivate`
///
/// Operation:
///  - Binary `in` operation for private names.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InPrivate;

impl Operation for InPrivate {
    const NAME: &'static str = "InPrivate";
    const INSTRUCTION: &'static str = "INST - InPrivate";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.private_names[index as usize];
        let rhs = context.vm.pop();

        let Some(rhs) = rhs.as_object() else {
            let err: JsError = JsNativeError::typ()
                .with_message(format!(
                    "right-hand side of 'in' should be an object, got `{}`",
                    rhs.type_of()
                ))
                .into();
            let err_as_value = err.to_opaque(context);
            context.vm.push(err_as_value);
            return CompletionType::Throw;
        };
        if rhs.private_element_find(&name, true, true).is_some() {
            context.vm.push(true);
        } else {
            context.vm.push(false);
        }

        CompletionType::Normal
    }
}

/// `InstanceOf` implements the Opcode Operation for `Opcode::InstanceOf`
///
/// Operation:
///  - Binary `instanceof` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct InstanceOf;

impl Operation for InstanceOf {
    const NAME: &'static str = "InstanceOf";
    const INSTRUCTION: &'static str = "INST - InstanceOf";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let target = context.vm.pop();
        let v = context.vm.pop();
        let value = ok_or_throw_completion!(v.instance_of(&target, context), context);

        context.vm.push(value);
        CompletionType::Normal
    }
}
