use crate::{
    Context,
    builtins::function::OrdinaryFunction,
    vm::opcode::{Operation, VaryingOperand},
};

/// `SetHomeObject` implements the Opcode Operation for `Opcode::SetHomeObject`
///
/// Operation:
///  - Set home object internal slot of a function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetHomeObject;

impl SetHomeObject {
    #[inline(always)]
    pub(crate) fn operation((function, home): (VaryingOperand, VaryingOperand), context: &Context) {
        let function = context.get_register(function.into());
        let home = context.get_register(home.into());

        function
            .as_object()
            .expect("must be object")
            .downcast_mut::<OrdinaryFunction>()
            .expect("must be function object")
            .set_home_object(home.as_object().expect("must be object"));
    }
}

impl Operation for SetHomeObject {
    const NAME: &'static str = "SetHomeObject";
    const INSTRUCTION: &'static str = "INST - SetHomeObject";
    const COST: u8 = 4;
}
