use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

pub(crate) mod class;
pub(crate) mod own_property;

pub(crate) use class::*;
pub(crate) use own_property::*;

/// `DefVar` implements the Opcode Operation for `Opcode::DefVar`
///
/// Operation:
///  - Declare `var` type variable.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefVar;

impl Operation for DefVar {
    const NAME: &'static str = "DefVar";
    const INSTRUCTION: &'static str = "INST - DefVar";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        // TODO: spec specifies to return `empty` on empty vars, but we're trying to initialize.
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code_block.bindings[index as usize];

        if binding_locator.is_global() {
            // already initialized at compile time
        } else {
            context.vm.environments.put_value_if_uninitialized(
                binding_locator.environment_index(),
                binding_locator.binding_index(),
                JsValue::Undefined,
            );
        }
        Ok(CompletionType::Normal)
    }
}

/// `DefInitVar` implements the Opcode Operation for `Opcode::DefInitVar`
///
/// Operation:
///  - Declare and initialize a function argument.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefInitVar;

impl Operation for DefInitVar {
    const NAME: &'static str = "DefInitVar";
    const INSTRUCTION: &'static str = "INST - DefInitVar";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let mut binding_locator = context.vm.frame().code_block.bindings[index as usize];
        if binding_locator.is_silent() {
            return Ok(CompletionType::Normal);
        }
        binding_locator.throw_mutate_immutable(context)?;

        context.find_runtime_binding(&mut binding_locator)?;

        context.set_binding(binding_locator, value, context.vm.frame().code_block.strict)?;

        Ok(CompletionType::Normal)
    }
}

/// `DefLet` implements the Opcode Operation for `Opcode::DefLet`
///
/// Operation:
///  - Declare `let` type variable.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefLet;

impl Operation for DefLet {
    const NAME: &'static str = "DefLet";
    const INSTRUCTION: &'static str = "INST - DefLet";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code_block.bindings[index as usize];
        context.vm.environments.put_declarative_value(
            binding_locator.environment_index(),
            binding_locator.binding_index(),
            JsValue::Undefined,
        );
        Ok(CompletionType::Normal)
    }
}

macro_rules! implement_declaratives {
    ($name:ident, $doc_string:literal) => {
        #[doc= concat!("`", stringify!($name), "` implements the OpCode Operation for `Opcode::", stringify!($name), "`\n")]
        #[doc= "\n"]
        #[doc="Operation:\n"]
        #[doc= concat!(" - ", $doc_string)]
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct $name;

        impl Operation for $name {
            const NAME: &'static str = stringify!($name);
            const INSTRUCTION: &'static str = stringify!("INST - " + $name);

            fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
                let index = context.vm.read::<u32>();
                let value = context.vm.pop();
                let binding_locator = context.vm.frame().code_block.bindings[index as usize];
                context.vm.environments.put_declarative_value(
                    binding_locator.environment_index(),
                    binding_locator.binding_index(),
                    value,
                );
                Ok(CompletionType::Normal)
            }
        }
    };
}

implement_declaratives!(DefInitLet, "Declare and initialize `let` type variable");
implement_declaratives!(DefInitConst, "Declare and initialize `const` type variable");
implement_declaratives!(DefInitArg, "Declare and initialize function arguments");
