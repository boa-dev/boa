use crate::{
    builtins::function::OrdinaryFunction,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `SetHomeObject` implements the Opcode Operation for `Opcode::SetHomeObject`
///
/// Operation:
///  - Set home object internal slot of a function object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetHomeObject;

impl Operation for SetHomeObject {
    const NAME: &'static str = "SetHomeObject";
    const INSTRUCTION: &'static str = "INST - SetHomeObject";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.pop();
        let home = context.vm.pop();

        function
            .as_object()
            .expect("must be object")
            .downcast_mut::<OrdinaryFunction>()
            .expect("must be function object")
            .set_home_object(home.as_object().expect("must be object").clone());

        context.vm.push(home);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}
