use crate::{
    builtins::function::OrdinaryFunction,
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsResult,
};

/// `SetHomeObject` implements the Opcode Operation for `Opcode::SetHomeObject`
///
/// Operation:
///  - Set home object internal slot of a function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetHomeObject;

impl SetHomeObject {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn operation(
        (function, home): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        _: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function.into());
        let home = registers.get(home.into());

        function
            .as_object()
            .expect("must be object")
            .downcast_mut::<OrdinaryFunction>()
            .expect("must be function object")
            .set_home_object(home.as_object().expect("must be object").clone());

        Ok(CompletionType::Normal)
    }
}

impl Operation for SetHomeObject {
    const NAME: &'static str = "SetHomeObject";
    const INSTRUCTION: &'static str = "INST - SetHomeObject";
    const COST: u8 = 4;
}
