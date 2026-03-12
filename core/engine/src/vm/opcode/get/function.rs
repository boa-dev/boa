use crate::{
    Context,
    builtins::function::OrdinaryFunction,
    vm::{
        code_block::create_function_object_fast,
        opcode::{IndexOperand, Operation, RegisterOperand},
    },
};

/// `GetFunction` implements the Opcode Operation for `Opcode::GetFunction`
///
/// Operation:
///  - Get function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetFunction;

impl GetFunction {
    #[inline(always)]
    pub(crate) fn operation((dst, index): (RegisterOperand, IndexOperand), context: &mut Context) {
        let code = context
            .vm
            .frame()
            .code_block()
            .constant_function(index.into());
        let function = create_function_object_fast(code, context);
        context.vm.set_register(dst.into(), function.into());
    }
}

impl Operation for GetFunction {
    const NAME: &'static str = "GetFunction";
    const INSTRUCTION: &'static str = "INST - GetFunction";
    const COST: u8 = 3;
}

/// `SetArrowLexicalThis` implements the Opcode Operation for `Opcode::SetArrowLexicalThis`
///
/// Operation:
///  - Set the captured lexical `this` on an arrow function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetArrowLexicalThis;

impl SetArrowLexicalThis {
    #[inline(always)]
    pub(crate) fn operation(
        (function, this_value): (RegisterOperand, RegisterOperand),
        context: &mut Context,
    ) {
        let this = context.vm.get_register(this_value.into()).clone();
        let func_obj = context
            .vm
            .get_register(function.into())
            .as_object()
            .expect("SetArrowLexicalThis: register must hold an object")
            .clone();
        func_obj
            .downcast_mut::<OrdinaryFunction>()
            .expect("SetArrowLexicalThis: object must be an OrdinaryFunction")
            .lexical_this = Some(this);
    }
}

impl Operation for SetArrowLexicalThis {
    const NAME: &'static str = "SetArrowLexicalThis";
    const INSTRUCTION: &'static str = "INST - SetArrowLexicalThis";
    const COST: u8 = 3;
}
