use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
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
    fn operation(index: usize, context: &mut Context) -> JsResult<CompletionType> {
        // TODO: spec specifies to return `empty` on empty vars, but we're trying to initialize.
        let binding_locator = context.vm.frame().code_block.bindings[index].clone();

        context.vm.environments.put_value_if_uninitialized(
            binding_locator.scope(),
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

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(index, context)
    }

    fn execute_u16(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(index, context)
    }

    fn execute_u32(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(index, context)
    }
}

/// `DefInitVar` implements the Opcode Operation for `Opcode::DefInitVar`
///
/// Operation:
///  - Declare and initialize a function argument.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefInitVar;

impl DefInitVar {
    fn operation(
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let frame = context.vm.frame();
        let strict = frame.code_block.strict();
        let mut binding_locator = frame.code_block.bindings[index].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        context.set_binding(&binding_locator, value.clone(), strict)?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for DefInitVar {
    const NAME: &'static str = "DefInitVar";
    const INSTRUCTION: &'static str = "INST - DefInitVar";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(value, index, registers, context)
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
    fn operation(
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let binding_locator = context.vm.frame().code_block.bindings[index].clone();
        context.vm.environments.put_lexical_value(
            binding_locator.scope(),
            binding_locator.binding_index(),
            value.clone(),
        );

        Ok(CompletionType::Normal)
    }
}

impl Operation for PutLexicalValue {
    const NAME: &'static str = "PutLexicalValue";
    const INSTRUCTION: &'static str = "INST - PutLexicalValue";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(value, index, registers, context)
    }
}
