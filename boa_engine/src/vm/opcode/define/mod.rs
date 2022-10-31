use crate::{
    property::PropertyDescriptor,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsString, JsValue,
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code.bindings[index as usize];

        if binding_locator.is_global() {
            let key = context
                .interner()
                .resolve_expect(binding_locator.name().sym())
                .into_common(false);
            context.global_bindings_mut().entry(key).or_insert(
                PropertyDescriptor::builder()
                    .value(JsValue::Undefined)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
            );
        } else {
            context.realm.environments.put_value_if_uninitialized(
                binding_locator.environment_index(),
                binding_locator.binding_index(),
                JsValue::Undefined,
            );
        }
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let value = context.vm.pop();
        let binding_locator = context.vm.frame().code.bindings[index as usize];
        binding_locator.throw_mutate_immutable(context)?;

        if binding_locator.is_global() {
            let key = context
                .interner()
                .resolve_expect(binding_locator.name().sym())
                .into_common::<JsString>(false)
                .into();
            crate::object::internal_methods::global::global_set_no_receiver(&key, value, context)?;
        } else {
            context.realm.environments.put_value(
                binding_locator.environment_index(),
                binding_locator.binding_index(),
                value,
            );
        }
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let binding_locator = context.vm.frame().code.bindings[index as usize];
        context.realm.environments.put_value(
            binding_locator.environment_index(),
            binding_locator.binding_index(),
            JsValue::Undefined,
        );
        Ok(ShouldExit::False)
    }
}

macro_rules! implement_declaritives {
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

            fn execute(context: &mut Context) -> JsResult<ShouldExit> {
                let index = context.vm.read::<u32>();
                let value = context.vm.pop();
                let binding_locator = context.vm.frame().code.bindings[index as usize];
                context.realm.environments.put_value(
                    binding_locator.environment_index(),
                    binding_locator.binding_index(),
                    value,
                );
                Ok(ShouldExit::False)
            }
        }
    };
}

implement_declaritives!(DefInitLet, "Declare and initialize `let` type variable");
implement_declaritives!(DefInitConst, "Declare and initialize `const` type variable");
implement_declaritives!(DefInitArg, "Declare and initialize function arguments");
