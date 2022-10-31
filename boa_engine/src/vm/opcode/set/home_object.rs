use crate::{
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let function = context.vm.pop();
        let function_object = function.as_object().expect("must be object");
        let home = context.vm.pop();
        let home_object = home.as_object().expect("must be object");

        function_object
            .borrow_mut()
            .as_function_mut()
            .expect("must be function object")
            .set_home_object(home_object.clone());

        context.vm.push(home);
        context.vm.push(function);
        Ok(ShouldExit::False)
    }
}
