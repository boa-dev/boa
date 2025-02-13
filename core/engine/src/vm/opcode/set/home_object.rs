use crate::{
    builtins::function::OrdinaryFunction,
    vm::{opcode::Operation, CompletionType, Registers},
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
    fn operation(
        function: u32,
        home: u32,
        registers: &mut Registers,
        _: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function);
        let home = registers.get(home);

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

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u8>().into();
        let home = context.vm.read::<u8>().into();
        Self::operation(function, home, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u16>().into();
        let home = context.vm.read::<u16>().into();
        Self::operation(function, home, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u32>();
        let home = context.vm.read::<u32>();
        Self::operation(function, home, registers, context)
    }
}
