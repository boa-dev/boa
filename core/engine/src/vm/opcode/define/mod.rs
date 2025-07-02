use super::VaryingOperand;
use crate::{Context, JsResult, JsValue, vm::opcode::Operation};

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
    #[inline(always)]
    pub(super) fn operation(index: VaryingOperand, context: &mut Context) {
        // TODO: spec specifies to return `empty` on empty vars, but we're trying to initialize.
        let binding_locator = context.vm.frame().code_block.bindings[usize::from(index)].clone();

        context.vm.environments.put_value_if_uninitialized(
            binding_locator.scope(),
            binding_locator.binding_index(),
            JsValue::undefined(),
        );
    }
}

impl Operation for DefVar {
    const NAME: &'static str = "DefVar";
    const INSTRUCTION: &'static str = "INST - DefVar";
    const COST: u8 = 3;
}

/// `DefInitVar` implements the Opcode Operation for `Opcode::DefInitVar`
///
/// Operation:
///  - Declare and initialize a function argument.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefInitVar;

impl DefInitVar {
    #[inline(always)]
    pub(super) fn operation(
        (value, index): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let value = context.vm.get_register(value.into()).clone();
        let frame = context.vm.frame();
        let strict = frame.code_block.strict();
        let mut binding_locator = frame.code_block.bindings[usize::from(index)].clone();
        context.find_runtime_binding(&mut binding_locator)?;
        context.set_binding(&binding_locator, value.clone(), strict)?;

        Ok(())
    }
}

impl Operation for DefInitVar {
    const NAME: &'static str = "DefInitVar";
    const INSTRUCTION: &'static str = "INST - DefInitVar";
    const COST: u8 = 3;
}

/// `PutLexicalValue` implements the Opcode Operation for `Opcode::PutLexicalValue`
///
/// Operation:
///  - Initialize a lexical binding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PutLexicalValue;

impl PutLexicalValue {
    #[inline(always)]
    pub(super) fn operation(
        (value, index): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        let value = context.vm.get_register(value.into());
        let binding_locator = context.vm.frame().code_block.bindings[usize::from(index)].clone();
        context.vm.environments.put_lexical_value(
            binding_locator.scope(),
            binding_locator.binding_index(),
            value.clone(),
        );
    }
}

impl Operation for PutLexicalValue {
    const NAME: &'static str = "PutLexicalValue";
    const INSTRUCTION: &'static str = "INST - PutLexicalValue";
    const COST: u8 = 3;
}
