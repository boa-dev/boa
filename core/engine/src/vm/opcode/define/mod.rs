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
    pub(super) fn operation(index: VaryingOperand, context: &Context) {
        // TODO: spec specifies to return `empty` on empty vars, but we're trying to initialize.
        let binding_locator = context.vm_mut().frame().code_block.bindings[usize::from(index)].clone();

        context.vm_mut().frame.environments.put_value_if_uninitialized(
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
        context: &Context,
    ) -> JsResult<()> {
        let (value, strict, mut binding_locator) = {
            let vm = context.vm_mut();
            let value = vm.get_register(value.into()).clone();
            let strict = vm.frame().code_block.strict();
            let binding_locator = vm.frame().code_block.bindings[usize::from(index)].clone();
            (value, strict, binding_locator)
        };
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
    pub(super) fn operation((value, index): (VaryingOperand, VaryingOperand), context: &Context) {
        let vm = context.vm_mut();
        let value = vm.get_register(value.into()).clone();
        let binding_locator = vm.frame().code_block.bindings[usize::from(index)].clone();
        vm.frame.environments.put_lexical_value(
            binding_locator.scope(),
            binding_locator.binding_index(),
            value,
        );
    }
}

impl Operation for PutLexicalValue {
    const NAME: &'static str = "PutLexicalValue";
    const INSTRUCTION: &'static str = "INST - PutLexicalValue";
    const COST: u8 = 3;
}
