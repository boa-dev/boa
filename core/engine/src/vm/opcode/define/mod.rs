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

impl DefVar {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        // TODO: spec specifies to return `empty` on empty vars, but we're trying to initialize.
        let binding_locator = context.vm.frame().code_block.bindings[index].clone();

        context.vm.environments.put_value_if_uninitialized(
            binding_locator.environment_index(),
            binding_locator.binding_index(),
            JsValue::undefined(),
        );
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefVar {
    const NAME: &'static str = "DefVar";
    const INSTRUCTION: &'static str = "INST - DefVar";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
    }
}

/// `DefInitVar` implements the Opcode Operation for `Opcode::DefInitVar`
///
/// Operation:
///  - Declare and initialize a function argument.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefInitVar;

impl DefInitVar {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let mut binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        context.set_binding(
            &binding_locator,
            value,
            context.vm.frame().code_block.strict(),
        )?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for DefInitVar {
    const NAME: &'static str = "DefInitVar";
    const INSTRUCTION: &'static str = "INST - DefInitVar";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
    }
}

/// `PutLexicalValue` implements the Opcode Operation for `Opcode::PutLexicalValue`
///
/// Operation:
///  - Initialize a lexical binding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PutLexicalValue;

impl PutLexicalValue {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.vm.environments.put_lexical_value(
            binding_locator.environment_index(),
            binding_locator.binding_index(),
            value,
        );

        Ok(CompletionType::Normal)
    }
}

impl Operation for PutLexicalValue {
    const NAME: &'static str = "PutLexicalValue";
    const INSTRUCTION: &'static str = "INST - PutLexicalValue";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>();
        Self::operation(context, index as usize)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        Self::operation(context, index as usize)
    }
}

/// `CreateGlobalFunctionBinding` implements the Opcode Operation for `Opcode::CreateGlobalFunctionBinding`
///
/// Operation:
/// - Performs [`CreateGlobalFunctionBinding ( N, V, D )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateGlobalFunctionBinding;

impl CreateGlobalFunctionBinding {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        context: &mut Context,
        index: usize,
        configurable: bool,
    ) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block().constant_string(index);
        let value = context.vm.pop();

        let function = value
            .as_object()
            .expect("valeu should be an function")
            .clone();
        context.create_global_function_binding(name, function, configurable)?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for CreateGlobalFunctionBinding {
    const NAME: &'static str = "CreateGlobalFunctionBinding";
    const INSTRUCTION: &'static str = "INST - CreateGlobalFunctionBinding";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index, configurable)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index, configurable)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index, configurable)
    }
}
